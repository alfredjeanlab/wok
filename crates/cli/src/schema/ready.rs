// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Schema types for `wok ready` JSON output.

use schemars::JsonSchema;
use serde::Serialize;

use super::IssueJson;

/// JSON output structure for the ready command.
///
/// The ready command returns an array of issue summaries directly.
#[derive(JsonSchema, Serialize)]
#[serde(transparent)]
pub struct ReadyOutputJson(pub Vec<IssueJson>);
