// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! SQLite-backed database for issue storage.
//!
//! The [`Database`] struct provides all data access operations for issues,
//! events, notes, tags, dependencies, and external links. Data is stored in
//! a SQLite file (typically `.work/issues.db`).

pub mod deps;
pub mod events;
pub mod issues;
pub mod labels;
pub mod links;
pub mod notes;
mod schema;

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

use crate::error::{Error, Result};
use schema::SCHEMA;

/// Parse a string value from the database, returning a rusqlite error on parse failure.
fn parse_db<T: std::str::FromStr>(
    value: &str,
    column: &str,
) -> std::result::Result<T, rusqlite::Error> {
    value.parse().map_err(|_| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(Error::CorruptedData(format!(
                "invalid value '{}' in column '{}'",
                value, column
            ))),
        )
    })
}

/// Parse an RFC3339 timestamp from the database.
fn parse_timestamp(
    value: &str,
    column: &str,
) -> std::result::Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(Error::CorruptedData(format!(
                    "invalid timestamp '{}' in column '{}'",
                    value, column
                ))),
            )
        })
}

/// SQLite database connection with issue tracker operations.
///
/// Database provides methods for managing issues, events, notes, tags, and dependencies.
/// Operations are split across submodules: [`issues`], [`events`], [`notes`], [`tags`], [`deps`].
pub struct Database {
    /// The underlying SQLite connection.
    pub conn: Connection,
}

impl Database {
    /// Open a database connection at the given path, creating and migrating if needed
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(path)?;

        // Configure SQLite for concurrent access:
        // - WAL mode allows multiple readers with a single writer
        // - busy_timeout prevents immediate SQLITE_BUSY errors
        // - foreign_keys ensures referential integrity
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;",
        )?;

        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing and benchmarks)
    ///
    /// Note: In-memory databases don't support WAL mode, so we only enable
    /// foreign keys and busy_timeout.
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;",
        )?;
        let db = Database { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Run database migrations
    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(SCHEMA)?;
        self.migrate_add_assignee()?;
        self.migrate_add_hlc_columns()?;
        Ok(())
    }

    /// Migration: Add assignee column to existing databases
    fn migrate_add_assignee(&self) -> Result<()> {
        // Check if assignee column exists
        let has_assignee: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = 'assignee'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_assignee {
            self.conn
                .execute("ALTER TABLE issues ADD COLUMN assignee TEXT", [])?;
        }
        Ok(())
    }

    /// Migration: Add HLC columns for CRDT sync compatibility.
    ///
    /// The core database (wk_core) uses HLC (Hybrid Logical Clock) columns for
    /// conflict resolution during sync. We add these columns to CLI's database
    /// so that both can share the same SQLite file.
    fn migrate_add_hlc_columns(&self) -> Result<()> {
        let columns = ["last_status_hlc", "last_title_hlc", "last_type_hlc"];

        for column in columns {
            let has_column: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = ?1",
                    [column],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if !has_column {
                let sql = format!("ALTER TABLE issues ADD COLUMN {} TEXT", column);
                self.conn.execute(&sql, [])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
