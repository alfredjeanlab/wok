// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Types of actions that can be recorded in the event log.
///
/// Every change to an issue generates an event with one of these action types,
/// creating an audit trail of the issue's history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Issue was created.
    Created,
    /// Issue title or type was modified.
    Edited,
    /// Work began (status -> in_progress).
    Started,
    /// Work paused (status -> todo).
    Stopped,
    /// Issue was completed (status -> done).
    Done,
    /// Issue was closed without completion.
    Closed,
    /// Issue was reopened from done/closed.
    Reopened,
    /// A label was added.
    Labeled,
    /// A label was removed.
    Unlabeled,
    /// A dependency was added (internal issue relationship).
    Related,
    /// A dependency was removed (internal issue relationship).
    Unrelated,
    /// An external link was added.
    Linked,
    /// An external link was removed.
    Unlinked,
    /// A note was added.
    Noted,
    /// A blocking issue was resolved.
    Unblocked,
    /// Issue was assigned to someone.
    Assigned,
    /// Issue assignment was removed.
    Unassigned,
}

impl Action {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Created => "created",
            Action::Edited => "edited",
            Action::Started => "started",
            Action::Stopped => "stopped",
            Action::Done => "done",
            Action::Closed => "closed",
            Action::Reopened => "reopened",
            Action::Labeled => "labeled",
            Action::Unlabeled => "unlabeled",
            Action::Related => "related",
            Action::Unrelated => "unrelated",
            Action::Linked => "linked",
            Action::Unlinked => "unlinked",
            Action::Noted => "noted",
            Action::Unblocked => "unblocked",
            Action::Assigned => "assigned",
            Action::Unassigned => "unassigned",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "created" => Ok(Action::Created),
            "edited" => Ok(Action::Edited),
            "started" => Ok(Action::Started),
            "stopped" => Ok(Action::Stopped),
            "done" => Ok(Action::Done),
            "closed" => Ok(Action::Closed),
            "reopened" => Ok(Action::Reopened),
            "labeled" => Ok(Action::Labeled),
            "unlabeled" => Ok(Action::Unlabeled),
            "related" => Ok(Action::Related),
            "unrelated" => Ok(Action::Unrelated),
            "linked" => Ok(Action::Linked),
            "unlinked" => Ok(Action::Unlinked),
            "noted" => Ok(Action::Noted),
            "unblocked" => Ok(Action::Unblocked),
            "assigned" => Ok(Action::Assigned),
            "unassigned" => Ok(Action::Unassigned),
            _ => Err(Error::InvalidStatus(s.to_string())),
        }
    }
}

/// An audit log entry recording a change to an issue.
///
/// Events form an immutable history of all changes, enabling activity logs
/// and change tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this event belongs to.
    pub issue_id: String,
    /// What type of change occurred.
    pub action: Action,
    /// Previous value (for edits, status changes).
    pub old_value: Option<String>,
    /// New value (for edits, tags, linked issues).
    pub new_value: Option<String>,
    /// User-provided explanation (for closes, reopens).
    pub reason: Option<String>,
    /// When the event occurred.
    pub created_at: DateTime<Utc>,
}

impl Event {
    /// Creates a new event with the current timestamp.
    ///
    /// The `id` field is set to 0 and will be assigned by the database on insert.
    pub fn new(issue_id: String, action: Action) -> Self {
        Event {
            id: 0, // Will be set by database
            issue_id,
            action,
            old_value: None,
            new_value: None,
            reason: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the old and new values for this event (builder pattern).
    pub fn with_values(mut self, old: Option<String>, new: Option<String>) -> Self {
        self.old_value = old;
        self.new_value = new;
        self
    }

    /// Sets the reason for this event (builder pattern).
    pub fn with_reason(mut self, reason: Option<String>) -> Self {
        self.reason = reason;
        self
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
