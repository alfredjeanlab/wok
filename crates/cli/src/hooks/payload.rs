// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook payload building for stdin JSON.

use crate::models::Issue;
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::Event;

use super::event::HookEvent;

/// JSON payload passed to hook scripts via stdin.
#[derive(Debug, Clone, Serialize)]
pub struct HookPayload {
    /// The event that triggered this hook (e.g., "issue.created").
    pub event: String,
    /// When the event occurred.
    pub timestamp: DateTime<Utc>,
    /// The issue that was affected.
    pub issue: IssuePayload,
    /// Details about the change.
    pub change: ChangePayload,
}

/// Issue information included in the hook payload.
#[derive(Debug, Clone, Serialize)]
pub struct IssuePayload {
    /// Issue ID.
    pub id: String,
    /// Issue type (task, bug, etc.).
    pub r#type: String,
    /// Issue title.
    pub title: String,
    /// Current status.
    pub status: String,
    /// Assignee if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Labels attached to the issue.
    pub labels: Vec<String>,
}

/// Change information included in the hook payload.
#[derive(Debug, Clone, Serialize)]
pub struct ChangePayload {
    /// Previous value (for edits, status changes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    /// New value (for edits, tags, related issues).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    /// Reason for the change (for closes, reopens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HookPayload {
    /// Build a payload from an event and issue.
    pub fn from_event(event: &Event, issue: &Issue, labels: Vec<String>) -> Self {
        let hook_event: HookEvent = event.action.into();

        HookPayload {
            event: hook_event.as_event_name().to_string(),
            timestamp: event.created_at,
            issue: IssuePayload {
                id: issue.id.clone(),
                r#type: issue.issue_type.as_str().to_string(),
                title: issue.title.clone(),
                status: issue.status.as_str().to_string(),
                assignee: issue.assignee.clone(),
                labels,
            },
            change: ChangePayload {
                old_value: event.old_value.clone(),
                new_value: event.new_value.clone(),
                reason: event.reason.clone(),
            },
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
#[path = "payload_tests.rs"]
mod tests;
