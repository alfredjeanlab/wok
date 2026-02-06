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
pub mod prefixes;
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
        self.migrate_add_closed_at()?;
        self.migrate_backfill_prefixes()?;
        self.migrate_tracked_by_relation()?;
        Ok(())
    }

    /// Migration: Rewrite "tracked_by" to "tracked-by" in deps table.
    ///
    /// Early versions serialized TrackedBy as "tracked_by" (underscore).
    /// The canonical form is "tracked-by" (kebab-case).
    fn migrate_tracked_by_relation(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE deps SET rel = 'tracked-by' WHERE rel = 'tracked_by'",
            [],
        )?;
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

    /// Migration: Add closed_at column and backfill from events.
    ///
    /// Stores the timestamp when an issue was closed (done/closed status) directly
    /// on the issues table, replacing the correlated subquery that computed it.
    fn migrate_add_closed_at(&self) -> Result<()> {
        let has_column: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM pragma_table_info('issues') WHERE name = 'closed_at'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_column {
            // Handle concurrent migration: another connection may add the column
            // between our check and this ALTER TABLE (shared user-level database).
            match self
                .conn
                .execute("ALTER TABLE issues ADD COLUMN closed_at TEXT", [])
            {
                Ok(_) => {
                    // Backfill: set closed_at from the most recent done/closed event
                    // that has no later reopened event (matching the old subquery logic)
                    self.conn.execute(
                        "UPDATE issues SET closed_at = (
                            SELECT MAX(e.created_at) FROM events e
                            WHERE e.issue_id = issues.id AND e.action IN ('done', 'closed')
                            AND NOT EXISTS (
                                SELECT 1 FROM events e2
                                WHERE e2.issue_id = e.issue_id
                                AND e2.action = 'reopened'
                                AND e2.created_at > e.created_at
                            )
                        ) WHERE status IN ('done', 'closed')",
                        [],
                    )?;
                }
                Err(e) if e.to_string().contains("duplicate column") => {
                    // Column was added by another connection concurrently
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    /// Migration: Backfill prefixes table from existing issues.
    ///
    /// Extracts prefixes from issue IDs and populates the prefixes table
    /// with correct issue counts. Only runs if the table is empty but
    /// issues exist.
    fn migrate_backfill_prefixes(&self) -> Result<()> {
        // Check if migration is needed (prefixes table empty but issues exist)
        let prefix_count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM prefixes", [], |row| row.get(0))
            .unwrap_or(0);

        if prefix_count == 0 {
            // Backfill from existing issues by extracting prefix from ID
            // Issue IDs follow pattern: {prefix}-{hash} where prefix is before first '-'
            self.conn.execute(
                "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count)
                 SELECT
                     substr(id, 1, instr(id, '-') - 1) as prefix,
                     MIN(created_at) as created_at,
                     COUNT(*) as issue_count
                 FROM issues
                 WHERE id LIKE '%-%'
                 GROUP BY prefix",
                [],
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
