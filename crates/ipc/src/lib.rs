// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Shared IPC protocol for CLI-daemon communication.
//!
//! This crate defines the message types and framing protocol used between
//! the `wok` CLI and the `wokd` daemon. Messages are serialized as JSON
//! with length-prefixed framing.

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Error returned by `FromStr` impls for IPC model types.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Invalid status string.
    InvalidStatus(String),
    /// Invalid action string.
    InvalidAction(String),
    /// Invalid relation string.
    InvalidRelation(String),
    /// Invalid link type string.
    InvalidLinkType(String),
    /// Invalid link relation string.
    InvalidLinkRel(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidStatus(s) => write!(f, "invalid status: '{}'", s),
            ParseError::InvalidAction(s) => write!(f, "invalid action: '{}'", s),
            ParseError::InvalidRelation(s) => write!(f, "invalid relation: '{}'", s),
            ParseError::InvalidLinkType(s) => write!(f, "invalid link type: '{}'", s),
            ParseError::InvalidLinkRel(s) => write!(f, "invalid link relation: '{}'", s),
        }
    }
}

impl std::error::Error for ParseError {}

// Re-export IssueType from core (canonical definition).
pub use wk_core::IssueType;

// ============================================================================
// Model types for IPC serialization
// ============================================================================

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
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Status {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(Status::Todo),
            "in_progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            "closed" => Ok(Status::Closed),
            _ => Err(ParseError::InvalidStatus(s.to_string())),
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

impl From<wk_core::Status> for Status {
    fn from(status: wk_core::Status) -> Self {
        match status {
            wk_core::Status::Todo => Status::Todo,
            wk_core::Status::InProgress => Status::InProgress,
            wk_core::Status::Done => Status::Done,
            wk_core::Status::Closed => Status::Closed,
        }
    }
}

impl From<LinkType> for wk_core::LinkType {
    fn from(lt: LinkType) -> Self {
        match lt {
            LinkType::Github => wk_core::LinkType::Github,
            LinkType::Jira => wk_core::LinkType::Jira,
            LinkType::Gitlab => wk_core::LinkType::Gitlab,
            LinkType::Confluence => wk_core::LinkType::Confluence,
        }
    }
}

impl From<wk_core::LinkType> for LinkType {
    fn from(lt: wk_core::LinkType) -> Self {
        match lt {
            wk_core::LinkType::Github => LinkType::Github,
            wk_core::LinkType::Jira => LinkType::Jira,
            wk_core::LinkType::Gitlab => LinkType::Gitlab,
            wk_core::LinkType::Confluence => LinkType::Confluence,
        }
    }
}

impl From<LinkRel> for wk_core::LinkRel {
    fn from(lr: LinkRel) -> Self {
        match lr {
            LinkRel::Import => wk_core::LinkRel::Import,
            LinkRel::Blocks => wk_core::LinkRel::Blocks,
            LinkRel::Tracks => wk_core::LinkRel::Tracks,
            LinkRel::TrackedBy => wk_core::LinkRel::TrackedBy,
        }
    }
}

impl From<wk_core::LinkRel> for LinkRel {
    fn from(lr: wk_core::LinkRel) -> Self {
        match lr {
            wk_core::LinkRel::Import => LinkRel::Import,
            wk_core::LinkRel::Blocks => LinkRel::Blocks,
            wk_core::LinkRel::Tracks => LinkRel::Tracks,
            wk_core::LinkRel::TrackedBy => LinkRel::TrackedBy,
        }
    }
}

impl From<Link> for wk_core::Link {
    fn from(link: Link) -> Self {
        wk_core::Link {
            id: link.id,
            issue_id: link.issue_id,
            link_type: link.link_type.map(Into::into),
            url: link.url,
            external_id: link.external_id,
            rel: link.rel.map(Into::into),
            created_at: link.created_at,
        }
    }
}

impl From<wk_core::Link> for Link {
    fn from(link: wk_core::Link) -> Self {
        Link {
            id: link.id,
            issue_id: link.issue_id,
            link_type: link.link_type.map(Into::into),
            url: link.url,
            external_id: link.external_id,
            rel: link.rel.map(Into::into),
            created_at: link.created_at,
        }
    }
}

impl From<PrefixInfo> for wk_core::PrefixInfo {
    fn from(pi: PrefixInfo) -> Self {
        wk_core::PrefixInfo {
            prefix: pi.prefix,
            issue_count: pi.issue_count,
            created_at: pi.created_at,
        }
    }
}

impl From<wk_core::PrefixInfo> for PrefixInfo {
    fn from(pi: wk_core::PrefixInfo) -> Self {
        PrefixInfo {
            prefix: pi.prefix,
            issue_count: pi.issue_count,
            created_at: pi.created_at,
        }
    }
}

impl From<Action> for wk_core::Action {
    fn from(action: Action) -> Self {
        match action {
            Action::Created => wk_core::Action::Created,
            Action::Edited => wk_core::Action::Edited,
            Action::Started => wk_core::Action::Started,
            Action::Stopped => wk_core::Action::Stopped,
            Action::Done => wk_core::Action::Done,
            Action::Closed => wk_core::Action::Closed,
            Action::Reopened => wk_core::Action::Reopened,
            Action::Labeled => wk_core::Action::Labeled,
            Action::Unlabeled => wk_core::Action::Unlabeled,
            Action::Related => wk_core::Action::Related,
            Action::Unrelated => wk_core::Action::Unrelated,
            Action::Linked => wk_core::Action::Linked,
            Action::Unlinked => wk_core::Action::Unlinked,
            Action::Noted => wk_core::Action::Noted,
            Action::Unblocked => wk_core::Action::Unblocked,
            Action::Assigned => wk_core::Action::Assigned,
            Action::Unassigned => wk_core::Action::Unassigned,
        }
    }
}

impl From<wk_core::Action> for Action {
    fn from(action: wk_core::Action) -> Self {
        match action {
            wk_core::Action::Created => Action::Created,
            wk_core::Action::Edited => Action::Edited,
            wk_core::Action::Started => Action::Started,
            wk_core::Action::Stopped => Action::Stopped,
            wk_core::Action::Done => Action::Done,
            wk_core::Action::Closed => Action::Closed,
            wk_core::Action::Reopened => Action::Reopened,
            wk_core::Action::Labeled => Action::Labeled,
            wk_core::Action::Unlabeled => Action::Unlabeled,
            wk_core::Action::Related => Action::Related,
            wk_core::Action::Unrelated => Action::Unrelated,
            wk_core::Action::Linked => Action::Linked,
            wk_core::Action::Unlinked => Action::Unlinked,
            wk_core::Action::Noted => Action::Noted,
            wk_core::Action::Unblocked => Action::Unblocked,
            wk_core::Action::Assigned => Action::Assigned,
            wk_core::Action::Unassigned => Action::Unassigned,
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
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, ParseError> {
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
            _ => Err(ParseError::InvalidAction(s.to_string())),
        }
    }
}

/// Internal relationship types between issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// A blocks B = B should wait for A.
    Blocks,
    /// A is tracked by B (B contains A).
    TrackedBy,
    /// A tracks B (A contains B).
    Tracks,
}

impl Relation {
    /// Returns the string representation used in storage and display.
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
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            "blocks" => Ok(Relation::Blocks),
            "tracked-by" => Ok(Relation::TrackedBy),
            "tracks" => Ok(Relation::Tracks),
            _ => Err(ParseError::InvalidRelation(s.to_string())),
        }
    }
}

