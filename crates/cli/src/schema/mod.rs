// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for JSON output structures.
//!
//! These are separate from runtime types to allow schema-specific annotations
//! and to avoid adding schemars dependency to production output paths.
//!
//! Note: Types in this module are not constructed directly - they exist purely
//! for deriving JSON Schema definitions via schemars.

// Allow unused variants - these types exist only for schema generation
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;

pub mod list;
pub mod ready;
pub mod search;
pub mod show;

/// Issue type classification.
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Feature,
    Task,
    Bug,
    Chore,
    Idea,
}

/// Workflow status of an issue.
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Closed,
}

/// Types of actions that can be recorded in the event log.
#[derive(JsonSchema, Serialize)]
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

/// Type of external link (auto-detected from URL).
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Github,
    Jira,
    Gitlab,
    Confluence,
}

/// Relationship of external link to issue.
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    Import,
    Blocks,
    Tracks,
    TrackedBy,
}

/// A text note attached to an issue.
#[derive(JsonSchema, Serialize)]
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

/// An external link attached to an issue.
#[derive(JsonSchema, Serialize)]
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

/// An audit log entry recording a change to an issue.
#[derive(JsonSchema, Serialize)]
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
