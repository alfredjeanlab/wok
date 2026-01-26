// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::commands::lifecycle::{close_impl, done_impl, reopen_impl, resolve_reason, start_impl};
use crate::commands::testing::TestContext;
use crate::models::{IssueType, Relation};

// Test status transition validation logic (via Status methods)
#[test]
fn test_transitions_to_in_progress() {
    // Start: todo -> in_progress
    assert!(Status::Todo.can_transition_to(Status::InProgress));
    // Self-transition not allowed
    assert!(!Status::InProgress.can_transition_to(Status::InProgress));
    // Done/closed cannot go directly to in_progress (must use reopen to todo first)
    assert!(!Status::Done.can_transition_to(Status::InProgress));
    assert!(!Status::Closed.can_transition_to(Status::InProgress));
}

#[test]
fn test_reopen_transitions_to_todo() {
    // Reopen: in_progress/done/closed -> todo
    assert!(Status::InProgress.can_transition_to(Status::Todo));
    assert!(Status::Done.can_transition_to(Status::Todo));
    assert!(Status::Closed.can_transition_to(Status::Todo));
    // Self-transition not allowed
    assert!(!Status::Todo.can_transition_to(Status::Todo));
}

#[test]
fn test_done_valid_from_in_progress_and_todo() {
    assert!(Status::InProgress.can_transition_to(Status::Done));
    assert!(Status::Todo.can_transition_to(Status::Done)); // With reason
    assert!(!Status::Done.can_transition_to(Status::Done));
    assert!(!Status::Closed.can_transition_to(Status::Done));
}

#[test]
fn test_close_valid_from_todo_and_in_progress() {
    assert!(Status::Todo.can_transition_to(Status::Closed));
    assert!(Status::InProgress.can_transition_to(Status::Closed));
    assert!(!Status::Done.can_transition_to(Status::Closed));
    assert!(!Status::Closed.can_transition_to(Status::Closed));
}

#[test]
fn test_reopen_valid_from_done_and_closed() {
    assert!(Status::Done.can_transition_to(Status::Todo));
    assert!(Status::Closed.can_transition_to(Status::Todo));
}

// Test log_unblocked_events logic
#[test]
fn test_log_unblocked_when_blocker_completed() {
    let ctx = TestContext::new();

    // Create two issues: A blocks B
    ctx.create_issue_with_status("issue-a", IssueType::Task, "Issue A", Status::InProgress);
    ctx.create_issue_with_status("issue-b", IssueType::Task, "Issue B", Status::Todo);
    ctx.db
        .add_dependency("issue-a", "issue-b", Relation::Blocks)
        .unwrap();

    // Complete issue A
    ctx.db.update_issue_status("issue-a", Status::Done).unwrap();

    // Call log_unblocked_events
    log_unblocked_events(&ctx.db, &ctx.work_dir, &ctx.config, "issue-a").unwrap();

    // Check that an unblocked event was logged for issue B
    let events = ctx.db.get_events("issue-b").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Unblocked));
}

#[test]
fn test_no_unblocked_when_multiple_blockers() {
    let ctx = TestContext::new();

    // Create three issues: A and C both block B
    ctx.create_issue_with_status("issue-a", IssueType::Task, "Issue A", Status::InProgress);
    ctx.create_issue_with_status("issue-b", IssueType::Task, "Issue B", Status::Todo);
    ctx.create_issue_with_status("issue-c", IssueType::Task, "Issue C", Status::InProgress);
    ctx.db
        .add_dependency("issue-a", "issue-b", Relation::Blocks)
        .unwrap();
    ctx.db
        .add_dependency("issue-c", "issue-b", Relation::Blocks)
        .unwrap();

    // Complete issue A (but C still blocks B)
    ctx.db.update_issue_status("issue-a", Status::Done).unwrap();

    // Call log_unblocked_events
    log_unblocked_events(&ctx.db, &ctx.work_dir, &ctx.config, "issue-a").unwrap();

    // Check that NO unblocked event was logged for issue B (still blocked by C)
    let events = ctx.db.get_events("issue-b").unwrap();
    assert!(!events.iter().any(|e| e.action == Action::Unblocked));
}

