// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::{Action, Issue, IssueType};

#[test]
fn test_log_and_get_events() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    let event = Event::new("test-1234".to_string(), Action::Created);
    db.log_event(&event).unwrap();

    let event = Event::new("test-1234".to_string(), Action::Started);
    db.log_event(&event).unwrap();

    let events = db.get_events("test-1234").unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].action, Action::Created);
    assert_eq!(events[1].action, Action::Started);
}

#[test]
fn test_event_with_values() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    let event = Event::new("test-1234".to_string(), Action::Edited)
        .with_values(Some("Old title".to_string()), Some("New title".to_string()));
    db.log_event(&event).unwrap();

    let events = db.get_events("test-1234").unwrap();
    assert_eq!(events[0].old_value, Some("Old title".to_string()));
    assert_eq!(events[0].new_value, Some("New title".to_string()));
}

#[test]
fn test_get_recent_events() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    for _ in 0..5 {
        let event = Event::new("test-1234".to_string(), Action::Noted);
        db.log_event(&event).unwrap();
    }

    let events = db.get_recent_events(3).unwrap();
    assert_eq!(events.len(), 3);
}
