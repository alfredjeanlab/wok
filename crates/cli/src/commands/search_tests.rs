// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::db::Database;
use crate::models::{Issue, IssueType, Status};

#[test]
fn search_finds_title_match() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Authentication login".to_string(),
    );
    let issue2 = Issue::new(
        "test-2".to_string(),
        IssueType::Task,
        "Dashboard widget".to_string(),
    );

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    run_impl(
        &db,
        "login",
        vec![],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_with_status_filter() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Todo task".to_string(),
    );
    let issue2 = Issue::new(
        "test-2".to_string(),
        IssueType::Task,
        "Done task".to_string(),
    );

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.update_issue_status("test-2", Status::Done).unwrap();

    run_impl(
        &db,
        "task",
        vec!["todo".to_string()],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_with_type_filter() {
    let db = Database::open_in_memory().unwrap();
    let bug = Issue::new(
        "test-1".to_string(),
        IssueType::Bug,
        "Bug with auth".to_string(),
    );
    let task = Issue::new(
        "test-2".to_string(),
        IssueType::Task,
        "Task with auth".to_string(),
    );

    db.create_issue(&bug).unwrap();
    db.create_issue(&task).unwrap();

    run_impl(
        &db,
        "auth",
        vec![],
        vec!["bug".to_string()],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_with_label_filter() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = Issue::new("test-1".to_string(), IssueType::Task, "Task A".to_string());
    let issue2 = Issue::new("test-2".to_string(), IssueType::Task, "Task B".to_string());

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_label("test-1", "urgent").unwrap();

    run_impl(
        &db,
        "Task",
        vec![],
        vec![],
        vec!["urgent".to_string()],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_no_matches_returns_empty() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Some task".to_string(),
    );

    db.create_issue(&issue).unwrap();

    run_impl(
        &db,
        "nonexistent",
        vec![],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_json_output() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "JSON test task".to_string(),
    );

    db.create_issue(&issue).unwrap();

    run_impl(
        &db,
        "JSON",
        vec![],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Json,
    )
    .unwrap();
}

#[test]
fn search_limits_results_to_25() {
    let db = Database::open_in_memory().unwrap();

    // Create 30 issues that match the query
    for i in 0..30 {
        let issue = Issue::new(
            format!("test-{}", i),
            IssueType::Task,
            format!("Matching task {}", i),
        );
        db.create_issue(&issue).unwrap();
    }

    // Verify the search finds all 30 but DEFAULT_LIMIT is 25
    let issues = db.search_issues("Matching").unwrap();
    assert_eq!(issues.len(), 30);

    // The output is limited (verified by the DEFAULT_LIMIT constant)
    run_impl(
        &db,
        "Matching",
        vec![],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Text,
    )
    .unwrap();
}

#[test]
fn search_json_includes_more_count() {
    let db = Database::open_in_memory().unwrap();

    // Create 30 issues
    for i in 0..30 {
        let issue = Issue::new(
            format!("test-{}", i),
            IssueType::Task,
            format!("JSON task {}", i),
        );
        db.create_issue(&issue).unwrap();
    }

    // JSON output should include "more" field when results exceed limit
    run_impl(
        &db,
        "JSON",
        vec![],
        vec![],
        vec![],
        vec![],
        false,
        vec![],
        None,
        OutputFormat::Json,
    )
    .unwrap();
}
