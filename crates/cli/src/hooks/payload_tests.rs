// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::models::{Action, IssueType, Status};
use chrono::Utc;

fn make_issue(id: &str) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type: IssueType::Bug,
        title: "Test bug".to_string(),
        description: None,
        status: Status::InProgress,
        assignee: Some("alice".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    }
}

fn make_event(issue_id: &str, action: Action) -> Event {
    Event {
        id: 0,
        issue_id: issue_id.to_string(),
        action,
        old_value: None,
        new_value: None,
        reason: None,
        created_at: Utc::now(),
    }
}

#[test]
fn build_payload_basic() {
    let issue = make_issue("proj-abc");
    let event = make_event("proj-abc", Action::Created);
    let labels = vec!["urgent".to_string(), "backend".to_string()];

    let payload = HookPayload::build(&event, &issue, &labels);

    assert_eq!(payload.event, "issue.created");
    assert_eq!(payload.issue.id, "proj-abc");
    assert_eq!(payload.issue.issue_type, "bug");
    assert_eq!(payload.issue.title, "Test bug");
    assert_eq!(payload.issue.status, "in_progress");
    assert_eq!(payload.issue.assignee, Some("alice".to_string()));
    assert_eq!(payload.issue.labels, labels);
}

#[test]
fn build_payload_with_change_values() {
    let issue = make_issue("proj-abc");
    let mut event = make_event("proj-abc", Action::Labeled);
    event.old_value = None;
    event.new_value = Some("urgent".to_string());

    let payload = HookPayload::build(&event, &issue, &[]);

    assert_eq!(payload.change.old_value, None);
    assert_eq!(payload.change.new_value, Some("urgent".to_string()));
}

#[test]
fn build_payload_with_reason() {
    let issue = make_issue("proj-abc");
    let mut event = make_event("proj-abc", Action::Closed);
    event.reason = Some("Duplicate".to_string());

    let payload = HookPayload::build(&event, &issue, &[]);

    assert_eq!(payload.change.reason, Some("Duplicate".to_string()));
}

#[test]
fn payload_serializes_to_json() {
    let issue = make_issue("proj-abc");
    let event = make_event("proj-abc", Action::Done);
    let payload = HookPayload::build(&event, &issue, &["done".to_string()]);

    let json = serde_json::to_string(&payload).unwrap();

    assert!(json.contains("\"event\":\"issue.done\""));
    assert!(json.contains("\"id\":\"proj-abc\""));
    assert!(json.contains("\"type\":\"bug\""));
    assert!(json.contains("\"labels\":[\"done\"]"));
}

#[test]
fn payload_skips_none_values() {
    let mut issue = make_issue("proj-abc");
    issue.assignee = None;
    let event = make_event("proj-abc", Action::Created);

    let payload = HookPayload::build(&event, &issue, &[]);
    let json = serde_json::to_string(&payload).unwrap();

    // assignee should be skipped when None
    assert!(!json.contains("\"assignee\":null"));
}

#[test]
fn event_types_map_correctly() {
    let issue = make_issue("proj-abc");

    for action in [
        Action::Created,
        Action::Edited,
        Action::Started,
        Action::Stopped,
        Action::Done,
        Action::Closed,
        Action::Reopened,
        Action::Labeled,
        Action::Unlabeled,
        Action::Assigned,
        Action::Unassigned,
        Action::Noted,
        Action::Linked,
        Action::Unlinked,
        Action::Related,
        Action::Unrelated,
        Action::Unblocked,
    ] {
        let event = make_event("proj-abc", action);
        let payload = HookPayload::build(&event, &issue, &[]);
        assert!(payload.event.starts_with("issue."));
    }
}
