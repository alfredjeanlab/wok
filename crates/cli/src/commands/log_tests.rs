// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{Action, IssueType, Status};

#[test]
fn test_get_events_for_issue() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.action == Action::Created));
}

#[test]
fn test_get_events_empty_for_nonexistent_issue() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Note: issue_exists check happens at command level
    let result = ctx.db.issue_exists("nonexistent").unwrap();
    assert!(!result);
}

#[test]
fn test_events_ordered_by_time() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::InProgress)
        .add_label("test-1", "urgent");

    let events = ctx.db.get_events("test-1").unwrap();
    // Events should be in chronological order
    assert!(events.len() >= 3);

    // First event should be Created
    assert_eq!(events[0].action, Action::Created);
}

#[test]
fn test_get_recent_events_across_issues() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .create_issue("test-2", IssueType::Bug, "Bug 1")
        .set_status("test-1", Status::InProgress);

    let recent = ctx.db.get_recent_events(10).unwrap();
    assert!(recent.len() >= 3);
}

#[test]
fn test_get_recent_events_with_limit() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .create_issue("test-2", IssueType::Task, "Task 2")
        .create_issue("test-3", IssueType::Task, "Task 3");

    let recent = ctx.db.get_recent_events(2).unwrap();
    assert_eq!(recent.len(), 2);
}

#[test]
fn test_status_change_events() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::InProgress)
        .set_status("test-1", Status::Done);

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Started));
    assert!(events.iter().any(|e| e.action == Action::Done));
}

#[test]
fn test_label_events() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "backend");

    // Remove the label manually to generate Unlabeled event
    ctx.db.remove_label("test-1", "backend").unwrap();
    let event = crate::models::Event::new("test-1".to_string(), Action::Unlabeled)
        .with_values(None, Some("backend".to_string()));
    ctx.db.log_event(&event).unwrap();

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Labeled));
    assert!(events.iter().any(|e| e.action == Action::Unlabeled));
}

#[test]
fn test_note_events() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "This is a note");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Noted));
}

#[test]
fn test_event_values() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::InProgress);

    let events = ctx.db.get_events("test-1").unwrap();
    let start_event = events.iter().find(|e| e.action == Action::Started).unwrap();

    // Started event should have old_value=todo, new_value=in_progress
    assert_eq!(start_event.old_value, Some("todo".to_string()));
    assert_eq!(start_event.new_value, Some("in_progress".to_string()));
}

// Tests for run_impl

use crate::commands::log::run_impl;

#[test]
fn test_run_impl_global_log() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .create_issue("test-2", IssueType::Bug, "Another issue");

    let result = run_impl(&ctx.db, None, 10);
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_issue_log() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = run_impl(&ctx.db, Some("test-1".to_string()), 10);
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = run_impl(&ctx.db, Some("nonexistent".to_string()), 10);
    assert!(result.is_err());
}

#[test]
fn test_run_impl_with_limit() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .create_issue("test-2", IssueType::Task, "Task 2")
        .create_issue("test-3", IssueType::Task, "Task 3");

    let result = run_impl(&ctx.db, None, 2);
    assert!(result.is_ok());
}
