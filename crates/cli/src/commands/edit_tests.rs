// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::edit::run_impl;
use crate::commands::testing::TestContext;
use crate::models::{Action, IssueType, Status};

#[test]
fn test_update_title() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original title");

    let result = run_impl(&ctx.db, "test-1", "title", "Updated title");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.title, "Updated title");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Edited));
}

#[test]
fn test_update_description() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My issue");

    let result = run_impl(&ctx.db, "test-1", "description", "New description");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.description, Some("New description".to_string()));

    let events = ctx.db.get_events("test-1").unwrap();
    let edit_events: Vec<_> = events
        .iter()
        .filter(|e| e.action == Action::Edited)
        .collect();
    assert!(!edit_events.is_empty());
}

#[test]
fn test_edit_nonexistent_issue_fails() {
    let ctx = TestContext::new();

    let result = run_impl(&ctx.db, "nonexistent", "title", "New title");
    assert!(result.is_err());
}

#[test]
fn test_edit_preserves_status() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original")
        .set_status("test-1", Status::InProgress);

    run_impl(&ctx.db, "test-1", "title", "Updated").unwrap();

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::InProgress);
}

#[test]
fn test_edit_preserves_labels() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original")
        .add_label("test-1", "important")
        .add_label("test-1", "backend");

    run_impl(&ctx.db, "test-1", "title", "Updated").unwrap();

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"important".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn test_empty_title_rejected() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original");

    let result = run_impl(&ctx.db, "test-1", "title", "");
    assert!(result.is_err());
}

#[test]
fn test_whitespace_title_rejected() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original");

    let result = run_impl(&ctx.db, "test-1", "title", "   ");
    assert!(result.is_err());
}

#[test]
fn test_description_replaces_existing() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My issue");

    ctx.db
        .update_issue_description("test-1", "Old description")
        .unwrap();

    let result = run_impl(&ctx.db, "test-1", "description", "New description");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.description, Some("New description".to_string()));
}

#[test]
fn test_description_too_long() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My issue");

    let long_desc = "x".repeat(10_001);
    let result = run_impl(&ctx.db, "test-1", "description", &long_desc);
    assert!(result.is_err());
}

#[test]
fn test_empty_description_allowed() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My issue");

    ctx.db
        .update_issue_description("test-1", "Has desc")
        .unwrap();

    let result = run_impl(&ctx.db, "test-1", "description", "");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.description, Some("".to_string()));
}

#[test]
fn test_description_events_logged() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My issue");

    run_impl(&ctx.db, "test-1", "description", "Description").unwrap();

    let events = ctx.db.get_events("test-1").unwrap();
    let edit_events: Vec<_> = events
        .iter()
        .filter(|e| e.action == Action::Edited)
        .collect();
    assert!(!edit_events.is_empty());
    assert_eq!(
        edit_events.last().unwrap().new_value,
        Some("Description".to_string())
    );
}

#[test]
fn test_description_preserves_other_fields() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Bug, "Original")
        .set_status("test-1", Status::InProgress)
        .add_label("test-1", "urgent");

    run_impl(&ctx.db, "test-1", "description", "New desc").unwrap();

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Bug);
    assert_eq!(issue.status, Status::InProgress);
    assert_eq!(issue.title, "Original");

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_update_type() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My task");

    let result = run_impl(&ctx.db, "test-1", "type", "bug");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Bug);

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Edited));
}

#[test]
fn test_update_type_invalid() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My task");

    let result = run_impl(&ctx.db, "test-1", "type", "invalid");
    assert!(result.is_err());
}

#[test]
fn test_update_type_same_no_event() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My task");

    let result = run_impl(&ctx.db, "test-1", "type", "task");
    assert!(result.is_ok());

    let events = ctx.db.get_events("test-1").unwrap();
    // Only the Created event, no Edited event since type didn't change
    assert!(!events.iter().any(|e| e.action == Action::Edited));
}

#[test]
fn test_update_type_preserves_other_fields() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original title")
        .set_status("test-1", Status::InProgress)
        .add_label("test-1", "urgent");

    ctx.db
        .update_issue_description("test-1", "Description")
        .unwrap();

    run_impl(&ctx.db, "test-1", "type", "feature").unwrap();

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Feature);
    assert_eq!(issue.status, Status::InProgress);
    assert_eq!(issue.title, "Original title");
    assert_eq!(issue.description, Some("Description".to_string()));

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_unknown_attribute_fails() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My task");

    let result = run_impl(&ctx.db, "test-1", "unknown", "value");
    assert!(result.is_err());
}

#[test]
fn test_attribute_case_insensitive() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Original");

    // Test uppercase
    let result = run_impl(&ctx.db, "test-1", "TITLE", "New title");
    assert!(result.is_ok());

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.title, "New title");
}
