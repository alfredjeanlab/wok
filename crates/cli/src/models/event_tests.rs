// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

// Action tests - parameterized for all action types
#[parameterized(
    created = { Action::Created, "created" },
    edited = { Action::Edited, "edited" },
    started = { Action::Started, "started" },
    stopped = { Action::Stopped, "stopped" },
    done = { Action::Done, "done" },
    closed = { Action::Closed, "closed" },
    reopened = { Action::Reopened, "reopened" },
    labeled = { Action::Labeled, "labeled" },
    unlabeled = { Action::Unlabeled, "unlabeled" },
    related = { Action::Related, "related" },
    unrelated = { Action::Unrelated, "unrelated" },
    linked = { Action::Linked, "linked" },
    unlinked = { Action::Unlinked, "unlinked" },
    noted = { Action::Noted, "noted" },
    unblocked = { Action::Unblocked, "unblocked" },
)]
fn test_action_roundtrip(action: Action, expected: &str) {
    // Test as_str, Display, and FromStr in one test
    assert_eq!(action.as_str(), expected);
    assert_eq!(action.to_string(), expected);
    assert_eq!(expected.parse::<Action>().unwrap(), action);
}

#[parameterized(
    created_upper = { "CREATED", Action::Created },
    started_mixed = { "Started", Action::Started },
)]
fn test_action_from_str_case_insensitive(input: &str, expected: Action) {
    assert_eq!(input.parse::<Action>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    completed = { "completed" },
)]
fn test_action_from_str_invalid(input: &str) {
    assert!(input.parse::<Action>().is_err());
}

// Event tests
#[test]
fn test_event_new() {
    let event = Event::new("issue-123".to_string(), Action::Created);

    assert_eq!(event.id, 0); // Default, set by database
    assert_eq!(event.issue_id, "issue-123");
    assert_eq!(event.action, Action::Created);
    assert!(event.old_value.is_none());
    assert!(event.new_value.is_none());
    assert!(event.reason.is_none());
}

#[test]
fn test_event_with_values() {
    let event = Event::new("issue-123".to_string(), Action::Edited)
        .with_values(Some("old title".to_string()), Some("new title".to_string()));

    assert_eq!(event.old_value, Some("old title".to_string()));
    assert_eq!(event.new_value, Some("new title".to_string()));
}

#[test]
fn test_event_with_reason() {
    let event = Event::new("issue-123".to_string(), Action::Closed)
        .with_reason(Some("wontfix".to_string()));

    assert_eq!(event.reason, Some("wontfix".to_string()));
}

#[test]
fn test_event_builder_chain() {
    let event = Event::new("issue-123".to_string(), Action::Edited)
        .with_values(Some("todo".to_string()), Some("in_progress".to_string()))
        .with_reason(Some("starting work".to_string()));

    assert_eq!(event.issue_id, "issue-123");
    assert_eq!(event.action, Action::Edited);
    assert_eq!(event.old_value, Some("todo".to_string()));
    assert_eq!(event.new_value, Some("in_progress".to_string()));
    assert_eq!(event.reason, Some("starting work".to_string()));
}

#[test]
fn test_event_with_none_values() {
    let event = Event::new("issue-123".to_string(), Action::Labeled)
        .with_values(None, Some("urgent".to_string()));

    assert!(event.old_value.is_none());
    assert_eq!(event.new_value, Some("urgent".to_string()));
}
