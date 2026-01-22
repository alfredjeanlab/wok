// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

pub mod config;
pub mod dep;
pub mod edit;
pub mod export;
pub mod hooks;
pub mod import;
pub mod init;
pub mod label;
pub mod lifecycle;
pub mod link;
pub mod list;
pub mod log;
pub mod new;
pub mod note;
pub mod prime;
pub mod ready;
pub mod remote;
pub mod schema;
pub mod search;
pub mod show;
#[cfg(test)]
#[path = "mod_tests.rs"]
pub mod testing;
pub mod tree;

use std::path::Path;

use wk_core::{Hlc, HlcClock, Op, OpPayload};

use crate::config::{find_work_dir, get_daemon_dir, get_db_path, Config, RemoteType};
use crate::db::Database;
use crate::error::Result;
use crate::sync::OfflineQueue;
use crate::wal::Wal;

/// Helper to open the database from the current context
pub fn open_db() -> Result<(Database, Config)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    Ok((db, config))
}

/// Get the path to the persisted local HLC file (for generating new ops).
pub fn get_last_hlc_path(daemon_dir: &Path) -> std::path::PathBuf {
    daemon_dir.join("last_hlc.txt")
}

/// Get the path to the persisted server HLC file (for sync requests).
pub fn get_server_hlc_path(daemon_dir: &Path) -> std::path::PathBuf {
    daemon_dir.join("server_hlc.txt")
}

/// Read the last persisted local HLC from disk.
pub fn read_last_hlc(daemon_dir: &Path) -> Option<Hlc> {
    let path = get_last_hlc_path(daemon_dir);
    let content = std::fs::read_to_string(&path).ok()?;
    content.trim().parse().ok()
}

/// Read the last persisted server HLC from disk (for sync requests).
pub fn read_server_hlc(daemon_dir: &Path) -> Option<Hlc> {
    let path = get_server_hlc_path(daemon_dir);
    let content = std::fs::read_to_string(&path).ok()?;
    content.trim().parse().ok()
}

/// Write the local HLC high-water mark to disk.
pub fn write_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    use std::io::Write;

    let path = get_last_hlc_path(daemon_dir);
    let mut file = std::fs::File::create(&path)?;
    write!(file, "{}", hlc)?;
    file.sync_all()?;
    Ok(())
}

/// Write the server HLC high-water mark to disk.
pub fn write_server_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    use std::io::Write;

    let path = get_server_hlc_path(daemon_dir);
    let mut file = std::fs::File::create(&path)?;
    write!(file, "{}", hlc)?;
    file.sync_all()?;
    Ok(())
}

/// Update the persisted local HLC if the given HLC is greater than the current one.
pub fn update_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    if let Some(current) = read_last_hlc(daemon_dir) {
        if hlc > current {
            write_last_hlc(daemon_dir, hlc)?;
        }
    } else {
        write_last_hlc(daemon_dir, hlc)?;
    }
    Ok(())
}

/// Update the persisted server HLC if the given HLC is greater than the current one.
pub fn update_server_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    if let Some(current) = read_server_hlc(daemon_dir) {
        if hlc > current {
            write_server_hlc(daemon_dir, hlc)?;
        }
    } else {
        write_server_hlc(daemon_dir, hlc)?;
    }
    Ok(())
}

/// Generate an HLC timestamp incorporating the persisted high-water mark.
///
/// This ensures that new operations get timestamps higher than any
/// previously seen HLC, preventing clock skew issues in sync.
pub fn generate_hlc_with_context(daemon_dir: &Path) -> Hlc {
    let node_id = std::process::id();
    let clock = HlcClock::new(node_id);

    // Incorporate last seen HLC if available
    if let Some(last) = read_last_hlc(daemon_dir) {
        let _ = clock.receive(&last);
    }

    clock.now()
}

/// Write a pending operation to the sync queue if in remote mode.
///
/// This function handles both git and WebSocket remotes:
/// - Git remotes: writes to `pending_ops.jsonl` (WAL)
/// - WebSocket remotes: writes to `sync_queue.jsonl` (OfflineQueue)
///
/// If not in remote mode, this is a no-op.
pub fn write_pending_op(work_dir: &Path, config: &Config, op: &Op) -> Result<()> {
    // Only write if remote mode is configured
    let remote = match &config.remote {
        Some(r) => r,
        None => return Ok(()),
    };

    let daemon_dir = get_daemon_dir(work_dir, config);

    match remote.remote_type() {
        RemoteType::Git => {
            let wal_path = daemon_dir.join("pending_ops.jsonl");
            let wal = Wal::open(&wal_path)?;
            wal.append(op)?;
        }
        RemoteType::WebSocket => {
            let queue_path = daemon_dir.join("sync_queue.jsonl");
            let mut queue = OfflineQueue::open(&queue_path)
                .map_err(|e| crate::error::Error::Io(std::io::Error::other(e.to_string())))?;
            queue
                .enqueue(op)
                .map_err(|e| crate::error::Error::Io(std::io::Error::other(e.to_string())))?;

            // Notify daemon to push immediately (fire-and-forget)
            crate::daemon::notify_daemon_sync(&daemon_dir);
        }
    }

    Ok(())
}

/// Create an operation from a payload and write it to the sync queue.
///
/// Convenience function that combines `generate_hlc_with_context`, `Op::new`,
/// and `write_pending_op`. Uses context-aware HLC generation to ensure the
/// operation's timestamp is higher than any previously seen HLC.
pub fn queue_op(work_dir: &Path, config: &Config, payload: OpPayload) -> Result<()> {
    let daemon_dir = get_daemon_dir(work_dir, config);
    let hlc = generate_hlc_with_context(&daemon_dir);
    let op = Op::new(hlc, payload);

    // Persist the generated HLC so future operations are higher
    let _ = update_last_hlc(&daemon_dir, hlc);

    write_pending_op(work_dir, config, &op)
}
