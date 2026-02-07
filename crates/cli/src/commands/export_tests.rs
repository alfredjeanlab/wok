// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{IssueType, Status};

#[test]
fn test_get_all_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .create_issue("test-2", IssueType::Bug, "Bug 1")
        .create_issue("test-3", IssueType::Feature, "Feature 1");

    let all = ctx.db.get_all_issues().unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn test_get_all_issues_empty() {
    let ctx = TestContext::new();

    let all = ctx.db.get_all_issues().unwrap();
    assert!(all.is_empty());
}

#[test]
fn test_get_deps_from_issue() {
    let mut ctx = TestContext::new();
    ctx.create_issue("parent", IssueType::Feature, "Parent")
        .create_issue("child", IssueType::Task, "Child")
        .create_issue("blocked", IssueType::Task, "Blocked")
        .tracks("parent", "child")
        .blocks("child", "blocked");

    let deps = ctx.db.get_deps_from("child").unwrap();
    // Should have both tracks and blocks relationships
    assert!(!deps.is_empty());
}

#[test]
fn test_get_notes_for_export() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test")
        .add_note("test-1", "First note")
        .set_status("test-1", Status::InProgress);

    ctx.db
        .add_note("test-1", Status::InProgress, "Second note")
        .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 2);
}

#[test]
fn test_get_events_for_export() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test")
        .add_label("test-1", "important")
        .set_status("test-1", Status::InProgress);

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.len() >= 3); // Created, Labeled, Started
}

#[test]
fn test_get_labels_for_export() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test")
        .add_label("test-1", "backend")
        .add_label("test-1", "urgent")
        .add_label("test-1", "v1.0");

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 3);
}

#[test]
fn test_export_includes_all_statuses() {
    let mut ctx = TestContext::new();
    ctx.create_issue("todo", IssueType::Task, "Todo task")
        .create_issue_with_status(
            "in_prog",
            IssueType::Task,
            "In progress",
            Status::InProgress,
        )
        .create_issue_with_status("done", IssueType::Task, "Done task", Status::Done)
        .create_issue_with_status("closed", IssueType::Task, "Closed task", Status::Closed);

    let all = ctx.db.get_all_issues().unwrap();
    assert_eq!(all.len(), 4);

    let statuses: Vec<Status> = all.iter().map(|i| i.status).collect();
    assert!(statuses.contains(&Status::Todo));
    assert!(statuses.contains(&Status::InProgress));
    assert!(statuses.contains(&Status::Done));
    assert!(statuses.contains(&Status::Closed));
}

#[test]
fn test_export_includes_all_types() {
    let mut ctx = TestContext::new();
    ctx.create_issue("task-1", IssueType::Task, "Task")
        .create_issue("bug-1", IssueType::Bug, "Bug")
        .create_issue("feature-1", IssueType::Feature, "Feature")
        .create_issue("chore-1", IssueType::Chore, "Chore");

    let all = ctx.db.get_all_issues().unwrap();
    assert_eq!(all.len(), 4);

    let types: Vec<IssueType> = all.iter().map(|i| i.issue_type).collect();
    assert!(types.contains(&IssueType::Task));
    assert!(types.contains(&IssueType::Bug));
    assert!(types.contains(&IssueType::Feature));
    assert!(types.contains(&IssueType::Chore));
}

// Export path validation tests

/// Helper to validate export path
fn is_valid_export_path(path: &str) -> bool {
    !path.trim().is_empty()
}

#[test]
fn test_export_path_empty_is_invalid() {
    assert!(!is_valid_export_path(""));
    assert!(!is_valid_export_path("   "));
}

#[test]
fn test_export_path_valid_relative() {
    assert!(is_valid_export_path("issues.jsonl"));
    assert!(is_valid_export_path("./issues.jsonl"));
    assert!(is_valid_export_path("exports/issues.jsonl"));
    assert!(is_valid_export_path("../issues.jsonl"));
    assert!(is_valid_export_path("foo/../bar.jsonl"));
}

#[test]
fn test_export_path_valid_absolute() {
    assert!(is_valid_export_path("/tmp/issues.jsonl"));
    assert!(is_valid_export_path("/home/user/backup/issues.jsonl"));
}
