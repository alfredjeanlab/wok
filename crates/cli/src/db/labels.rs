// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use rusqlite::params;

use crate::error::Result;

use super::Database;

impl Database {
    /// Add a label to an issue
    pub fn add_label(&self, issue_id: &str, label: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO labels (issue_id, label) VALUES (?1, ?2)",
            params![issue_id, label],
        )?;
        Ok(())
    }

    /// Remove a label from an issue
    pub fn remove_label(&self, issue_id: &str, label: &str) -> Result<bool> {
        let affected = self.conn.execute(
            "DELETE FROM labels WHERE issue_id = ?1 AND label = ?2",
            params![issue_id, label],
        )?;
        Ok(affected > 0)
    }

    /// Get all labels for an issue
    pub fn get_labels(&self, issue_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT label FROM labels WHERE issue_id = ?1 ORDER BY label")?;

        let labels = stmt
            .query_map(params![issue_id], |row| row.get(0))?
            .collect::<std::result::Result<Vec<String>, _>>()?;

        Ok(labels)
    }
}

#[cfg(test)]
#[path = "labels_tests.rs"]
mod tests;
