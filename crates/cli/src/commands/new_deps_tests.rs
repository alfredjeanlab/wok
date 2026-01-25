// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Tests for dependency flags in the `new` command (--blocks, --blocked-by, --tracks, --tracked-by).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::OutputFormat;
use crate::commands::new::run_impl;
use crate::commands::testing::TestContext;
use crate::models::{IssueType, Relation};

#[test]
fn test_run_impl_with_blocks() {
    let ctx = TestContext::new();

    // Create target issue first
    ctx.create_issue("test-target", IssueType::Task, "Target task");

    // Create new issue that blocks the target
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "bug".to_string(),
        Some("Blocker".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec!["test-target".to_string()],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // Find the new issue
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let blocker = issues.iter().find(|i| i.title == "Blocker").unwrap();

    // Verify dependency exists
    let deps = ctx.db.get_deps_from(&blocker.id).unwrap();
    assert!(deps
        .iter()
        .any(|d| d.to_id == "test-target" && d.relation == Relation::Blocks));
}

#[test]
fn test_run_impl_with_blocked_by() {
    let ctx = TestContext::new();

    // Create blocker issue first
    ctx.create_issue("test-blocker", IssueType::Task, "Blocker task");

    // Create new issue that is blocked by the blocker
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Blocked task".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec!["test-blocker".to_string()],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // The blocker should now have a blocks relationship to the new issue
    let deps = ctx.db.get_deps_from("test-blocker").unwrap();
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let blocked = issues.iter().find(|i| i.title == "Blocked task").unwrap();
    assert!(deps
        .iter()
        .any(|d| d.to_id == blocked.id && d.relation == Relation::Blocks));
}

#[test]
fn test_run_impl_with_tracks() {
    let ctx = TestContext::new();

    // Create subtask first
    ctx.create_issue("test-subtask", IssueType::Task, "Subtask");

    // Create feature that tracks the subtask
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "feature".to_string(),
        Some("Feature".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec!["test-subtask".to_string()],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // Find the feature
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let feature = issues.iter().find(|i| i.title == "Feature").unwrap();

    // Verify bidirectional relationship
    let deps = ctx.db.get_deps_from(&feature.id).unwrap();
    assert!(deps
        .iter()
        .any(|d| d.to_id == "test-subtask" && d.relation == Relation::Tracks));

    let reverse_deps = ctx.db.get_deps_from("test-subtask").unwrap();
    assert!(reverse_deps
        .iter()
        .any(|d| d.to_id == feature.id && d.relation == Relation::TrackedBy));
}

#[test]
fn test_run_impl_with_tracked_by() {
    let ctx = TestContext::new();

    // Create feature first
    ctx.create_issue("test-feature", IssueType::Feature, "Feature");

    // Create task that is tracked by the feature
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Subtask".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec!["test-feature".to_string()],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // Find the subtask
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let subtask = issues.iter().find(|i| i.title == "Subtask").unwrap();

    // Verify bidirectional relationship
    let deps = ctx.db.get_deps_from("test-feature").unwrap();
    assert!(deps
        .iter()
        .any(|d| d.to_id == subtask.id && d.relation == Relation::Tracks));

    let reverse_deps = ctx.db.get_deps_from(&subtask.id).unwrap();
    assert!(reverse_deps
        .iter()
        .any(|d| d.to_id == "test-feature" && d.relation == Relation::TrackedBy));
}

#[test]
fn test_run_impl_with_comma_separated_blocks() {
    let ctx = TestContext::new();

    // Create target issues
    ctx.create_issue("target-1", IssueType::Task, "Target 1");
    ctx.create_issue("target-2", IssueType::Task, "Target 2");

    // Create new issue that blocks both
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "bug".to_string(),
        Some("Multi-blocker".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec!["target-1,target-2".to_string()],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // Find the new issue
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let blocker = issues.iter().find(|i| i.title == "Multi-blocker").unwrap();

    // Verify both dependencies exist
    let deps = ctx.db.get_deps_from(&blocker.id).unwrap();
    assert!(deps
        .iter()
        .any(|d| d.to_id == "target-1" && d.relation == Relation::Blocks));
    assert!(deps
        .iter()
        .any(|d| d.to_id == "target-2" && d.relation == Relation::Blocks));
}

#[test]
fn test_run_impl_with_invalid_target_fails() {
    let ctx = TestContext::new();

    // Create new issue that blocks nonexistent target
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "bug".to_string(),
        Some("Bad blocker".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec!["nonexistent".to_string()],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    // Should fail because target doesn't exist
    assert!(result.is_err());
}
