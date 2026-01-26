// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Prefix tracking for multi-project support.

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Information about a prefix in the issue tracker.
#[derive(Debug, Clone, Serialize)]
pub struct PrefixInfo {
    /// The prefix string (e.g., "proj", "api")
    pub prefix: String,
    /// When this prefix was first used
    pub created_at: DateTime<Utc>,
    /// Number of issues with this prefix
    pub issue_count: i64,
}