#[test]
fn test_unblocked_after_all_blockers_done() {
    let ctx = TestContext::new();

    // Create three issues: A and C both block B
    ctx.create_issue_with_status("issue-a", IssueType::Task, "Issue A", Status::InProgress);
    ctx.create_issue_with_status("issue-b", IssueType::Task, "Issue B", Status::Todo);
    ctx.create_issue_with_status("issue-c", IssueType::Task, "Issue C", Status::InProgress);
    ctx.db
        .add_dependency("issue-a", "issue-b", Relation::Blocks)
        .unwrap();
    ctx.db
        .add_dependency("issue-c", "issue-b", Relation::Blocks)
        .unwrap();

    // Complete both A and C
    ctx.db.update_issue_status("issue-a", Status::Done).unwrap();
    log_unblocked_events(&ctx.db, &ctx.work_dir, &ctx.config, "issue-a").unwrap();

    ctx.db.update_issue_status("issue-c", Status::Done).unwrap();
    log_unblocked_events(&ctx.db, &ctx.work_dir, &ctx.config, "issue-c").unwrap();

    // Now B should have an unblocked event
    let events = ctx.db.get_events("issue-b").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Unblocked));
}

// Test error cases
#[test]
fn test_invalid_transition_error_format() {
    let err = Error::InvalidTransition {
        from: "todo".to_string(),
        to: "todo".to_string(),
        valid_targets: "in_progress, done (with reason), closed (with reason)".to_string(),
    };

    let msg = err.to_string();
    assert!(msg.contains("todo"));
    assert!(msg.contains("in_progress"));
}

// Tests for run_impl functions

#[test]
fn test_start_impl_from_todo() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Start test");

    let result = start_impl(&ctx.db, &ctx.config, &ctx.work_dir, &["test-1".to_string()]);

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::InProgress);
}

#[test]
fn test_start_impl_from_in_progress_fails() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Already started")
        .start_issue("test-1");

    let result = start_impl(&ctx.db, &ctx.config, &ctx.work_dir, &["test-1".to_string()]);

    assert!(result.is_err());
}

#[test]
fn test_start_impl_from_done_fails() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Completed task");

    let result = start_impl(&ctx.db, &ctx.config, &ctx.work_dir, &["test-1".to_string()]);

    assert!(result.is_err());
}

#[test]
fn test_done_impl_from_in_progress() {
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "Done test");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Done);
}

#[test]
fn test_done_impl_from_todo_requires_reason() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Todo task");

    // Set AI env var to ensure non-interactive mode (requires reason)
    let prev = std::env::var_os("CLAUDE_CODE");
    std::env::set_var("CLAUDE_CODE", "1");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    );

    match prev {
        Some(v) => std::env::set_var("CLAUDE_CODE", v),
        None => std::env::remove_var("CLAUDE_CODE"),
    }

    assert!(result.is_err());
}

#[test]
fn test_done_impl_from_todo_with_reason() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Todo task");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        Some("Already completed externally"),
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Done);
}

#[test]
fn test_done_impl_from_done_fails() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Already done");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_close_impl_from_todo() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Close test");

    let result = close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        "Won't fix",
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Closed);
}

#[test]
fn test_close_impl_from_in_progress() {
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "In progress task");

    let result = close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        "Requirements changed",
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Closed);
}

#[test]
fn test_close_impl_from_done_fails() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Done task");

    let result = close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        "Shouldn't work",
    );

    assert!(result.is_err());
}

#[test]
fn test_reopen_impl_from_done() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Completed task");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        Some("Found a bug"),
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Todo);
}

#[test]
fn test_reopen_impl_from_closed() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Closed task")
        .close_issue("test-1");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        Some("Actually needed"),
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Todo);
}

#[test]
fn test_reopen_impl_from_in_progress_succeeds() {
    // Reopen from in_progress works and doesn't require a reason
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "In progress task");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    );

    assert!(result.is_ok());
    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.status, Status::Todo);
}

#[test]
fn test_reopen_impl_from_todo_fails() {
    // Cannot reopen from todo - it's already in todo state
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Todo task");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    );

    assert!(result.is_err());
}

// === Reason Notes Tests ===

#[test]
fn test_close_creates_note() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        "duplicate of test-2",
    )
    .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "duplicate of test-2" && n.status == Status::Closed));
}

