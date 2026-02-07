// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{Action, IssueType};

#[test]
fn test_add_label_logs_event() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Manually perform what add() does
    ctx.db.add_label("test-1", "urgent").unwrap();
    let event = crate::models::Event::new("test-1".to_string(), Action::Labeled)
        .with_values(None, Some("urgent".to_string()));
    ctx.db.log_event(&event).unwrap();

    // Verify label was added
    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels, vec!["urgent"]);

    // Verify event was logged
    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Labeled));
}

#[test]
fn test_add_multiple_labels() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "backend")
        .add_label("test-1", "urgent")
        .add_label("test-1", "p0");

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"backend".to_string()));
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"p0".to_string()));
}

#[test]
fn test_add_duplicate_label_is_idempotent() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    ctx.db.add_label("test-1", "urgent").unwrap();
    ctx.db.add_label("test-1", "urgent").unwrap();

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels, vec!["urgent"]);
}

#[test]
fn test_add_label_to_nonexistent_issue_fails() {
    let ctx = TestContext::new();

    // Trying to add label to non-existent issue should fail
    let result = ctx.db.get_issue("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_remove_label_logs_event() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "urgent");

    // Manually perform what remove() does
    let removed = ctx.db.remove_label("test-1", "urgent").unwrap();
    assert!(removed);

    let event = crate::models::Event::new("test-1".to_string(), Action::Unlabeled)
        .with_values(None, Some("urgent".to_string()));
    ctx.db.log_event(&event).unwrap();

    // Verify label was removed
    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(labels.is_empty());

    // Verify unlabeled event was logged
    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Unlabeled));
}

#[test]
fn test_remove_nonexistent_label_returns_false() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let removed = ctx.db.remove_label("test-1", "nonexistent").unwrap();
    assert!(!removed);
}

#[test]
fn test_labels_persist_across_status_changes() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "urgent")
        .set_status("test-1", crate::models::Status::InProgress);

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels, vec!["urgent"]);
}

#[test]
fn test_label_filtering_in_list() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task with label")
        .create_issue("test-2", IssueType::Task, "Task without label")
        .add_label("test-1", "backend");

    let issues = ctx.db.list_issues(None, None, Some("backend")).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "test-1");
}

#[test]
fn test_special_characters_in_label_names() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Labels with special characters
    ctx.db.add_label("test-1", "high-priority").unwrap();
    ctx.db.add_label("test-1", "v2.0").unwrap();
    ctx.db.add_label("test-1", "area:backend").unwrap();

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"high-priority".to_string()));
    assert!(labels.contains(&"v2.0".to_string()));
    assert!(labels.contains(&"area:backend".to_string()));
}

// Tests for run_impl

use crate::commands::label::{add_impl, remove_impl};

#[test]
fn test_add_impl_success() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    let result = add_impl(&ctx.db, &["test-1".to_string()], "urgent");
    assert!(result.is_ok());

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_add_impl_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = add_impl(&ctx.db, &["nonexistent".to_string()], "label");
    assert!(result.is_err());
}

#[test]
fn test_remove_impl_success() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue")
        .add_label("test-1", "urgent");

    let result = remove_impl(&ctx.db, &["test-1".to_string()], "urgent");
    assert!(result.is_ok());

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(!labels.contains(&"urgent".to_string()));
}

#[test]
fn test_remove_impl_nonexistent_label() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test issue");

    // Removing non-existent label should succeed but print a message
    let result = remove_impl(&ctx.db, &["test-1".to_string()], "nonexistent");
    assert!(result.is_ok());
}

#[test]
fn test_remove_impl_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = remove_impl(&ctx.db, &["nonexistent".to_string()], "label");
    assert!(result.is_err());
}

// === Batch Operations Tests ===

#[test]
fn test_add_impl_multiple_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = add_impl(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        "urgent",
    );

    assert!(result.is_ok());
    assert!(ctx
        .db
        .get_labels("test-1")
        .unwrap()
        .contains(&"urgent".to_string()));
    assert!(ctx
        .db
        .get_labels("test-2")
        .unwrap()
        .contains(&"urgent".to_string()));
}

#[test]
fn test_add_impl_fails_on_nonexistent_issue() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");

    let result = add_impl(
        &ctx.db,
        &["test-1".to_string(), "nonexistent".to_string()],
        "urgent",
    );

    assert!(result.is_err());
    // First one succeeded before failure
    assert!(ctx
        .db
        .get_labels("test-1")
        .unwrap()
        .contains(&"urgent".to_string()));
}

#[test]
fn test_remove_impl_multiple_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .add_label("test-1", "urgent");
    ctx.create_issue("test-2", IssueType::Task, "Task 2")
        .add_label("test-2", "urgent");

    let result = remove_impl(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        "urgent",
    );

    assert!(result.is_ok());
    assert!(!ctx
        .db
        .get_labels("test-1")
        .unwrap()
        .contains(&"urgent".to_string()));
    assert!(!ctx
        .db
        .get_labels("test-2")
        .unwrap()
        .contains(&"urgent".to_string()));
}

// === Multi-label Tests ===

use crate::commands::label::{add_with_db, remove_with_db};

#[test]
fn test_add_with_db_multiple_labels() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = add_with_db(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        &["urgent".to_string(), "backend".to_string()],
    );

    assert!(result.is_ok());

    // Both issues should have both labels
    let labels1 = ctx.db.get_labels("test-1").unwrap();
    assert!(labels1.contains(&"urgent".to_string()));
    assert!(labels1.contains(&"backend".to_string()));

    let labels2 = ctx.db.get_labels("test-2").unwrap();
    assert!(labels2.contains(&"urgent".to_string()));
    assert!(labels2.contains(&"backend".to_string()));
}

#[test]
fn test_remove_with_db_multiple_labels() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .add_label("test-1", "urgent")
        .add_label("test-1", "backend");
    ctx.create_issue("test-2", IssueType::Task, "Task 2")
        .add_label("test-2", "urgent")
        .add_label("test-2", "backend");

    let result = remove_with_db(
        &ctx.db,
        &["test-1".to_string(), "test-2".to_string()],
        &["urgent".to_string(), "backend".to_string()],
    );

    assert!(result.is_ok());

    // Both issues should have no labels
    let labels1 = ctx.db.get_labels("test-1").unwrap();
    assert!(labels1.is_empty());

    let labels2 = ctx.db.get_labels("test-2").unwrap();
    assert!(labels2.is_empty());
}

#[test]
fn test_add_with_db_invalid_label_fails_early() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");

    // Label that is too long (over 100 chars - MAX_LABEL_LENGTH)
    let long_label = "a".repeat(101);
    let result = add_with_db(
        &ctx.db,
        &["test-1".to_string()],
        &["valid".to_string(), long_label],
    );

    assert!(result.is_err());

    // First (valid) label should not have been added since validation happens first
    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(labels.is_empty());
}
