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
pub mod search;
pub mod show;
#[cfg(test)]
#[path = "mod_tests.rs"]
pub mod testing;
pub mod tree;

use std::path::Path;

use wk_core::{HlcClock, Op, OpPayload};

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

/// Generate an HLC timestamp for a new operation.
pub fn generate_hlc() -> wk_core::Hlc {
    // Use process ID as node ID for CLI operations
    let node_id = std::process::id();
    let clock = HlcClock::new(node_id);
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
        }
    }

    Ok(())
}

/// Create an operation from a payload and write it to the sync queue.
///
/// Convenience function that combines `generate_hlc`, `Op::new`, and `write_pending_op`.
pub fn queue_op(work_dir: &Path, config: &Config, payload: OpPayload) -> Result<()> {
    let hlc = generate_hlc();
    let op = Op::new(hlc, payload);
    write_pending_op(work_dir, config, &op)
}
