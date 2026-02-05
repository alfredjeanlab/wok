// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use super::*;
use crate::models::Action;

#[test]
fn test_hook_event_as_event_name() {
    assert_eq!(HookEvent::Created.as_event_name(), "issue.created");
    assert_eq!(HookEvent::Done.as_event_name(), "issue.done");
    assert_eq!(HookEvent::Closed.as_event_name(), "issue.closed");
    assert_eq!(HookEvent::Labeled.as_event_name(), "issue.labeled");
}

#[test]
fn test_hook_event_matches_pattern_exact() {
    assert!(HookEvent::Created.matches_pattern("issue.created"));
    assert!(!HookEvent::Created.matches_pattern("issue.done"));
    assert!(!HookEvent::Done.matches_pattern("issue.created"));
}

#[test]
fn test_hook_event_matches_pattern_wildcard() {
    assert!(HookEvent::Created.matches_pattern("issue.*"));
    assert!(HookEvent::Done.matches_pattern("issue.*"));
    assert!(HookEvent::Closed.matches_pattern("issue.*"));
    assert!(HookEvent::Labeled.matches_pattern("issue.*"));
}

#[test]
fn test_hook_event_matches_pattern_no_match() {
    assert!(!HookEvent::Created.matches_pattern("other.created"));
    assert!(!HookEvent::Created.matches_pattern("issue."));
    assert!(!HookEvent::Created.matches_pattern("*"));
}

#[test]
fn test_hook_event_from_action() {
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
