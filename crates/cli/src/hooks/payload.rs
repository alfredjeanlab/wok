// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook payload structures for passing data to hook scripts.

use crate::models::{Event, Issue};
use chrono::{DateTime, Utc};
use serde::Serialize;

use super::HookEvent;

/// JSON payload passed to hook scripts via stdin.
#[derive(Debug, Clone, Serialize)]
pub struct HookPayload {
    /// Event type that triggered the hook (e.g., "issue.created").
    pub event: String,
    /// Timestamp when the event occurred.
    pub timestamp: DateTime<Utc>,
    /// Details about the issue.
    pub issue: IssuePayload,
    /// Details about the change that triggered the event.
    pub change: ChangePayload,
}

/// Issue details included in hook payload.
#[derive(Debug, Clone, Serialize)]
pub struct IssuePayload {
    /// Issue ID (e.g., "proj-a1b2").
    pub id: String,
    /// Issue type (e.g., "bug", "task").
    #[serde(rename = "type")]
    pub issue_type: String,
    /// Issue title.
    pub title: String,
    /// Issue status (e.g., "todo", "in_progress").
    pub status: String,
    /// Assignee if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Labels attached to the issue.
    pub labels: Vec<String>,
}

/// Change details included in hook payload.
#[derive(Debug, Clone, Serialize)]
pub struct ChangePayload {
    /// Previous value (for edits, status changes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    /// New value (for edits, labels, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    /// Reason provided for the change.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookPayload {
    /// Build a hook payload from an event and issue context.
    #[must_use]
    pub fn build(event: &Event, issue: &Issue, labels: &[String]) -> Self {
        let hook_event: HookEvent = event.action.into();

        HookPayload {
            event: hook_event.as_event_name().to_string(),
            timestamp: event.created_at,
            issue: IssuePayload {
                id: issue.id.clone(),
                issue_type: issue.issue_type.as_str().to_string(),
                title: issue.title.clone(),
                status: issue.status.as_str().to_string(),
                assignee: issue.assignee.clone(),
                labels: labels.to_vec(),
            },
            change: ChangePayload {
                old_value: event.old_value.clone(),
                new_value: event.new_value.clone(),
                reason: event.reason.clone(),
            },
        }
    }
}

#[cfg(test)]
#[path = "payload_tests.rs"]
mod tests;
