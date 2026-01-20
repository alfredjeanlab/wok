// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use yare::parameterized;

// IssueType parsing tests
#[parameterized(
    feature_lower = { "feature", IssueType::Feature },
    task_lower = { "task", IssueType::Task },
    bug_lower = { "bug", IssueType::Bug },
    chore_lower = { "chore", IssueType::Chore },
    idea_lower = { "idea", IssueType::Idea },
    feature_upper = { "FEATURE", IssueType::Feature },
    idea_upper = { "IDEA", IssueType::Idea },
    idea_mixed = { "Idea", IssueType::Idea },
)]
fn issue_type_from_str_valid(input: &str, expected: IssueType) {
    assert_eq!(input.parse::<IssueType>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
)]
fn issue_type_from_str_invalid(input: &str) {
    assert!(input.parse::<IssueType>().is_err());
}

#[parameterized(
    feature = { IssueType::Feature, "feature" },
    task = { IssueType::Task, "task" },
    bug = { IssueType::Bug, "bug" },
    chore = { IssueType::Chore, "chore" },
    idea = { IssueType::Idea, "idea" },
)]
fn issue_type_as_str(issue_type: IssueType, expected: &str) {
    assert_eq!(issue_type.as_str(), expected);
}

// Status parsing tests
#[parameterized(
    todo = { "todo", Status::Todo },
    in_progress = { "in_progress", Status::InProgress },
    done = { "done", Status::Done },
    closed = { "closed", Status::Closed },
)]
fn status_from_str_valid(input: &str, expected: Status) {
    assert_eq!(input.parse::<Status>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
)]
fn status_from_str_invalid(input: &str) {
    assert!(input.parse::<Status>().is_err());
}

// Valid status transitions
#[parameterized(
    todo_to_in_progress = { Status::Todo, Status::InProgress },
    todo_to_done = { Status::Todo, Status::Done },
    todo_to_closed = { Status::Todo, Status::Closed },
    in_progress_to_todo = { Status::InProgress, Status::Todo },
    in_progress_to_done = { Status::InProgress, Status::Done },
    in_progress_to_closed = { Status::InProgress, Status::Closed },
    done_to_todo = { Status::Done, Status::Todo },
    closed_to_todo = { Status::Closed, Status::Todo },
)]
fn status_transition_valid(from: Status, to: Status) {
    assert!(
        from.can_transition_to(to),
        "{} -> {} should be valid",
        from,
        to
    );
}

// Invalid status transitions
#[parameterized(
    todo_to_todo = { Status::Todo, Status::Todo },
    in_progress_to_in_progress = { Status::InProgress, Status::InProgress },
    done_to_done = { Status::Done, Status::Done },
    closed_to_closed = { Status::Closed, Status::Closed },
    done_to_in_progress = { Status::Done, Status::InProgress },
    done_to_closed = { Status::Done, Status::Closed },
    closed_to_in_progress = { Status::Closed, Status::InProgress },
    closed_to_done = { Status::Closed, Status::Done },
)]
fn status_transition_invalid(from: Status, to: Status) {
    assert!(
        !from.can_transition_to(to),
        "{} -> {} should be invalid",
        from,
        to
    );
}

#[parameterized(
    todo = { Status::Todo, false },
    in_progress = { Status::InProgress, false },
    done = { Status::Done, true },
    closed = { Status::Closed, true },
)]
fn status_is_terminal(status: Status, expected: bool) {
    assert_eq!(status.is_terminal(), expected);
}

#[parameterized(
    todo = { Status::Todo, true },
    in_progress = { Status::InProgress, true },
    done = { Status::Done, false },
    closed = { Status::Closed, false },
)]
fn status_is_active(status: Status, expected: bool) {
    assert_eq!(status.is_active(), expected);
}

// Action parsing tests
#[parameterized(
    created = { "created", Action::Created },
    edited = { "edited", Action::Edited },
    started = { "started", Action::Started },
    stopped = { "stopped", Action::Stopped },
    done = { "done", Action::Done },
    closed = { "closed", Action::Closed },
    reopened = { "reopened", Action::Reopened },
    labeled = { "labeled", Action::Labeled },
    unlabeled = { "unlabeled", Action::Unlabeled },
    related = { "related", Action::Related },
    unrelated = { "unrelated", Action::Unrelated },
    noted = { "noted", Action::Noted },
    unblocked = { "unblocked", Action::Unblocked },
)]
fn action_from_str_valid(input: &str, expected: Action) {
    assert_eq!(input.parse::<Action>().unwrap(), expected);
}

#[test]
fn action_from_str_invalid() {
    assert!("invalid".parse::<Action>().is_err());
}

