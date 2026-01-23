// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wk ready` JSON output.

use schemars::JsonSchema;
use serde::Serialize;

use super::IssueJson;

/// JSON output structure for the ready command.
#[derive(JsonSchema, Serialize)]
pub struct ReadyOutputJson {
    /// List of ready (unblocked todo) issues.
    pub issues: Vec<IssueJson>,
}
