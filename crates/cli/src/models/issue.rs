// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};
use wk_core::IssueType;

/// Workflow status of an issue.
///
/// Status transitions follow a defined state machine with validation.
/// Use [`Status::can_transition_to`] to check valid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Not yet started. Initial state for new issues.
    Todo,
    /// Currently being worked on.
    InProgress,
    /// Successfully completed.
    Done,
    /// Closed without completion (won't fix, duplicate, etc.).
    Closed,
}

impl Status {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::InProgress => "in_progress",
            Status::Done => "done",
            Status::Closed => "closed",
        }
    }

    /// Check if a transition from this status to target is valid.
    ///
    /// All non-self transitions are valid (lenient transitions).
    pub fn can_transition_to(&self, target: Status) -> bool {
        *self != target
    }

    /// Get valid transition targets as a formatted string
    pub fn valid_targets(&self) -> String {
        match self {
            Status::Todo => "in_progress, done (with reason), closed (with reason)".to_string(),
            Status::InProgress => "todo, done, closed (with reason)".to_string(),
            Status::Done => "in_progress, todo (with reason), closed (with reason)".to_string(),
            Status::Closed => "in_progress, todo (with reason), done (with reason)".to_string(),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(Status::Todo),
            "in_progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            "closed" => Ok(Status::Closed),
            _ => Err(Error::InvalidStatus(s.to_string())),
        }
    }
}

impl From<Status> for wk_core::Status {
    fn from(status: Status) -> Self {
        match status {
            Status::Todo => wk_core::Status::Todo,
            Status::InProgress => wk_core::Status::InProgress,
            Status::Done => wk_core::Status::Done,
            Status::Closed => wk_core::Status::Closed,
        }
    }
}

/// The primary entity representing a tracked work item.
///
/// Issues are identified by a unique ID generated from a project prefix
/// and a hash of the title and creation time (e.g., "proj-a1b2").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier (format: `{prefix}-{hash}`).
    pub id: String,
    /// Classification of the issue.
    pub issue_type: IssueType,
    /// Short description of the work.
    pub title: String,
    /// Longer description providing context.
    pub description: Option<String>,
    /// Current workflow state.
    pub status: Status,
    /// Person or queue this issue is assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// When the issue was created.
    pub created_at: DateTime<Utc>,
    /// When the issue was last modified.
    pub updated_at: DateTime<Utc>,
    /// When the issue was closed (done or closed status). None if not closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
}

impl Issue {
    /// Test helper: construct an Issue with default status and current timestamp.
    /// Production code constructs Issues with explicit fields including timestamps.
    #[cfg(test)]
    pub fn new(id: String, issue_type: IssueType, title: String) -> Self {
        let now = Utc::now();
        Issue {
            id,
            issue_type,
            title,
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: now,
            updated_at: now,
            closed_at: None,
        }
    }
}

#[cfg(test)]
#[path = "issue_tests.rs"]
mod tests;
