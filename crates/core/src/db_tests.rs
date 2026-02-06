// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hlc::Hlc;
use crate::issue::Action;
use chrono::Utc;

fn test_issue(id: &str, title: &str) -> Issue {
    Issue::new(
        id.to_string(),
        IssueType::Task,
        title.to_string(),
        Utc::now(),
    )
}

#[test]
fn create_and_get_issue() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");

    db.create_issue(&issue).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();

    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.title, "Test issue");
    assert_eq!(retrieved.status, Status::Todo);
    assert!(retrieved.closed_at.is_none());
    assert!(retrieved.last_status_hlc.is_none());
}

#[test]
fn issue_exists() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");

    assert!(!db.issue_exists("test-1").unwrap());
    db.create_issue(&issue).unwrap();
    assert!(db.issue_exists("test-1").unwrap());
}

#[test]
fn update_issue_status() {
    let mut db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_status("test-1", Status::InProgress)
        .unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::InProgress);
}

#[test]
fn update_issue_status_hlc() {
    let mut db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let hlc = Hlc::new(1000, 0, 1);
    db.update_issue_status_hlc("test-1", hlc).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.last_status_hlc, Some(hlc));
}

#[test]
fn update_issue_status_sets_closed_at() {
    let mut db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    // Transitioning to Done should set closed_at
    db.update_issue_status("test-1", Status::Done).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Done);
    assert!(retrieved.closed_at.is_some());

    // Reopening (back to Todo) should clear closed_at
    db.update_issue_status("test-1", Status::Todo).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Todo);
    assert!(retrieved.closed_at.is_none());

    // Transitioning to Closed should set closed_at
    db.update_issue_status("test-1", Status::Closed).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Closed);
    assert!(retrieved.closed_at.is_some());
}

#[test]
fn update_issue_title() {
    let mut db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Old title");
    db.create_issue(&issue).unwrap();

    db.update_issue_title("test-1", "New title").unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.title, "New title");
}

#[test]
fn update_issue_type() {
    let mut db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_type("test-1", IssueType::Bug).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.issue_type, IssueType::Bug);
}

#[test]
fn list_issues_filter_status() {
    let db = Database::open_in_memory().unwrap();

    let mut issue1 = test_issue("test-1", "Issue 1");
    issue1.status = Status::Todo;
    let mut issue2 = test_issue("test-2", "Issue 2");
    issue2.status = Status::InProgress;

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    let todos = db.list_issues(Some(Status::Todo), None, None).unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, "test-1");

    let in_progress = db
        .list_issues(Some(Status::InProgress), None, None)
        .unwrap();
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].id, "test-2");
}

#[test]
fn log_and_get_events() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let event = Event::new("test-1".to_string(), Action::Created);
    db.log_event(&event).unwrap();

    let events = db.get_events("test-1").unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, Action::Created);
}

#[test]
fn add_and_get_notes() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_note("test-1", Status::Todo, "First note").unwrap();
    db.add_note("test-1", Status::InProgress, "Second note")
        .unwrap();

    let notes = db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].content, "First note");
    assert_eq!(notes[1].content, "Second note");
}

#[test]
fn add_and_get_labels() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    db.add_label("test-1", "backend").unwrap();

    let labels = db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn remove_label() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    assert!(db.remove_label("test-1", "urgent").unwrap());
    assert!(!db.remove_label("test-1", "urgent").unwrap()); // Already removed

    let labels = db.get_labels("test-1").unwrap();
    assert!(labels.is_empty());
}

#[test]
fn add_and_get_dependencies() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();

    let deps = db.get_deps_from("test-1").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].from_id, "test-1");
    assert_eq!(deps[0].to_id, "test-2");
    assert_eq!(deps[0].relation, Relation::Blocks);
}

#[test]
fn self_dependency_error() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Issue 1");
    db.create_issue(&issue).unwrap();

    let result = db.add_dependency("test-1", "test-1", Relation::Blocks);
    assert!(matches!(result, Err(Error::SelfDependency)));
}

#[test]
fn cycle_detection() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    let issue3 = test_issue("test-3", "Issue 3");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();
    db.create_issue(&issue3).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();
    db.add_dependency("test-2", "test-3", Relation::Blocks)
        .unwrap();

    // This would create a cycle: test-3 -> test-1 -> test-2 -> test-3
    let result = db.add_dependency("test-3", "test-1", Relation::Blocks);
    assert!(matches!(result, Err(Error::CycleDetected)));
}

#[test]
fn get_blockers() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();

    let blockers = db.get_blockers("test-2").unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0], "test-1");
}

#[test]
fn issue_not_found() {
    let db = Database::open_in_memory().unwrap();
    let result = db.get_issue("nonexistent");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn get_all_labels() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    db.add_label("test-2", "backend").unwrap();

    let all_labels = db.get_all_labels().unwrap();
    assert_eq!(all_labels.len(), 2);
}
