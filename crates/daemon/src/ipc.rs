// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! IPC protocol for daemon-CLI communication.
//!
//! This module mirrors the protocol defined in the CLI crate's daemon/ipc.rs.
//! Messages are serialized as JSON with length-prefixed framing.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request sent from CLI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DaemonRequest {
    /// Get daemon status.
    Status,
    /// Graceful shutdown.
    Shutdown,
    /// Ping to check if daemon is alive.
    Ping,
    /// Version handshake request.
    Hello { version: String },
    /// Database query operation.
    Query(QueryOp),
    /// Database mutation operation.
    Mutate(MutateOp),
}

/// Query operations for reading from the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op")]
pub enum QueryOp {
    /// Resolve a partial ID to a full ID.
    ResolveId { partial_id: String },
    /// Check if an issue exists.
    IssueExists { id: String },
    /// Get a single issue by ID.
    GetIssue { id: String },
    /// List issues with optional filters.
    ListIssues {
        status: Option<Status>,
        issue_type: Option<IssueType>,
        label: Option<String>,
    },
    /// Search issues by query string.
    SearchIssues { query: String },
    /// Get IDs of blocked issues.
    GetBlockedIssueIds,
    /// Get labels for an issue.
    GetLabels { id: String },
    /// Get labels for multiple issues.
    GetLabelsBatch { ids: Vec<String> },
    /// Get notes for an issue.
    GetNotes { id: String },
    /// Get events for an issue.
    GetEvents { id: String },
    /// Get all events with optional limit.
    GetAllEvents { limit: Option<usize> },
    /// Get dependencies from an issue.
    GetDepsFrom { id: String },
    /// Get blockers for an issue.
    GetBlockers { id: String },
    /// Get issues blocked by an issue.
    GetBlocking { id: String },
    /// Get tracked issues.
    GetTracked { id: String },
    /// Get tracking issues.
    GetTracking { id: String },
    /// Get transitive blockers.
    GetTransitiveBlockers { id: String },
    /// Get links for an issue.
    GetLinks { id: String },
    /// Get a specific link by URL.
    GetLinkByUrl { id: String, url: String },
    /// List all prefixes.
    ListPrefixes,
}

/// Mutation operations for writing to the database.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op")]
pub enum MutateOp {
    /// Create a new issue.
    CreateIssue { issue: Issue },
    /// Update issue status.
    UpdateIssueStatus { id: String, status: Status },
    /// Update issue title.
    UpdateIssueTitle { id: String, title: String },
    /// Update issue description.
    UpdateIssueDescription { id: String, description: String },
    /// Update issue type.
    UpdateIssueType { id: String, issue_type: IssueType },
    /// Set issue assignee.
    SetAssignee { id: String, assignee: String },
    /// Clear issue assignee.
    ClearAssignee { id: String },
    /// Add a label to an issue.
    AddLabel { id: String, label: String },
    /// Remove a label from an issue.
    RemoveLabel { id: String, label: String },
    /// Add a note to an issue.
    AddNote {
        id: String,
        status: Status,
        content: String,
    },
    /// Log an event.
    LogEvent { event: Event },
    /// Add a dependency.
    AddDependency {
        from_id: String,
        to_id: String,
        relation: Relation,
    },
    /// Remove a dependency.
    RemoveDependency {
        from_id: String,
        to_id: String,
        relation: Relation,
    },
    /// Add a link to an issue.
    AddLink {
        id: String,
        link_type: Option<LinkType>,
        url: Option<String>,
        external_id: Option<String>,
        rel: Option<LinkRel>,
    },
    /// Remove a link from an issue.
    RemoveLink { id: String, url: String },
    /// Ensure a prefix exists.
    EnsurePrefix { prefix: String },
    /// Increment prefix issue count.
    IncrementPrefixCount { prefix: String },
}

/// Response sent from daemon to CLI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum DaemonResponse {
    /// Status response.
    Status(DaemonStatus),
    /// Shutdown acknowledged.
    ShuttingDown,
    /// Pong response.
    Pong,
    /// Error response.
    Error { message: String },
    /// Version handshake response.
    Hello { version: String },
    /// Query result.
    QueryResult(QueryResult),
    /// Mutation acknowledgment.
    MutateResult(MutateResult),
}

