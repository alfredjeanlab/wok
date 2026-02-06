// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::collections::HashMap;

use crate::error::Result;

use super::Database;

impl Database {
    /// Get labels for multiple issues in a single query.
    pub fn get_labels_batch(&self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
        Ok(self.0.get_labels_batch(issue_ids)?)
    }

    /// Add a label to an issue
    pub fn add_label(&self, issue_id: &str, label: &str) -> Result<()> {
        Ok(self.0.add_label(issue_id, label)?)
    }

    /// Remove a label from an issue
    pub fn remove_label(&self, issue_id: &str, label: &str) -> Result<bool> {
        Ok(self.0.remove_label(issue_id, label)?)
    }

    /// Get all labels for an issue
    pub fn get_labels(&self, issue_id: &str) -> Result<Vec<String>> {
        Ok(self.0.get_labels(issue_id)?)
    }
}

#[cfg(test)]
#[path = "labels_tests.rs"]
mod tests;
