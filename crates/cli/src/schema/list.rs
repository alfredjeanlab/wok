// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wok list` JSON output.

use schemars::JsonSchema;
use serde::Serialize;

use super::IssueJson;

/// JSON output structure for the list command.
#[derive(JsonSchema, Serialize)]
pub struct ListOutputJson {
    /// List of issues matching the query.
    pub issues: Vec<IssueJson>,
    /// Filter expressions that were applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<Vec<String>>,
    /// Maximum number of results requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}
