// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Database access for the CLI.
//!
//! In private mode, the CLI opens the database directly using [`wk_core::Database`].
//! CLI-only extensions (notes grouped by status, note replacement) are provided
//! via the [`DatabaseExt`] trait. Standalone functions provide priority parsing
//! and link construction.

pub use wk_core::Database;

use chrono::Utc;
use rusqlite::params;

use crate::error::Result;
use crate::models::{Link, Note, Status};

/// CLI-specific database extensions not provided by core.
pub trait DatabaseExt {
    /// Get notes grouped by status (preserving insertion order of status groups).
    fn get_notes_by_status(&self, issue_id: &str) -> Result<Vec<(Status, Vec<Note>)>>;

    /// Replace the most recent note for an issue with new content.
    fn replace_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64>;
}

impl DatabaseExt for Database {
    fn get_notes_by_status(&self, issue_id: &str) -> Result<Vec<(Status, Vec<Note>)>> {
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

    fn replace_note(&self, issue_id: &str, status: Status, content: &str) -> Result<i64> {
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
            None => Err(crate::error::Error::NoNotesToReplace {
                issue_id: issue_id.to_string(),
            }),
        }
    }
}

/// Extract priority from label list.
///
/// Prefers "priority:" over "p:" if both present.
/// Returns 0-4 where 0 is highest priority.
/// Default (no priority label): 2 (medium)
pub fn priority_from_tags(tags: &[String]) -> u8 {
    for tag in tags {
        if let Some(value) = tag.strip_prefix("priority:") {
            if let Some(p) = parse_priority_value(value) {
                return p;
            }
        }
    }
    for tag in tags {
        if let Some(value) = tag.strip_prefix("p:") {
            if let Some(p) = parse_priority_value(value) {
                return p;
            }
        }
    }
    2
}

/// Parse priority value (numeric 0-4 or named).
fn parse_priority_value(value: &str) -> Option<u8> {
    match value {
        "0" | "highest" => Some(0),
        "1" | "high" => Some(1),
        "2" | "medium" | "med" => Some(2),
        "3" | "low" => Some(3),
        "4" | "lowest" => Some(4),
        _ => None,
    }
}

/// Create a new Link with default values and the current timestamp.
pub fn new_link(issue_id: &str) -> Link {
    Link {
        id: 0,
        issue_id: issue_id.to_string(),
        link_type: None,
        url: None,
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
