// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Classification of issues by their nature and scope.
///
/// Issue types help organize work and provide visual distinction in lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// Large feature or initiative containing multiple tasks.
    Feature,
    /// Standard unit of work.
    Task,
    /// Defect or problem to fix.
    Bug,
    /// Maintenance work (refactoring, cleanup, dependency updates).
    Chore,
}

impl IssueType {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Feature => "feature",
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Chore => "chore",
        }
    }
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for IssueType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Ok(IssueType::Feature),
            "task" => Ok(IssueType::Task),
            "bug" => Ok(IssueType::Bug),
            "chore" => Ok(IssueType::Chore),
            _ => Err(Error::InvalidIssueType(s.to_string())),
        }
    }
}

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
    pub fn can_transition_to(&self, target: Status) -> bool {
        matches!(
            (self, target),
            (Status::Todo, Status::InProgress)
                | (Status::Todo, Status::Done)
                | (Status::Todo, Status::Closed)
                | (Status::InProgress, Status::Todo)
                | (Status::InProgress, Status::Done)
                | (Status::InProgress, Status::Closed)
                | (Status::Done, Status::Todo)
                | (Status::Closed, Status::Todo)
        )
    }

    /// Get valid transition targets as a formatted string
    pub fn valid_targets(&self) -> String {
        match self {
            Status::Todo => "in_progress, done (with reason), closed (with reason)".to_string(),
            Status::InProgress => "todo, done, closed (with reason)".to_string(),
            Status::Done => "todo (with reason to reopen)".to_string(),
            Status::Closed => "todo (with reason to reopen)".to_string(),
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