/// Results from query operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "result")]
pub enum QueryResult {
    /// Resolved ID.
    ResolvedId { id: String },
    /// Boolean result (e.g., issue_exists).
    Bool { value: bool },
    /// Single issue.
    Issue { issue: Issue },
    /// List of issues.
    Issues { issues: Vec<Issue> },
    /// List of string IDs.
    Ids { ids: Vec<String> },
    /// List of labels.
    Labels { labels: Vec<String> },
    /// Labels for multiple issues.
    LabelsBatch {
        labels: HashMap<String, Vec<String>>,
    },
    /// List of notes.
    Notes { notes: Vec<Note> },
    /// List of events.
    Events { events: Vec<Event> },
    /// List of dependencies.
    Dependencies { deps: Vec<Dependency> },
    /// List of links.
    Links { links: Vec<Link> },
    /// Optional link.
    Link { link: Option<Link> },
    /// List of prefix info.
    Prefixes { prefixes: Vec<PrefixInfo> },
}

/// Results from mutation operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "result")]
pub enum MutateResult {
    /// Mutation succeeded.
    Ok,
    /// Mutation succeeded, label was removed (returns true if it existed).
    LabelRemoved { removed: bool },
}

/// Daemon status information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonStatus {
    /// Current daemon PID.
    pub pid: u32,
    /// Uptime in seconds.
    pub uptime_secs: u64,
}

impl DaemonStatus {
    /// Create a new status with the given parameters.
    pub fn new(pid: u32, uptime_secs: u64) -> Self {
        Self { pid, uptime_secs }
    }
}

// ============================================================================
// Model types (mirrored from CLI for IPC serialization)
// ============================================================================

/// Issue type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Task,
    Bug,
    Epic,
}

impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Epic => "epic",
        }
    }
}

/// Issue status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Closed,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::InProgress => "in_progress",
            Status::Done => "done",
            Status::Closed => "closed",
        }
    }
}

/// An issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    pub id: String,
    pub issue_type: IssueType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
}

/// Event action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Created,
    Edited,
    Started,
    Stopped,
    Done,
    Closed,
    Reopened,
    Labeled,
    Unlabeled,
    Related,
    Unrelated,
    Linked,
    Unlinked,
    Noted,
    Unblocked,
    Assigned,
    Unassigned,
}

impl Action {
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

/// An audit log event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: i64,
    pub issue_id: String,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A note attached to an issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
    pub id: i64,
    pub issue_id: String,
    pub status: Status,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Dependency relationship type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    Blocks,
    TrackedBy,
    Tracks,
}

impl Relation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Relation::Blocks => "blocks",
            Relation::TrackedBy => "tracked-by",
            Relation::Tracks => "tracks",
        }
    }
}

/// A dependency between issues.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    pub from_id: String,
    pub to_id: String,
    pub relation: Relation,
    pub created_at: DateTime<Utc>,
}

/// External link type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Github,
    Jira,
    Gitlab,
    Confluence,
}

/// External link relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    Import,
    Blocks,
    Tracks,
    TrackedBy,
}

/// An external link.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub id: i64,
    pub issue_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<LinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<LinkRel>,
    pub created_at: DateTime<Utc>,
}

/// Prefix information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefixInfo {
    pub prefix: String,
    pub issue_count: i64,
    pub created_at: DateTime<Utc>,
}

/// IPC message framing.
///
/// Messages are framed as:
/// - 4 bytes: message length (big-endian u32)
/// - N bytes: JSON-encoded message
pub mod framing {
    use std::io::{Read, Write};

    use super::*;

    /// Maximum message size (1MB) to prevent malformed responses from causing hangs.
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Read a request from the given reader.
    pub fn read_request<R: Read>(reader: &mut R) -> std::io::Result<DaemonRequest> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(std::io::Error::other(format!(
                "message too large: {} bytes (max {})",
                len, MAX_MESSAGE_SIZE
            )));
        }

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        serde_json::from_slice(&buf)
            .map_err(|e| std::io::Error::other(format!("deserialize error: {}", e)))
    }

    /// Write a response to the given writer.
    pub fn write_response<W: Write>(
        writer: &mut W,
        response: &DaemonResponse,
    ) -> std::io::Result<()> {
        let json = serde_json::to_vec(response)
            .map_err(|e| std::io::Error::other(format!("serialize error: {}", e)))?;
        let len =
            u32::try_from(json.len()).map_err(|_| std::io::Error::other("message too large"))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&json)?;
        writer.flush()?;
        Ok(())
    }
}
