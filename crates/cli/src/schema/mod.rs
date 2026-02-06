// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for JSON output structures.
//!
//! Domain model types with `JsonSchema` derives come from `wk_core` (enabled
//! via the `schemars` feature flag). [`IssueJson`] is the unified issue
//! summary type used by list, ready, and search commands.

use schemars::JsonSchema;
use serde::Serialize;

// Re-export core types that carry JsonSchema derives (via `schemars` feature).
pub use wk_core::{Event, IssueType, Link, Note, Status};

pub mod list;
pub mod ready;
pub mod search;
pub mod show;

/// JSON representation of an issue summary.
/// Used by list, ready, and search command outputs.
#[derive(JsonSchema, Serialize)]
pub struct IssueJson {
    /// Unique issue identifier.
    pub id: String,
    /// Classification of the issue.
    pub issue_type: IssueType,
    /// Current workflow state.
    pub status: Status,
    /// Short description of the work.
    pub title: String,
    /// Person or queue this issue is assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Labels attached to the issue.
    pub labels: Vec<String>,
}

impl IssueJson {
    /// Create a new IssueJson from runtime issue data.
    pub fn new(
        id: String,
        issue_type: IssueType,
        status: Status,
        title: String,
        assignee: Option<String>,
        labels: Vec<String>,
    ) -> Self {
        IssueJson {
            id,
            issue_type,
            status,
            title,
            assignee,
            labels,
        }
    }
}