/// Type of external link (auto-detected from URL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Github,
    Jira,
    Gitlab,
    Confluence,
}

impl LinkType {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkType::Github => "github",
            LinkType::Jira => "jira",
            LinkType::Gitlab => "gitlab",
            LinkType::Confluence => "confluence",
        }
    }
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LinkType {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            "github" => Ok(LinkType::Github),
            "jira" => Ok(LinkType::Jira),
            "gitlab" => Ok(LinkType::Gitlab),
            "confluence" => Ok(LinkType::Confluence),
            _ => Err(ParseError::InvalidLinkType(s.to_string())),
        }
    }
}

/// Relationship of external link to issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    /// Issue was imported from this external source.
    Import,
    /// External issue blocks this issue.
    Blocks,
    /// This issue tracks the external issue.
    Tracks,
    /// This issue is tracked by the external issue.
    TrackedBy,
}

impl LinkRel {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkRel::Import => "import",
            LinkRel::Blocks => "blocks",
            LinkRel::Tracks => "tracks",
            LinkRel::TrackedBy => "tracked-by",
        }
    }
}

impl fmt::Display for LinkRel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LinkRel {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, ParseError> {
        match s.to_lowercase().as_str() {
            "import" => Ok(LinkRel::Import),
            "blocks" => Ok(LinkRel::Blocks),
            "tracks" => Ok(LinkRel::Tracks),
            "tracked-by" => Ok(LinkRel::TrackedBy),
            _ => Err(ParseError::InvalidLinkRel(s.to_string())),
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
    /// When the issue was closed (done or closed status). None if not closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
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
    /// New value (for edits, tags, linked issues).
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
    ///
    /// The `id` field is set to 0 and will be assigned by the database on insert.
    pub fn new(issue_id: String, action: Action) -> Self {
        Event {
            id: 0,
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

/// A text note attached to an issue.
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

/// A relationship between two issues.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The source issue of the relationship.
    pub from_id: String,
    /// The target issue of the relationship.
    pub to_id: String,
    /// The type of relationship.
    pub relation: Relation,
    /// When the dependency was created.
    pub created_at: DateTime<Utc>,
}

/// An external link attached to an issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this link belongs to.
    pub issue_id: String,
    /// Type of external link (auto-detected from URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<LinkType>,
    /// Full URL (may be None for shorthand like jira://PE-5555).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// External issue ID (e.g., "PE-5555" for Jira, "123" for GitHub).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Relationship to the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<LinkRel>,
    /// When the link was created.
    pub created_at: DateTime<Utc>,
}

/// Information about a prefix in the issue tracker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefixInfo {
    /// The prefix string (e.g., "proj", "api").
    pub prefix: String,
    /// Number of issues with this prefix.
    pub issue_count: i64,
    /// When this prefix was first used.
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Protocol types
// ============================================================================

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
// Message framing
// ============================================================================

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
