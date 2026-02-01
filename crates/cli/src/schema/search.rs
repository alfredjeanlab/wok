// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wok search` JSON output.

use schemars::JsonSchema;
use serde::Serialize;

use super::IssueJson;

/// JSON output structure for the search command.
///
/// The search command returns an array of issue summaries directly.
#[derive(JsonSchema, Serialize)]
#[serde(transparent)]
pub struct SearchOutputJson(pub Vec<IssueJson>);