#[test]
fn test_done_with_reason_creates_note() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        Some("already completed upstream"),
    )
    .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "already completed upstream" && n.status == Status::Done));
}

#[test]
fn test_done_without_reason_no_note() {
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "Test task");

    done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        None,
    )
    .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    // Should have no notes created by done command (may have notes from other sources)
    assert!(!notes.iter().any(|n| n.status == Status::Done));
}

#[test]
fn test_reopen_creates_note() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Completed task");

    reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string()],
        Some("regression found in v2"),
    )
    .unwrap();

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "regression found in v2" && n.status == Status::Todo));
}

// === Batch Operations Tests ===

#[test]
fn test_start_impl_multiple_from_todo() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
    );

    assert!(result.is_ok());
    assert_eq!(
        ctx.db.get_issue("test-1").unwrap().status,
        Status::InProgress
    );
    assert_eq!(
        ctx.db.get_issue("test-2").unwrap().status,
        Status::InProgress
    );
}

#[test]
fn test_start_impl_fails_on_invalid_status() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2")
        .start_issue("test-2");

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
    );

    // First succeeds, second fails
    assert!(result.is_err());
    assert_eq!(
        ctx.db.get_issue("test-1").unwrap().status,
        Status::InProgress
    );
}

#[test]
fn test_done_impl_multiple_from_in_progress() {
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "Task 1");
    ctx.create_and_start("test-2", IssueType::Task, "Task 2");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
        None,
    );

    assert!(result.is_ok());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Done);
    assert_eq!(ctx.db.get_issue("test-2").unwrap().status, Status::Done);
}

#[test]
fn test_done_impl_multiple_with_reason() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
        Some("upstream"),
    );

    assert!(result.is_ok());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Done);
    assert_eq!(ctx.db.get_issue("test-2").unwrap().status, Status::Done);
}

#[test]
fn test_close_impl_multiple() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
        "duplicate",
    );

    assert!(result.is_ok());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Closed);
    assert_eq!(ctx.db.get_issue("test-2").unwrap().status, Status::Closed);
}

#[test]
fn test_reopen_impl_multiple() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Task 1");
    ctx.create_issue("test-2", IssueType::Task, "Task 2")
        .close_issue("test-2");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
        Some("regression"),
    );

    assert!(result.is_ok());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Todo);
    assert_eq!(ctx.db.get_issue("test-2").unwrap().status, Status::Todo);
}

// === resolve_reason Tests ===

#[test]
fn test_resolve_reason_with_explicit_reason() {
    let result = resolve_reason(Some("Manual close"), "Closed");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Manual close");
}

#[test]
fn test_resolve_reason_empty_explicit_fails() {
    let result = resolve_reason(Some("   "), "Closed");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[test]
fn test_resolve_reason_trims_whitespace() {
    let result = resolve_reason(Some("  some reason  "), "Closed");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "some reason");
}

#[test]
fn test_resolve_reason_without_reason_in_non_interactive() {
    // In tests, we're not in a TTY, so this simulates non-interactive mode
    // Set AI env var to ensure non-interactive
    let prev = std::env::var_os("CLAUDE_CODE");
    std::env::set_var("CLAUDE_CODE", "1");

    let result = resolve_reason(None, "Closed");

    match prev {
        Some(v) => std::env::set_var("CLAUDE_CODE", v),
        None => std::env::remove_var("CLAUDE_CODE"),
    }

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("required for agent"));
}

#[test]
fn test_resolve_reason_without_reason_in_ci() {
    let prev = std::env::var_os("CI");
    std::env::set_var("CI", "true");

    let result = resolve_reason(None, "Reopened");

    match prev {
        Some(v) => std::env::set_var("CI", v),
        None => std::env::remove_var("CI"),
    }

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("required for agent"));
}

// === BulkResult Tests ===

use super::BulkResult;

#[test]
fn test_bulk_result_default_is_success() {
    let result = BulkResult::default();
    assert!(result.is_success());
    assert_eq!(result.failure_count(), 0);
}

#[test]
fn test_bulk_result_with_unknown_ids() {
    let mut result = BulkResult::default();
    result.unknown_ids.push("test-1".to_string());
    assert!(!result.is_success());
    assert_eq!(result.failure_count(), 1);
}

