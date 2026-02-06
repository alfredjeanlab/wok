// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{Action, IssueType, Status};

#[test]
fn test_add_note_to_issue() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Add note like the command does
    let issue = ctx.db.get_issue("test-1").unwrap();
    ctx.db
        .add_note("test-1", issue.status, "This is a note")
        .unwrap();

    let event = crate::models::Event::new("test-1".to_string(), Action::Noted)
        .with_values(None, Some("This is a note".to_string()));
    ctx.db.log_event(&event).unwrap();

    // Verify note was added
    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "This is a note");
}

#[test]
fn test_note_captures_current_status() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::InProgress);

    // Add note while in progress
    ctx.db
        .add_note("test-1", Status::InProgress, "Working on it")
        .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].status, Status::InProgress);
}

#[test]
fn test_multiple_notes_at_different_statuses() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Note in todo
    ctx.db
        .add_note("test-1", Status::Todo, "Initial thought")
        .unwrap();

    // Move to in_progress and add note
    ctx.set_status("test-1", Status::InProgress);
    ctx.db
        .add_note("test-1", Status::InProgress, "Started work")
        .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 2);
}

#[test]
fn test_note_logs_event() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "Test note");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Noted));

    let noted_event = events.iter().find(|e| e.action == Action::Noted).unwrap();
    assert_eq!(noted_event.new_value, Some("Test note".to_string()));
}

#[test]
fn test_note_on_nonexistent_issue_fails() {
    let mut ctx = TestContext::new();

    let result = ctx.db.get_issue("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_empty_note_content() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Empty content is allowed by DB
    ctx.db.add_note("test-1", Status::Todo, "").unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "");
}

#[test]
fn test_multiline_note() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let multiline = "Line 1\nLine 2\nLine 3";
    ctx.db.add_note("test-1", Status::Todo, multiline).unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes[0].content, multiline);
}

#[test]
fn test_get_notes_by_status_groups_correctly() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Add notes at different statuses
    ctx.db
        .add_note("test-1", Status::Todo, "Todo note 1")
        .unwrap();
    ctx.db
        .add_note("test-1", Status::Todo, "Todo note 2")
        .unwrap();
    ctx.set_status("test-1", Status::InProgress);
    ctx.db
        .add_note("test-1", Status::InProgress, "Progress note")
        .unwrap();

    let grouped = ctx.db.get_notes_by_status("test-1").unwrap();
    // Should have entries for different statuses
    assert!(!grouped.is_empty());
}

// Tests for run_impl

use crate::commands::note::run_impl;

#[test]
fn test_run_impl_add_note() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = run_impl(&ctx.db, "test-1", "A new note", false);
    assert!(result.is_ok());

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert!(notes.iter().any(|n| n.content == "A new note"));
}

#[test]
fn test_run_impl_replace_note() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "Original note");

    let result = run_impl(&ctx.db, "test-1", "Replaced note", true);
    assert!(result.is_ok());

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Replaced note");
}

#[test]
fn test_run_impl_nonexistent_issue() {
    let mut ctx = TestContext::new();

    let result = run_impl(&ctx.db, "nonexistent", "A note", false);
    assert!(result.is_err());
}

#[test]
fn test_run_impl_replace_no_existing_note() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Try to replace when there's no note
    let result = run_impl(&ctx.db, "test-1", "New note", true);
    assert!(result.is_err());
}

#[test]
fn test_run_impl_closed_issue_rejected() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::Closed);

    let result = run_impl(&ctx.db, "test-1", "Should fail", false);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("cannot add notes to closed issues"),
        "Expected error about closed issues, got: {}",
        msg
    );
}

#[test]
fn test_run_impl_closed_issue_replace_rejected() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "Original note")
        .set_status("test-1", Status::Closed);

    let result = run_impl(&ctx.db, "test-1", "Should fail", true);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err
        .to_string()
        .contains("cannot add notes to closed issues"));
}
