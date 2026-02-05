// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::models::{Action, Event, IssueType, Status};
use chrono::Utc;

fn make_test_issue() -> Issue {
    Issue {
        id: "test-123".to_string(),
        issue_type: IssueType::Bug,
        title: "Fix login bug".to_string(),
        description: None,
        status: Status::InProgress,
        assignee: Some("alice".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    }
}

fn make_test_event(action: Action) -> Event {
    Event {
        id: 1,
        issue_id: "test-123".to_string(),
        action,
        old_value: Some("todo".to_string()),
        new_value: Some("in_progress".to_string()),
        reason: None,
        created_at: Utc::now(),
    }
}

#[test]
fn test_payload_from_event() {
    let issue = make_test_issue();
    let event = make_test_event(Action::Started);
    let labels = vec!["urgent".to_string(), "backend".to_string()];

    let payload = HookPayload::from_event(&event, &issue, labels.clone());

    assert_eq!(payload.event, "issue.started");
    assert_eq!(payload.issue.id, "test-123");
    assert_eq!(payload.issue.r#type, "bug");
    assert_eq!(payload.issue.title, "Fix login bug");
    assert_eq!(payload.issue.status, "in_progress");
    assert_eq!(payload.issue.assignee, Some("alice".to_string()));
    assert_eq!(payload.issue.labels, labels);
    assert_eq!(payload.change.old_value, Some("todo".to_string()));
    assert_eq!(payload.change.new_value, Some("in_progress".to_string()));
    assert!(payload.change.reason.is_none());
}

#[test]
fn test_payload_to_json() {
    let issue = make_test_issue();
    let event = make_test_event(Action::Done);
    let labels = vec!["urgent".to_string()];

    let payload = HookPayload::from_event(&event, &issue, labels);
    let json = payload.to_json().unwrap();

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["event"], "issue.done");
    assert_eq!(parsed["issue"]["id"], "test-123");
    assert_eq!(parsed["issue"]["type"], "bug");
}

#[test]
fn test_payload_event_mapping() {
    let issue = make_test_issue();
    let labels = vec![];

    let test_cases = [
        (Action::Created, "issue.created"),
        (Action::Edited, "issue.edited"),
        (Action::Started, "issue.started"),
        (Action::Stopped, "issue.stopped"),
        (Action::Done, "issue.done"),
        (Action::Closed, "issue.closed"),
        (Action::Reopened, "issue.reopened"),
        (Action::Labeled, "issue.labeled"),
        (Action::Unlabeled, "issue.unlabeled"),
    ];

    for (action, expected_event) in test_cases {
        let event = make_test_event(action);
        let payload = HookPayload::from_event(&event, &issue, labels.clone());
        assert_eq!(payload.event, expected_event);
    }
}