// Relation parsing tests
#[parameterized(
    blocks = { "blocks", Relation::Blocks },
    tracked_by = { "tracked_by", Relation::TrackedBy },
    tracks = { "tracks", Relation::Tracks },
)]
fn relation_from_str_valid(input: &str, expected: Relation) {
    assert_eq!(input.parse::<Relation>().unwrap(), expected);
}

#[test]
fn relation_from_str_invalid() {
    assert!("invalid".parse::<Relation>().is_err());
}

#[test]
fn event_builder_pattern() {
    let event = Event::new("test-123".to_string(), Action::Edited)
        .with_values(Some("old".to_string()), Some("new".to_string()))
        .with_reason(Some("because".to_string()));

    assert_eq!(event.issue_id, "test-123");
    assert_eq!(event.action, Action::Edited);
    assert_eq!(event.old_value, Some("old".to_string()));
    assert_eq!(event.new_value, Some("new".to_string()));
    assert_eq!(event.reason, Some("because".to_string()));
}

#[test]
fn issue_new() {
    let now = Utc::now();
    let issue = Issue::new(
        "test-123".to_string(),
        IssueType::Task,
        "Test".to_string(),
        now,
    );

    assert_eq!(issue.id, "test-123");
    assert_eq!(issue.issue_type, IssueType::Task);
    assert_eq!(issue.title, "Test");
    assert_eq!(issue.status, Status::Todo);
    assert_eq!(issue.created_at, now);
    assert_eq!(issue.updated_at, now);
    assert!(issue.last_status_hlc.is_none());
    assert!(issue.last_title_hlc.is_none());
    assert!(issue.last_type_hlc.is_none());
}

#[test]
fn issue_type_serialization() {
    let task = IssueType::Task;
    let json = serde_json::to_string(&task).unwrap();
    assert_eq!(json, "\"task\"");
    let parsed: IssueType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, task);
}

#[test]
fn idea_type_serialization() {
    let idea = IssueType::Idea;
    let json = serde_json::to_string(&idea).unwrap();
    assert_eq!(json, "\"idea\"");
    let parsed: IssueType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, idea);
}

#[test]
fn status_serialization() {
    let status = Status::InProgress;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"in_progress\"");
    let parsed: Status = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, status);
}

#[test]
fn event_with_timestamp() {
    let custom_time = chrono::DateTime::parse_from_rfc3339("2026-01-15T12:00:00Z")
        .unwrap()
        .to_utc();
    let event = Event::new("test-123".to_string(), Action::Started).with_timestamp(custom_time);

    assert_eq!(event.created_at, custom_time);
}

#[parameterized(
    todo = { Status::Todo, "in_progress, done (with reason), closed (with reason)" },
    in_progress = { Status::InProgress, "todo, done, closed (with reason)" },
    done = { Status::Done, "todo (with reason to reopen)" },
    closed = { Status::Closed, "todo (with reason to reopen)" },
)]
fn status_valid_targets(status: Status, expected: &str) {
    assert_eq!(status.valid_targets(), expected);
}

#[test]
fn issue_type_display() {
    assert_eq!(format!("{}", IssueType::Feature), "feature");
    assert_eq!(format!("{}", IssueType::Task), "task");
    assert_eq!(format!("{}", IssueType::Bug), "bug");
    assert_eq!(format!("{}", IssueType::Chore), "chore");
    assert_eq!(format!("{}", IssueType::Idea), "idea");
}

#[test]
fn status_display() {
    assert_eq!(format!("{}", Status::Todo), "todo");
    assert_eq!(format!("{}", Status::InProgress), "in_progress");
    assert_eq!(format!("{}", Status::Done), "done");
    assert_eq!(format!("{}", Status::Closed), "closed");
}

#[test]
fn action_display() {
    assert_eq!(format!("{}", Action::Created), "created");
    assert_eq!(format!("{}", Action::Edited), "edited");
    assert_eq!(format!("{}", Action::Started), "started");
    assert_eq!(format!("{}", Action::Stopped), "stopped");
    assert_eq!(format!("{}", Action::Done), "done");
    assert_eq!(format!("{}", Action::Closed), "closed");
    assert_eq!(format!("{}", Action::Reopened), "reopened");
    assert_eq!(format!("{}", Action::Labeled), "labeled");
    assert_eq!(format!("{}", Action::Unlabeled), "unlabeled");
    assert_eq!(format!("{}", Action::Related), "related");
    assert_eq!(format!("{}", Action::Unrelated), "unrelated");
    assert_eq!(format!("{}", Action::Noted), "noted");
    assert_eq!(format!("{}", Action::Unblocked), "unblocked");
}

#[test]
fn relation_display() {
    assert_eq!(format!("{}", Relation::Blocks), "blocks");
    assert_eq!(format!("{}", Relation::TrackedBy), "tracked_by");
    assert_eq!(format!("{}", Relation::Tracks), "tracks");
}
