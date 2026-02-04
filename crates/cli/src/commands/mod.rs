// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

pub mod config;
pub mod daemon;
pub mod dep;
pub mod edit;
pub mod export;
pub mod filtering;
#[cfg(test)]
pub mod hlc_persistence;
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
pub mod schema;
pub mod search;
pub mod show;
#[cfg(test)]
#[path = "mod_tests.rs"]
pub mod testing;
pub mod tree;

use std::path::PathBuf;

use crate::config::{find_work_dir, get_daemon_dir, get_db_path, Config};
use crate::daemon::{get_socket_path, spawn_daemon, DaemonClient};
use crate::db::Database;
use crate::db_handle::DatabaseHandle;
use crate::error::Result;
use crate::models::Event;

/// Helper to open the database from the current context.
///
/// In user-level mode, this connects to the daemon.
/// In private mode, this opens SQLite directly.
pub fn open_db() -> Result<(Database, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = crate::time_phase!("db::open", { Database::open(&db_path)? });
    Ok((db, config, work_dir))
}

/// Open a database handle that routes through daemon in user-level mode.
///
/// In user-level mode (config.private = false):
/// - Auto-starts daemon if not running
/// - Returns DatabaseHandle::Daemon for IPC communication
///
/// In private mode (config.private = true):
/// - Opens SQLite directly
/// - Returns DatabaseHandle::Direct
///
// KEEP UNTIL: Commands are migrated to use DatabaseHandle for daemon routing.
#[allow(dead_code)]
pub fn open_db_handle() -> Result<(DatabaseHandle, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;

    let handle = if config.private {
        // Private mode: direct SQLite access
        let db_path = get_db_path(&work_dir, &config);
        let db = crate::time_phase!("db::open", { Database::open(&db_path)? });
        DatabaseHandle::Direct(db)
    } else {
        // User-level mode: connect to daemon
        let daemon_dir = get_daemon_dir(&config);
        let socket_path = get_socket_path(&daemon_dir);

        // Auto-start daemon if not running
        if !socket_path.exists() {
            crate::time_phase!("daemon::spawn", { spawn_daemon(&daemon_dir)? });
        }

        // Connect to daemon
        let client =
            crate::time_phase!("daemon::connect", { DaemonClient::connect(&socket_path)? });
        DatabaseHandle::Daemon(client)
    };

    Ok((handle, config, work_dir))
}

/// Apply a mutation by logging an event to the local database.
///
/// This helper handles the common pattern of logging an event for all
/// issue mutations to ensure a consistent audit trail.
pub fn apply_mutation(db: &Database, event: Event) -> Result<()> {
    db.log_event(&event)?;
    Ok(())
}

/// Apply a mutation by logging an event through the database handle.
///
/// This helper handles the common pattern of logging an event for all
/// issue mutations to ensure a consistent audit trail.
///
// KEEP UNTIL: Commands are migrated to use DatabaseHandle for daemon routing.
#[allow(dead_code)]
pub fn apply_mutation_handle(handle: &mut DatabaseHandle, event: Event) -> Result<()> {
    handle.log_event(&event)?;
    Ok(())
}
