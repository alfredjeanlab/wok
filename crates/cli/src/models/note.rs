// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Status;

/// A text note attached to an issue.
///
/// Notes capture context, decisions, and progress updates. Each note records
/// the issue's status at the time it was added, enabling status-grouped display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this note belongs to.
    pub issue_id: String,
    /// Issue status when the note was added.
    pub status: Status,
    /// The note text.
    pub content: String,
    /// When the note was created.
    pub created_at: DateTime<Utc>,
}

impl Note {
    /// Test helper: construct a Note with default id and current timestamp.
    /// Production code constructs Notes from DB rows which include stored ids/timestamps.
    #[cfg(test)]
    pub fn new(issue_id: String, status: Status, content: String) -> Self {
        Note {
            id: 0, // Will be set by database
            issue_id,
            status,
            content,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
#[path = "note_tests.rs"]
mod tests;
