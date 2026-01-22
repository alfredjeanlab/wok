// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::collections::HashMap;

use rusqlite::params;

use crate::error::Result;

use super::Database;

impl Database {
    /// Get labels for multiple issues in a single query.
    /// Returns a map from issue_id to labels vector.
    pub fn get_labels_batch(&self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        if issue_ids.is_empty() {
            return Ok(HashMap::new());
        }

        // Build query with positional placeholders
        let placeholders: Vec<_> = (1..=issue_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT issue_id, label FROM labels WHERE issue_id IN ({}) ORDER BY issue_id, label",
            placeholders.join(", ")
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = issue_ids
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        let mut rows = stmt.query(params.as_slice())?;
        while let Some(row) = rows.next()? {
            let issue_id: String = row.get(0)?;
            let label: String = row.get(1)?;
            map.entry(issue_id).or_default().push(label);
        }

        Ok(map)
    }
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
