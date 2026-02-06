// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Core issue types for the wk issue tracker.
//!
//! This module contains the fundamental data types: Issue, IssueType, Status,
//! Action, and Event.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::hlc::Hlc;

/// Classification of issues by their nature and scope.
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
    /// Early-stage thought or proposal, not yet refined into actionable work.
    Idea,
    /// Cross-cutting initiative spanning multiple features or projects.
    Epic,
}

impl IssueType {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Feature => "feature",
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Chore => "chore",
            IssueType::Idea => "idea",
            IssueType::Epic => "epic",
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
            "idea" => Ok(IssueType::Idea),
            "epic" => Ok(IssueType::Epic),
            _ => Err(Error::InvalidIssueType(s.to_string())),
        }
    }
}

/// Workflow status of an issue.
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

    /// Get valid transition targets as a formatted string.
    pub fn valid_targets(&self) -> String {
        match self {
            Status::Todo => "in_progress, done (with reason), closed (with reason)".to_string(),
            Status::InProgress => "todo, done, closed (with reason)".to_string(),
            Status::Done => "in_progress, todo (with reason), closed (with reason)".to_string(),
            Status::Closed => "in_progress, todo (with reason), done (with reason)".to_string(),
        }
    }

    /// Returns true if this is a terminal state (done or closed).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Status::Done | Status::Closed)
    }

    /// Returns true if this is an active state (not done/closed).
    pub fn is_active(&self) -> bool {
        !self.is_terminal()
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    /// Unique identifier (format: `{prefix}-{hash}`).
    pub id: String,
    /// Classification of the issue.
    pub issue_type: IssueType,
    /// Short description of the work.
    pub title: String,
    /// Longer description providing context.
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// HLC timestamp of last status change (for conflict resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_status_hlc: Option<Hlc>,
    /// HLC timestamp of last title change (for conflict resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_title_hlc: Option<Hlc>,
    /// HLC timestamp of last type change (for conflict resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_type_hlc: Option<Hlc>,
    /// HLC timestamp of last description change (for conflict resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_description_hlc: Option<Hlc>,
    /// HLC timestamp of last assignee change (for conflict resolution).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_assignee_hlc: Option<Hlc>,
}

impl Issue {
    /// Creates a new issue with default HLC fields as None.
    pub fn new(
        id: String,
        issue_type: IssueType,
        title: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Issue {
            id,
            issue_type,
            title,
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at,
            updated_at: created_at,
            last_status_hlc: None,
            last_title_hlc: None,
            last_type_hlc: None,
            last_description_hlc: None,
            last_assignee_hlc: None,
        }
    }
}

/// Types of actions that can be recorded in the event log.
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
            _ => Err(Error::InvalidAction(s.to_string())),
        }
    }
}

/// An audit log entry recording a change to an issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this event belongs to.
    pub issue_id: String,
    /// What type of change occurred.
    pub action: Action,
    /// Previous value (for edits, status changes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    /// New value (for edits, tags, related issues).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    /// User-provided explanation (for closes, reopens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// When the event occurred.
    pub created_at: DateTime<Utc>,
}

impl Event {
    /// Creates a new event with the current timestamp.
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

    /// Sets a specific timestamp for this event.
    pub fn with_timestamp(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = created_at;
        self
    }
}

/// Relation types for dependencies between issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// The from_id blocks to_id (to_id cannot proceed until from_id is done).
    Blocks,
    /// The from_id is tracked by to_id (organizational hierarchy).
    TrackedBy,
    /// The from_id tracks to_id (organizational hierarchy).
    Tracks,
}

impl Relation {
    /// Returns the string representation used in storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Relation::Blocks => "blocks",
            Relation::TrackedBy => "tracked-by",
            Relation::Tracks => "tracks",
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Relation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(Relation::Blocks),
            "tracked-by" | "tracked_by" => Ok(Relation::TrackedBy),
            "tracks" => Ok(Relation::Tracks),
            _ => Err(Error::InvalidRelation(s.to_string())),
        }
    }
}

/// A dependency relationship between two issues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The source issue ID.
    pub from_id: String,
    /// The target issue ID.
    pub to_id: String,
    /// The type of relationship.
    pub relation: Relation,
    /// When the dependency was created.
    pub created_at: DateTime<Utc>,
}

/// A note attached to an issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this note belongs to.
    pub issue_id: String,
    /// Status when note was added.
    pub status: Status,
    /// The note content.
    pub content: String,
    /// When the note was created.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
#[path = "issue_tests.rs"]
mod tests;
