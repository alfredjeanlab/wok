// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::Utc;
use rusqlite::params;

use crate::error::Result;
use crate::models::{Note, Status};

use super::{parse_db, parse_timestamp, Database};

impl Database {
    /// Add a note to an issue
    pub fn add_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO notes (issue_id, status, content, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![issue_id, status.as_str(), content, Utc::now().to_rfc3339()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get all notes for an issue, ordered by creation time
    pub fn get_notes(&self, issue_id: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, issue_id, status, content, created_at
             FROM notes WHERE issue_id = ?1 ORDER BY created_at",
        )?;

        let notes = stmt
            .query_map(params![issue_id], |row| {
                let status_str: String = row.get(2)?;
                let created_str: String = row.get(4)?;
                Ok(Note {
                    id: row.get(0)?,
                    issue_id: row.get(1)?,
                    status: parse_db(&status_str, "status")?,
                    content: row.get(3)?,
                    created_at: parse_timestamp(&created_str, "created_at")?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    /// Replace the most recent note for an issue with new content
    pub fn replace_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
        // Find the most recent note for this issue
        let note_id: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM notes WHERE issue_id = ?1 ORDER BY created_at DESC LIMIT 1",
                params![issue_id],
                |row| row.get(0),
            )
            .ok();

        match note_id {
            Some(id) => {
                self.conn.execute(
                    "UPDATE notes SET content = ?1, status = ?2, created_at = ?3 WHERE id = ?4",
                    params![content, status.as_str(), Utc::now().to_rfc3339(), id],
                )?;
                Ok(id)
            }
            None => Err(crate::error::Error::InvalidInput(format!(
                "no notes to replace for issue {}",
                issue_id
            ))),
        }
    }

    /// Get notes grouped by status
    pub fn get_notes_by_status(&self, issue_id: &str) -> Result<Vec<(Status, Vec<Note>)>> {
        let notes = self.get_notes(issue_id)?;

        let mut grouped: Vec<(Status, Vec<Note>)> = Vec::new();

        for note in notes {
            if let Some((_, notes_vec)) = grouped.iter_mut().find(|(s, _)| *s == note.status) {
                notes_vec.push(note);
            } else {
                grouped.push((note.status, vec![note]));
            }
        }

        Ok(grouped)
    }
}

#[cfg(test)]
#[path = "notes_tests.rs"]
mod tests;
