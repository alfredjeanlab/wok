// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wok show` JSON output.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;

use super::{Event, IssueType, Link, Note, Status};

/// Full issue details including notes, links, and events.
#[derive(JsonSchema, Serialize)]
pub struct IssueDetails {
    /// Unique issue identifier.
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
    /// When the issue was closed (done or closed status).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    /// Labels attached to the issue.
    pub labels: Vec<String>,
    /// Issue IDs that block this issue.
    pub blockers: Vec<String>,
    /// Issue IDs that this issue blocks.
    pub blocking: Vec<String>,
    /// Parent issue IDs (tracking this issue).
    pub parents: Vec<String>,
    /// Child issue IDs (tracked by this issue).
    pub children: Vec<String>,
    /// Notes attached to the issue.
    pub notes: Vec<Note>,
    /// External links attached to the issue.
    pub links: Vec<Link>,
    /// Event history for the issue.
    pub events: Vec<Event>,
}
