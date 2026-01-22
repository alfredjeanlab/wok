// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wk list` JSON output.

use schemars::JsonSchema;
use serde::Serialize;

use super::{IssueType, Status};

/// JSON representation of an issue in list output.
#[derive(JsonSchema, Serialize)]
pub struct ListIssueJson {
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

/// JSON output structure for the list command.
#[derive(JsonSchema, Serialize)]
pub struct ListOutputJson {
    /// List of issues matching the query.
    pub issues: Vec<ListIssueJson>,
    /// Filter expressions that were applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<Vec<String>>,
    /// Maximum number of results requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}
