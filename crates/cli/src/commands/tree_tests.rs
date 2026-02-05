// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::commands::testing::TestContext;
use crate::models::{IssueType, Status};

#[test]
fn test_get_tracked_for_parent() {
    let ctx = TestContext::new();
    ctx.create_issue("parent", IssueType::Feature, "Parent feature")
        .create_issue("child1", IssueType::Task, "Child 1")
        .create_issue("child2", IssueType::Task, "Child 2")
        .tracks("parent", "child1")
        .tracks("parent", "child2");

    let children = ctx.db.get_tracked("parent").unwrap();
    assert_eq!(children.len(), 2);
}

#[test]
fn test_get_tracked_empty() {
    let ctx = TestContext::new();
    ctx.create_issue("leaf", IssueType::Task, "Leaf task");

    let children = ctx.db.get_tracked("leaf").unwrap();
    assert!(children.is_empty());
}

#[test]
fn test_nested_hierarchy() {
    let ctx = TestContext::new();
    ctx.create_issue("feature", IssueType::Feature, "Feature")
        .create_issue("task", IssueType::Task, "Task under feature")
        .create_issue("subtask", IssueType::Task, "Subtask under task")
        .tracks("feature", "task")
        .tracks("task", "subtask");

    let feature_children = ctx.db.get_tracked("feature").unwrap();
    assert_eq!(feature_children.len(), 1);
    assert_eq!(feature_children[0], "task");

    let task_children = ctx.db.get_tracked("task").unwrap();
    assert_eq!(task_children.len(), 1);
    assert_eq!(task_children[0], "subtask");
}

#[test]
fn test_transitive_blockers() {
    let ctx = TestContext::new();
    ctx.create_issue("task-a", IssueType::Task, "Task A")
        .create_issue("task-b", IssueType::Task, "Task B")
        .create_issue("task-c", IssueType::Task, "Task C")
        .blocks("task-a", "task-b")
        .blocks("task-b", "task-c");

    // task-c is transitively blocked by task-a (via task-b)
    let blockers = ctx.db.get_transitive_blockers("task-c").unwrap();
    // Should include both task-a and task-b
    assert!(blockers.contains(&"task-a".to_string()));
    assert!(blockers.contains(&"task-b".to_string()));
}

#[test]
fn test_blockers_filtered_by_status() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker1", IssueType::Task, "Open blocker")
        .create_issue("blocker2", IssueType::Task, "Done blocker")
        .create_issue("blocked", IssueType::Task, "Blocked")
        .blocks("blocker1", "blocked")
        .blocks("blocker2", "blocked")
        .set_status("blocker2", Status::InProgress)
        .set_status("blocker2", Status::Done);

    // Only open blockers should count
    let blockers = ctx.db.get_transitive_blockers("blocked").unwrap();
    // blocker1 is still open, blocker2 is done
    assert!(blockers.contains(&"blocker1".to_string()));
    assert!(!blockers.contains(&"blocker2".to_string()));
}

#[test]
fn test_tree_with_blocking_and_hierarchy() {
    let ctx = TestContext::new();
    ctx.create_issue("feature", IssueType::Feature, "Feature")
        .create_issue("task1", IssueType::Task, "Task 1")
        .create_issue("task2", IssueType::Task, "Task 2")
        .create_issue("blocker", IssueType::Task, "Blocker")
        .tracks("feature", "task1")
        .tracks("feature", "task2")
        .blocks("blocker", "task2");

    // Feature has children
    let children = ctx.db.get_tracked("feature").unwrap();
    assert_eq!(children.len(), 2);

    // Task2 is blocked by blocker
    let blockers = ctx.db.get_transitive_blockers("task2").unwrap();
    assert!(blockers.contains(&"blocker".to_string()));
}

#[test]
fn test_issue_without_blockers() {
    let ctx = TestContext::new();
    ctx.create_issue("independent", IssueType::Task, "Independent task");

    let blockers = ctx.db.get_transitive_blockers("independent").unwrap();
    assert!(blockers.is_empty());
}

#[test]
fn test_multiple_direct_blockers() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker1", IssueType::Task, "Blocker 1")
        .create_issue("blocker2", IssueType::Task, "Blocker 2")
        .create_issue("blocked", IssueType::Task, "Blocked task")
        .blocks("blocker1", "blocked")
        .blocks("blocker2", "blocked");

    let blockers = ctx.db.get_transitive_blockers("blocked").unwrap();
    assert_eq!(blockers.len(), 2);
    assert!(blockers.contains(&"blocker1".to_string()));
    assert!(blockers.contains(&"blocker2".to_string()));
}

// Tests for run_impl

use crate::commands::tree::run_impl;

#[test]
fn test_run_impl_simple() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Feature, "Test feature")
        .create_issue("test-2", IssueType::Task, "Child task")
        .tracks("test-1", "test-2");

    let result = run_impl(&ctx.db, "test-1");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_leaf_node() {
    let ctx = TestContext::new();
    ctx.create_issue("leaf", IssueType::Task, "Leaf task");

    let result = run_impl(&ctx.db, "leaf");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_nonexistent() {
    let ctx = TestContext::new();

    let result = run_impl(&ctx.db, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_run_impl_with_blockers() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked")
        .blocks("blocker", "blocked");

    let result = run_impl(&ctx.db, "blocked");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_deep_hierarchy() {
    let ctx = TestContext::new();
    ctx.create_issue("l1", IssueType::Feature, "Level 1")
        .create_issue("l2", IssueType::Task, "Level 2")
        .create_issue("l3", IssueType::Task, "Level 3")
        .tracks("l1", "l2")
        .tracks("l2", "l3");

    let result = run_impl(&ctx.db, "l1");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_tracked_and_blocked() {
    let ctx = TestContext::new();
    ctx.create_issue("epic", IssueType::Epic, "Epic")
        .create_issue("feature", IssueType::Feature, "Feature under epic")
        .create_issue("dependent", IssueType::Feature, "Dependent feature")
        .tracks("epic", "feature")
        .blocks("epic", "dependent");

    // Epic has both tracked and blocking relationships
    let tracked = ctx.db.get_tracked("epic").unwrap();
    assert_eq!(tracked.len(), 1);
    assert_eq!(tracked[0], "feature");

    let blocking = ctx.db.get_blocking("epic").unwrap();
    assert_eq!(blocking.len(), 1);
    assert_eq!(blocking[0], "dependent");

    let result = run_impl(&ctx.db, "epic");
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_blocks_only() {
    let ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker task")
        .create_issue("dependent1", IssueType::Task, "Dependent 1")
        .create_issue("dependent2", IssueType::Task, "Dependent 2")
        .blocks("blocker", "dependent1")
        .blocks("blocker", "dependent2");

    // Blocker has only blocking relationships, no tracked
    let tracked = ctx.db.get_tracked("blocker").unwrap();
    assert!(tracked.is_empty());

    let blocking = ctx.db.get_blocking("blocker").unwrap();
    assert_eq!(blocking.len(), 2);

    let result = run_impl(&ctx.db, "blocker");
    assert!(result.is_ok());
}
