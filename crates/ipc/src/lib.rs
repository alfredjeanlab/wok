// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared IPC protocol for CLI-daemon communication.
//!
//! This crate defines the message types and framing protocol used between
//! the `wok` CLI and the `wokd` daemon. Messages are serialized as JSON
//! with length-prefixed framing.
//!
//! Model types (Status, Action, Relation, etc.) are re-exported from `wk_core`
//! as the canonical definitions. Only `Issue` is defined locally, since the IPC
//! representation omits HLC fields and includes a computed `closed_at`.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export all shared model types from wk_core.
pub use wk_core::{
    Action, Dependency, Event, IssueType, Link, LinkRel, LinkType, Note, PrefixInfo, Relation,
    Status,
};

/// The primary entity representing a tracked work item (IPC representation).
///
/// This differs from `wk_core::Issue` by omitting HLC conflict-resolution fields
/// and including a computed `closed_at` timestamp derived from the event log.
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
    /// When the issue was closed (done or closed status). None if not closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
}

impl From<wk_core::Issue> for Issue {
    fn from(core: wk_core::Issue) -> Self {
        Issue {
            id: core.id,
            issue_type: core.issue_type,
            title: core.title,
            description: core.description,
            status: core.status,
            assignee: core.assignee,
            created_at: core.created_at,
            updated_at: core.updated_at,
            closed_at: core.closed_at,
        }
    }
}

impl From<Issue> for wk_core::Issue {
    fn from(ipc: Issue) -> Self {
        wk_core::Issue {
            id: ipc.id,
            issue_type: ipc.issue_type,
            title: ipc.title,
            description: ipc.description,
            status: ipc.status,
            assignee: ipc.assignee,
            created_at: ipc.created_at,
            updated_at: ipc.updated_at,
            closed_at: ipc.closed_at,
            last_status_hlc: None,
            last_title_hlc: None,
            last_type_hlc: None,
            last_description_hlc: None,
            last_assignee_hlc: None,
        }
    }
}

impl Issue {
    /// Construct an Issue with default status (Todo) and current timestamp.
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

/// Identifies a dependency between two issues with its relationship type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DependencyRef {
    pub from_id: String,
    pub to_id: String,
    pub relation: Relation,
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
    AddDependency(DependencyRef),
    /// Remove a dependency.
    RemoveDependency(DependencyRef),
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

/// IPC message framing.
///
/// Messages are framed as:
/// - 4 bytes: message length (big-endian u32)
/// - N bytes: JSON-encoded message
pub mod framing {
    use std::io::{Read, Write};

    use serde::de::DeserializeOwned;
    use serde::Serialize;

    /// Maximum message size (1MB) to prevent malformed messages from causing hangs.
    const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

    /// Write a serializable message to the given writer.
    pub fn write_message<W: Write, T: Serialize>(
        writer: &mut W,
        message: &T,
    ) -> std::io::Result<()> {
        let json = serde_json::to_vec(message)
            .map_err(|e| std::io::Error::other(format!("serialize error: {}", e)))?;
        let len =
            u32::try_from(json.len()).map_err(|_| std::io::Error::other("message too large"))?;
        writer.write_all(&len.to_be_bytes())?;
        writer.write_all(&json)?;
        writer.flush()?;
        Ok(())
    }

    /// Read a deserializable message from the given reader.
    pub fn read_message<R: Read, T: DeserializeOwned>(reader: &mut R) -> std::io::Result<T> {
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
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
