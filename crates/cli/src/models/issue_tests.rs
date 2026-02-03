// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

// IssueType tests - using parameterized tests
#[parameterized(
    feature = { IssueType::Feature, "feature" },
    task = { IssueType::Task, "task" },
    bug = { IssueType::Bug, "bug" },
    chore = { IssueType::Chore, "chore" },
)]
fn test_issue_type_as_str(issue_type: IssueType, expected: &str) {
    assert_eq!(issue_type.as_str(), expected);
    assert_eq!(issue_type.to_string(), expected);
}

#[parameterized(
    feature_lower = { "feature", IssueType::Feature },
    task_lower = { "task", IssueType::Task },
    bug_lower = { "bug", IssueType::Bug },
    chore_lower = { "chore", IssueType::Chore },
    feature_upper = { "FEATURE", IssueType::Feature },
    task_mixed = { "Task", IssueType::Task },
    bug_upper = { "BUG", IssueType::Bug },
    chore_upper = { "CHORE", IssueType::Chore },
    chore_mixed = { "Chore", IssueType::Chore },
)]
fn test_issue_type_from_str_valid(input: &str, expected: IssueType) {
    assert_eq!(input.parse::<IssueType>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    epic = { "epic" },
)]
fn test_issue_type_from_str_invalid(input: &str) {
    assert!(input.parse::<IssueType>().is_err());
}

// Status tests - using parameterized tests
#[parameterized(
    todo = { Status::Todo, "todo" },
    in_progress = { Status::InProgress, "in_progress" },
    done = { Status::Done, "done" },
    closed = { Status::Closed, "closed" },
)]
fn test_status_as_str(status: Status, expected: &str) {
    assert_eq!(status.as_str(), expected);
    assert_eq!(status.to_string(), expected);
}

#[parameterized(
    todo_lower = { "todo", Status::Todo },
    in_progress_lower = { "in_progress", Status::InProgress },
    done_lower = { "done", Status::Done },
    closed_lower = { "closed", Status::Closed },
    todo_upper = { "TODO", Status::Todo },
    in_progress_upper = { "IN_PROGRESS", Status::InProgress },
)]
fn test_status_from_str_valid(input: &str, expected: Status) {
    assert_eq!(input.parse::<Status>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    open = { "open" },
)]
fn test_status_from_str_invalid(input: &str) {
    assert!(input.parse::<Status>().is_err());
}

// Valid status transitions (all non-self transitions are valid)
#[parameterized(
    todo_to_in_progress = { Status::Todo, Status::InProgress },
    todo_to_done = { Status::Todo, Status::Done },
    todo_to_closed = { Status::Todo, Status::Closed },
    in_progress_to_todo = { Status::InProgress, Status::Todo },
    in_progress_to_done = { Status::InProgress, Status::Done },
    in_progress_to_closed = { Status::InProgress, Status::Closed },
    done_to_todo = { Status::Done, Status::Todo },
    done_to_in_progress = { Status::Done, Status::InProgress },
    done_to_closed = { Status::Done, Status::Closed },
    closed_to_todo = { Status::Closed, Status::Todo },
    closed_to_in_progress = { Status::Closed, Status::InProgress },
    closed_to_done = { Status::Closed, Status::Done },
)]
fn test_status_transition_valid(from: Status, to: Status) {
    assert!(
        from.can_transition_to(to),
        "{} -> {} should be valid",
        from,
        to
    );
}

// Self-transitions are not valid (handled as idempotent at the command level)
#[parameterized(
    todo_to_todo = { Status::Todo, Status::Todo },
    in_progress_to_in_progress = { Status::InProgress, Status::InProgress },
    done_to_done = { Status::Done, Status::Done },
    closed_to_closed = { Status::Closed, Status::Closed },
)]
fn test_status_self_transition_invalid(from: Status, to: Status) {
    assert!(
        !from.can_transition_to(to),
        "{} -> {} should be invalid (self-transition)",
        from,
        to
    );
}

#[test]
fn test_status_valid_targets() {
    // Just verify the strings contain expected info
    let todo_targets = Status::Todo.valid_targets();
    assert!(todo_targets.contains("in_progress"));
    assert!(todo_targets.contains("closed"));

    let in_progress_targets = Status::InProgress.valid_targets();
    assert!(in_progress_targets.contains("todo"));
    assert!(in_progress_targets.contains("done"));
    assert!(in_progress_targets.contains("closed"));

    let done_targets = Status::Done.valid_targets();
    assert!(done_targets.contains("in_progress"));
    assert!(done_targets.contains("todo"));
    assert!(done_targets.contains("closed"));

    let closed_targets = Status::Closed.valid_targets();
    assert!(closed_targets.contains("in_progress"));
    assert!(closed_targets.contains("todo"));
    assert!(closed_targets.contains("done"));
}

// Issue tests
#[test]
fn test_issue_new() {
    let issue = Issue::new(
        "test-123".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    assert_eq!(issue.id, "test-123");
    assert_eq!(issue.issue_type, IssueType::Task);
    assert_eq!(issue.title, "Test issue");
    assert_eq!(issue.status, Status::Todo); // Default status
                                            // created_at and updated_at should be equal for new issues
    assert_eq!(issue.created_at, issue.updated_at);
}

#[test]
fn test_issue_new_different_types() {
    let feature = Issue::new("f-1".to_string(), IssueType::Feature, "Feature".to_string());
    let task = Issue::new("t-1".to_string(), IssueType::Task, "Task".to_string());
    let bug = Issue::new("b-1".to_string(), IssueType::Bug, "Bug".to_string());
    let chore = Issue::new("c-1".to_string(), IssueType::Chore, "Chore".to_string());

    assert_eq!(feature.issue_type, IssueType::Feature);
    assert_eq!(task.issue_type, IssueType::Task);
    assert_eq!(bug.issue_type, IssueType::Bug);
    assert_eq!(chore.issue_type, IssueType::Chore);
}

#[test]
fn test_issue_type_serde() {
    let chore = IssueType::Chore;
    let json = serde_json::to_string(&chore).unwrap();
    assert_eq!(json, "\"chore\"");
    let parsed: IssueType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, IssueType::Chore);
}

#[test]
fn status_converts_to_core_status() {
    assert_eq!(wk_core::Status::Todo, Status::Todo.into());
    assert_eq!(wk_core::Status::InProgress, Status::InProgress.into());
    assert_eq!(wk_core::Status::Done, Status::Done.into());
    assert_eq!(wk_core::Status::Closed, Status::Closed.into());
}
