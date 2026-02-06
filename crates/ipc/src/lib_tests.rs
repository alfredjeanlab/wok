// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for IPC protocol types and framing.

#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use chrono::Utc;

use super::*;
use yare::parameterized;

#[parameterized(
    status = { DaemonRequest::Status },
    shutdown = { DaemonRequest::Shutdown },
    ping = { DaemonRequest::Ping },
    hello = { DaemonRequest::Hello { version: "0.1.0".to_string() } },
)]
fn daemon_request_serialization(request: DaemonRequest) {
    let json = serde_json::to_string(&request).unwrap();
    let parsed: DaemonRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request, parsed);
}

#[parameterized(
    status = { DaemonResponse::Status(DaemonStatus::new(1234, 3600)) },
    shutting_down = { DaemonResponse::ShuttingDown },
    pong = { DaemonResponse::Pong },
    error = { DaemonResponse::Error { message: "test error".to_string() } },
    hello = { DaemonResponse::Hello { version: "0.1.0".to_string() } },
)]
fn daemon_response_serialization(response: DaemonResponse) {
    let json = serde_json::to_string(&response).unwrap();
    let parsed: DaemonResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response, parsed);
}

#[test]
fn daemon_status_new() {
    let status = DaemonStatus::new(5678, 7200);
    assert_eq!(status.pid, 5678);
    assert_eq!(status.uptime_secs, 7200);
}

#[parameterized(
    status = { DaemonRequest::Status },
    shutdown = { DaemonRequest::Shutdown },
    ping = { DaemonRequest::Ping },
    hello = { DaemonRequest::Hello { version: "0.1.0".to_string() } },
)]
fn framing_roundtrip_request(request: DaemonRequest) {
    let mut buf = Vec::new();
    framing::write_message(&mut buf, &request).unwrap();

    let mut cursor = Cursor::new(buf);
    let decoded: DaemonRequest = framing::read_message(&mut cursor).unwrap();
    assert_eq!(request, decoded);
}

#[parameterized(
    status = { DaemonResponse::Status(DaemonStatus::new(1000, 100)) },
    shutting_down = { DaemonResponse::ShuttingDown },
    pong = { DaemonResponse::Pong },
    error = { DaemonResponse::Error { message: "test".to_string() } },
    hello = { DaemonResponse::Hello { version: "0.1.0".to_string() } },
)]
fn framing_roundtrip_response(response: DaemonResponse) {
    let mut buf = Vec::new();
    framing::write_message(&mut buf, &response).unwrap();

    let mut cursor = Cursor::new(buf);
    let decoded: DaemonResponse = framing::read_message(&mut cursor).unwrap();
    assert_eq!(response, decoded);
}

#[test]
fn status_display() {
    assert_eq!(Status::Todo.to_string(), "todo");
    assert_eq!(Status::InProgress.to_string(), "in_progress");
    assert_eq!(Status::Done.to_string(), "done");
    assert_eq!(Status::Closed.to_string(), "closed");
}

#[test]
fn status_as_str() {
    assert_eq!(Status::Todo.as_str(), "todo");
    assert_eq!(Status::InProgress.as_str(), "in_progress");
    assert_eq!(Status::Done.as_str(), "done");
    assert_eq!(Status::Closed.as_str(), "closed");
}

#[test]
fn action_display() {
    assert_eq!(Action::Created.to_string(), "created");
    assert_eq!(Action::Linked.to_string(), "linked");
    assert_eq!(Action::Unlinked.to_string(), "unlinked");
}

#[test]
fn relation_display() {
    assert_eq!(Relation::Blocks.to_string(), "blocks");
    assert_eq!(Relation::TrackedBy.to_string(), "tracked-by");
    assert_eq!(Relation::Tracks.to_string(), "tracks");
}

#[test]
fn event_builder() {
    let event = Event::new("test-001".to_string(), Action::Edited)
        .with_values(Some("old".to_string()), Some("new".to_string()))
        .with_reason(Some("fixing typo".to_string()));

    assert_eq!(event.issue_id, "test-001");
    assert_eq!(event.action, Action::Edited);
    assert_eq!(event.old_value.as_deref(), Some("old"));
    assert_eq!(event.new_value.as_deref(), Some("new"));
    assert_eq!(event.reason.as_deref(), Some("fixing typo"));
    assert_eq!(event.id, 0);
}

#[test]
fn core_to_ipc_issue_drops_hlc() {
    let now = Utc::now();
    let core_issue = wk_core::Issue {
        id: "test-abc123".to_string(),
        issue_type: IssueType::Task,
        title: "A task".to_string(),
        description: Some("Details".to_string()),
        status: Status::InProgress,
        assignee: Some("alice".to_string()),
        created_at: now,
        updated_at: now,
        closed_at: None,
        last_status_hlc: Some(wk_core::hlc::Hlc::new(42, 1, 100)),
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };

    let ipc_issue: Issue = core_issue.clone().into();

    assert_eq!(ipc_issue.id, core_issue.id);
    assert_eq!(ipc_issue.issue_type, core_issue.issue_type);
    assert_eq!(ipc_issue.title, core_issue.title);
    assert_eq!(ipc_issue.description, core_issue.description);
    assert_eq!(ipc_issue.status, core_issue.status);
    assert_eq!(ipc_issue.assignee, core_issue.assignee);
    assert_eq!(ipc_issue.created_at, core_issue.created_at);
    assert_eq!(ipc_issue.updated_at, core_issue.updated_at);
    assert_eq!(ipc_issue.closed_at, core_issue.closed_at);
}

#[test]
fn ipc_to_core_issue_sets_hlc_none() {
    let now = Utc::now();
    let ipc_issue = Issue {
        id: "test-def456".to_string(),
        issue_type: IssueType::Bug,
        title: "A bug".to_string(),
        description: None,
        status: Status::Done,
        assignee: None,
        created_at: now,
        updated_at: now,
        closed_at: Some(now),
    };

    let core_issue: wk_core::Issue = ipc_issue.clone().into();

    assert_eq!(core_issue.id, ipc_issue.id);
    assert_eq!(core_issue.issue_type, ipc_issue.issue_type);
    assert_eq!(core_issue.title, ipc_issue.title);
    assert_eq!(core_issue.description, ipc_issue.description);
    assert_eq!(core_issue.status, ipc_issue.status);
    assert_eq!(core_issue.assignee, ipc_issue.assignee);
    assert_eq!(core_issue.created_at, ipc_issue.created_at);
    assert_eq!(core_issue.updated_at, ipc_issue.updated_at);
    assert_eq!(core_issue.closed_at, ipc_issue.closed_at);
    assert!(core_issue.last_status_hlc.is_none());
    assert!(core_issue.last_title_hlc.is_none());
    assert!(core_issue.last_type_hlc.is_none());
    assert!(core_issue.last_description_hlc.is_none());
    assert!(core_issue.last_assignee_hlc.is_none());
}

#[test]
fn roundtrip_preserves_non_hlc_fields() {
    let now = Utc::now();
    let original = Issue {
        id: "test-rt789".to_string(),
        issue_type: IssueType::Feature,
        title: "Round trip".to_string(),
        description: Some("Should survive".to_string()),
        status: Status::Todo,
        assignee: Some("bob".to_string()),
        created_at: now,
        updated_at: now,
        closed_at: None,
    };

    let core_issue: wk_core::Issue = original.clone().into();
    let back: Issue = core_issue.into();

    assert_eq!(back, original);
}
