// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::Result;
use crate::models::{Note, Status};

use super::Database;

impl Database {
    /// Add a note to an issue
    pub fn add_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        Ok(self.0.add_note(issue_id, status, content)?)
    }

    /// Get all notes for an issue, ordered by creation time
    pub fn get_notes(&self, issue_id: &str) -> Result<Vec<Note>> {
        Ok(self.0.get_notes(issue_id)?)
    }

    /// Replace the most recent note for an issue with new content
    pub fn replace_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        Ok(self.0.replace_note(issue_id, status, content)?)
    }

    /// Get notes grouped by status
    pub fn get_notes_by_status(&self, issue_id: &str) -> Result<Vec<(Status, Vec<Note>)>> {
        Ok(self.0.get_notes_by_status(issue_id)?)
    }
}

#[cfg(test)]
#[path = "notes_tests.rs"]
mod tests;