#[test]
fn test_bulk_result_with_transition_failures() {
    let mut result = BulkResult::default();
    result
        .transition_failures
        .push(("test-1".to_string(), "reason".to_string()));
    assert!(!result.is_success());
    assert_eq!(result.failure_count(), 1);
}

#[test]
fn test_bulk_result_mixed_failures() {
    let mut result = BulkResult {
        success_count: 2,
        ..Default::default()
    };
    result.unknown_ids.push("test-1".to_string());
    result
        .transition_failures
        .push(("test-2".to_string(), "reason".to_string()));
    assert!(!result.is_success());
    assert_eq!(result.failure_count(), 2);
}

// === Partial Bulk Update Tests ===

#[test]
fn test_start_partial_update_with_unknown() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "unknown-123".to_string()],
    );

    // Should fail overall but test-1 should be transitioned
    assert!(result.is_err());
    assert_eq!(
        ctx.db.get_issue("test-1").unwrap().status,
        Status::InProgress
    );

    // Check error type
    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded,
            failed,
            unknown_ids,
            transition_failures,
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 1);
            assert_eq!(unknown_ids, vec!["unknown-123"]);
            assert!(transition_failures.is_empty());
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_start_partial_update_with_invalid_transition() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .start_issue("test-1"); // Already started
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "test-2".to_string()],
    );

    // test-1 fails (already started), test-2 succeeds
    assert!(result.is_err());
    assert_eq!(
        ctx.db.get_issue("test-2").unwrap().status,
        Status::InProgress
    );

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded,
            failed,
            unknown_ids,
            transition_failures,
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 1);
            assert!(unknown_ids.is_empty());
            assert_eq!(transition_failures.len(), 1);
            assert_eq!(transition_failures[0].0, "test-1");
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_start_partial_update_mixed_failures() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1")
        .start_issue("test-1"); // Already started
    ctx.create_issue("test-2", IssueType::Task, "Task 2");

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &[
            "test-1".to_string(),
            "test-2".to_string(),
            "unknown-999".to_string(),
        ],
    );

    // test-1 fails (invalid), test-2 succeeds, unknown-999 not found
    assert!(result.is_err());
    assert_eq!(
        ctx.db.get_issue("test-2").unwrap().status,
        Status::InProgress
    );

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded,
            failed,
            unknown_ids,
            transition_failures,
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 2);
            assert_eq!(unknown_ids, vec!["unknown-999"]);
            assert_eq!(transition_failures.len(), 1);
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_done_partial_update_with_unknown() {
    let ctx = TestContext::new();
    ctx.create_and_start("test-1", IssueType::Task, "Task 1");

    let result = done_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "unknown-123".to_string()],
        None,
    );

    assert!(result.is_err());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Done);

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded, failed, ..
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 1);
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_close_partial_update_with_unknown() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task 1");

    let result = close_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "unknown-123".to_string()],
        "duplicate",
    );

    assert!(result.is_err());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Closed);

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded, failed, ..
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 1);
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_reopen_partial_update_with_unknown() {
    let ctx = TestContext::new();
    ctx.create_completed("test-1", IssueType::Task, "Task 1");

    let result = reopen_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &["test-1".to_string(), "unknown-123".to_string()],
        Some("regression"),
    );

    assert!(result.is_err());
    assert_eq!(ctx.db.get_issue("test-1").unwrap().status, Status::Todo);

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded, failed, ..
        } => {
            assert_eq!(succeeded, 1);
            assert_eq!(failed, 1);
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}

#[test]
fn test_start_all_unknown_ids() {
    let ctx = TestContext::new();

    let result = start_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        &[
            "unknown-1".to_string(),
            "unknown-2".to_string(),
            "unknown-3".to_string(),
        ],
    );

    assert!(result.is_err());

    match result.unwrap_err() {
        Error::PartialBulkFailure {
            succeeded,
            failed,
            unknown_ids,
            ..
        } => {
            assert_eq!(succeeded, 0);
            assert_eq!(failed, 3);
            assert_eq!(unknown_ids.len(), 3);
        }
        _ => panic!("Expected PartialBulkFailure"),
    }
}
