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

use crate::config::{find_work_dir, get_db_path, Config};
use crate::db::Database;
use crate::error::Result;
use crate::models::Event;

/// Helper to open the database from the current context
pub fn open_db() -> Result<(Database, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = crate::time_phase!("db::open", { Database::open(&db_path)? });
    Ok((db, config, work_dir))
}

/// Apply a mutation by logging an event to the local database.
///
/// This helper handles the common pattern of logging an event for all
/// issue mutations to ensure a consistent audit trail.
pub fn apply_mutation(db: &Database, event: Event) -> Result<()> {
    db.log_event(&event)?;
    Ok(())
}
