// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database operations for prefix tracking.

use chrono::Utc;
use rusqlite::params;

use super::{parse_timestamp, Database};
use crate::error::Result;
use crate::models::PrefixInfo;

impl Database {
    /// Ensure a prefix exists in the prefixes table.
    ///
    /// Creates the prefix entry if it doesn't exist, using the current timestamp.
    pub fn ensure_prefix(&self, prefix: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count) VALUES (?1, ?2, 0)",
            params![prefix, now],
        )?;
        Ok(())
    }

    /// Increment the issue count for a prefix.
    ///
    /// Should be called after creating an issue with this prefix.
    pub fn increment_prefix_count(&self, prefix: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE prefixes SET issue_count = issue_count + 1 WHERE prefix = ?1",
            params![prefix],
        )?;
        Ok(())
    }

    /// List all prefixes with their issue counts.
    ///
    /// Results are ordered by issue count (descending).
    pub fn list_prefixes(&self) -> Result<Vec<PrefixInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT prefix, created_at, issue_count FROM prefixes ORDER BY issue_count DESC, prefix ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let prefix: String = row.get(0)?;
            let created_at_str: String = row.get(1)?;
            let created_at = parse_timestamp(&created_at_str, "created_at")?;
            let issue_count: i64 = row.get(2)?;
            Ok(PrefixInfo {
                prefix,
                created_at,
                issue_count,
            })
        })?;

        let mut prefixes = Vec::new();
        for row in rows {
            prefixes.push(row?);
        }
        Ok(prefixes)
    }

    /// Rename a prefix in the prefixes table.
    ///
    /// This transfers the created_at and issue_count to the new prefix name.
    /// If the new prefix already exists, merges the counts.
    pub fn rename_prefix(&self, old: &str, new: &str) -> Result<()> {
        // Get the old prefix's info
        let old_info: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT created_at, issue_count FROM prefixes WHERE prefix = ?1",
                params![old],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        if let Some((created_at, issue_count)) = old_info {
            // Check if new prefix already exists
            let new_exists: bool = self
                .conn
                .query_row(
                    "SELECT 1 FROM prefixes WHERE prefix = ?1",
                    params![new],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if new_exists {
                // Merge: add old count to new, keep earlier created_at
                self.conn.execute(
                    "UPDATE prefixes SET
                        issue_count = issue_count + ?1,
                        created_at = MIN(created_at, ?2)
                     WHERE prefix = ?3",
                    params![issue_count, created_at, new],
                )?;
            } else {
                // Insert new prefix with old info
                self.conn.execute(
                    "INSERT INTO prefixes (prefix, created_at, issue_count) VALUES (?1, ?2, ?3)",
                    params![new, created_at, issue_count],
                )?;
            }

            // Delete old prefix
            self.conn
                .execute("DELETE FROM prefixes WHERE prefix = ?1", params![old])?;
        }

        Ok(())
    }
}
