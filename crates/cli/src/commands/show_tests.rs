// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{IssueType, Status};

#[test]
fn test_get_issue_details() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.id, "test-1");
    assert_eq!(issue.title, "Test issue");
    assert_eq!(issue.issue_type, IssueType::Task);
    assert_eq!(issue.status, Status::Todo);
}

#[test]
fn test_get_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = ctx.db.get_issue("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_get_issue_labels() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "urgent")
        .add_label("test-1", "backend");

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn test_get_issue_blockers() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked issue")
        .blocks("blocker", "blocked");

    let blockers = ctx.db.get_blockers("blocked").unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0], "blocker");
}

#[test]
fn test_get_issue_blocking() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked issue")
        .blocks("blocker", "blocked");

    let blocking = ctx.db.get_blocking("blocker").unwrap();
    assert_eq!(blocking.len(), 1);
    assert_eq!(blocking[0], "blocked");
}

#[test]
fn test_get_issue_parents() {
    let ctx = TestContext::new();
    ctx.create_issue("parent", IssueType::Feature, "Parent feature")
        .create_issue("child", IssueType::Task, "Child task")
        .tracks("parent", "child");

    let parents = ctx.db.get_tracking("child").unwrap();
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], "parent");
}

#[test]
fn test_get_issue_children() {
    let ctx = TestContext::new();
    ctx.create_issue("parent", IssueType::Feature, "Parent feature")
        .create_issue("child1", IssueType::Task, "Child 1")
        .create_issue("child2", IssueType::Task, "Child 2")
        .tracks("parent", "child1")
        .tracks("parent", "child2");

    let children = ctx.db.get_tracked("parent").unwrap();
    assert_eq!(children.len(), 2);
    assert!(children.contains(&"child1".to_string()));
    assert!(children.contains(&"child2".to_string()));
}

#[test]
fn test_get_issue_notes_by_status() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "Note in todo state")
        .set_status("test-1", Status::InProgress);

    // Add another note in in_progress state
    ctx.db
        .add_note("test-1", Status::InProgress, "Note in progress")
        .unwrap();

    let notes = ctx.db.get_notes_by_status("test-1").unwrap();
    // Should have notes grouped by status
    assert!(!notes.is_empty());
}

#[test]
fn test_get_issue_events() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .set_status("test-1", Status::InProgress)
        .add_label("test-1", "urgent");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.len() >= 3); // Created, Started, Labeled
}

#[test]
fn test_show_complex_issue() {
    let ctx = TestContext::new();
    ctx.create_issue("feature-1", IssueType::Feature, "Feature")
        .create_issue("task-1", IssueType::Task, "Task")
        .create_issue("task-2", IssueType::Task, "Blocked task")
        .tracks("feature-1", "task-1")
        .blocks("task-1", "task-2")
        .add_label("task-1", "backend")
        .add_note("task-1", "Implementation note")
        .set_status("task-1", Status::InProgress);

    // Verify all relationships
    let issue = ctx.db.get_issue("task-1").unwrap();
    assert_eq!(issue.status, Status::InProgress);

    let labels = ctx.db.get_labels("task-1").unwrap();
    assert!(labels.contains(&"backend".to_string()));

    let parents = ctx.db.get_tracking("task-1").unwrap();
    assert!(parents.contains(&"feature-1".to_string()));

    let blocking = ctx.db.get_blocking("task-1").unwrap();
    assert!(blocking.contains(&"task-2".to_string()));

    let notes = ctx.db.get_notes("task-1").unwrap();
    assert!(!notes.is_empty());

    let events = ctx.db.get_events("task-1").unwrap();
    assert!(events.len() >= 4);
}

#[test]
fn test_notes_for_json_output() {
    // Tests that get_notes returns flat list suitable for JSON serialization
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_note("test-1", "First note")
        .set_status("test-1", Status::InProgress);

    // Add note in new status
    ctx.db
        .add_note("test-1", Status::InProgress, "Second note")
        .unwrap();

    // get_notes returns flat list for JSON
    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 2);

    // Verify notes are serializable (they derive Serialize)
    let json = serde_json::to_string(&notes).unwrap();
    assert!(json.contains("First note"));
    assert!(json.contains("Second note"));
}

#[test]
fn test_issue_details_serialization() {
    use serde::Serialize;

    #[derive(Serialize)]
    struct IssueDetails {
        #[serde(flatten)]
        issue: crate::models::Issue,
        labels: Vec<String>,
        notes: Vec<crate::models::Note>,
    }

    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "backend")
        .add_note("test-1", "A note");

    let issue = ctx.db.get_issue("test-1").unwrap();
    let labels = ctx.db.get_labels("test-1").unwrap();
    let notes = ctx.db.get_notes("test-1").unwrap();

    let details = IssueDetails {
        issue,
        labels,
        notes,
    };

    // Should serialize to valid JSON
    let json = serde_json::to_string_pretty(&details).unwrap();
    assert!(json.contains("\"id\": \"test-1\""));
    assert!(json.contains("\"title\": \"Test issue\""));
    assert!(json.contains("\"issue_type\": \"task\""));
    assert!(json.contains("\"backend\""));
    assert!(json.contains("A note"));
}

// Tests for run_impl

use crate::commands::show::run_impl;

#[test]
fn test_run_impl_text_format() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = run_impl(&ctx.db, &["test-1".to_string()], "text");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = run_impl(&ctx.db, &["test-1".to_string()], "json");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_labels() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "urgent");

    let result = run_impl(&ctx.db, &["test-1".to_string()], "text");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = run_impl(&ctx.db, &["nonexistent".to_string()], "text");
    assert!(result.is_err());
}

#[test]
fn test_run_impl_invalid_format() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = run_impl(&ctx.db, &["test-1".to_string()], "invalid");
    assert!(result.is_err());
}

#[test]
fn test_run_impl_multiple_issues_text() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "First issue")
        .create_issue("test-2", IssueType::Task, "Second issue");

    let result = run_impl(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        "text",
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_multiple_issues_json() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "First issue")
        .create_issue("test-2", IssueType::Task, "Second issue");

    let result = run_impl(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        "json",
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_fails_if_any_id_invalid() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Valid issue");

    // Should fail because "nonexistent" is invalid, even though test-1 is valid
    let result = run_impl(
        &ctx.db,
        &["test-1".to_string(), "nonexistent".to_string()],
        "text",
    );
    assert!(result.is_err());
}
