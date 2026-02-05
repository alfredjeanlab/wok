// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::models::Action;

#[test]
fn hook_event_from_action() {
    assert_eq!(HookEvent::from(Action::Created), HookEvent::Created);
    assert_eq!(HookEvent::from(Action::Edited), HookEvent::Edited);
    assert_eq!(HookEvent::from(Action::Started), HookEvent::Started);
    assert_eq!(HookEvent::from(Action::Stopped), HookEvent::Stopped);
    assert_eq!(HookEvent::from(Action::Done), HookEvent::Done);
    assert_eq!(HookEvent::from(Action::Closed), HookEvent::Closed);
    assert_eq!(HookEvent::from(Action::Reopened), HookEvent::Reopened);
    assert_eq!(HookEvent::from(Action::Labeled), HookEvent::Labeled);
    assert_eq!(HookEvent::from(Action::Unlabeled), HookEvent::Unlabeled);
    assert_eq!(HookEvent::from(Action::Assigned), HookEvent::Assigned);
    assert_eq!(HookEvent::from(Action::Unassigned), HookEvent::Unassigned);
    assert_eq!(HookEvent::from(Action::Noted), HookEvent::Noted);
    assert_eq!(HookEvent::from(Action::Linked), HookEvent::Linked);
    assert_eq!(HookEvent::from(Action::Unlinked), HookEvent::Unlinked);
    assert_eq!(HookEvent::from(Action::Related), HookEvent::Related);
    assert_eq!(HookEvent::from(Action::Unrelated), HookEvent::Unrelated);
    assert_eq!(HookEvent::from(Action::Unblocked), HookEvent::Unblocked);
}

#[test]
fn event_names_are_correct() {
    assert_eq!(HookEvent::Created.as_event_name(), "issue.created");
    assert_eq!(HookEvent::Edited.as_event_name(), "issue.edited");
    assert_eq!(HookEvent::Started.as_event_name(), "issue.started");
    assert_eq!(HookEvent::Stopped.as_event_name(), "issue.stopped");
    assert_eq!(HookEvent::Done.as_event_name(), "issue.done");
    assert_eq!(HookEvent::Closed.as_event_name(), "issue.closed");
    assert_eq!(HookEvent::Reopened.as_event_name(), "issue.reopened");
    assert_eq!(HookEvent::Labeled.as_event_name(), "issue.labeled");
    assert_eq!(HookEvent::Unlabeled.as_event_name(), "issue.unlabeled");
    assert_eq!(HookEvent::Assigned.as_event_name(), "issue.assigned");
    assert_eq!(HookEvent::Unassigned.as_event_name(), "issue.unassigned");
    assert_eq!(HookEvent::Noted.as_event_name(), "issue.noted");
    assert_eq!(HookEvent::Linked.as_event_name(), "issue.linked");
    assert_eq!(HookEvent::Unlinked.as_event_name(), "issue.unlinked");
    assert_eq!(HookEvent::Related.as_event_name(), "issue.related");
    assert_eq!(HookEvent::Unrelated.as_event_name(), "issue.unrelated");
    assert_eq!(HookEvent::Unblocked.as_event_name(), "issue.unblocked");
}

#[test]
fn matches_exact_pattern() {
    assert!(HookEvent::Created.matches_pattern("issue.created"));
    assert!(HookEvent::Done.matches_pattern("issue.done"));
    assert!(!HookEvent::Created.matches_pattern("issue.done"));
    assert!(!HookEvent::Done.matches_pattern("issue.created"));
}

#[test]
fn matches_wildcard_pattern() {
    assert!(HookEvent::Created.matches_pattern("issue.*"));
    assert!(HookEvent::Edited.matches_pattern("issue.*"));
    assert!(HookEvent::Done.matches_pattern("issue.*"));
    assert!(HookEvent::Closed.matches_pattern("issue.*"));
    assert!(HookEvent::Labeled.matches_pattern("issue.*"));
    assert!(HookEvent::Unblocked.matches_pattern("issue.*"));
}

#[test]
fn does_not_match_invalid_patterns() {
    assert!(!HookEvent::Created.matches_pattern("invalid"));
    assert!(!HookEvent::Created.matches_pattern("issue.invalid"));
    assert!(!HookEvent::Created.matches_pattern("*"));
    assert!(!HookEvent::Created.matches_pattern(""));
}
