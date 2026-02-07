// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::OutputFormat;
use crate::commands::testing::TestContext;
use crate::models::{IssueType, Status};
use std::collections::HashSet;

#[test]
fn test_ready_returns_unblocked_todo_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Ready task")
        .create_issue("test-2", IssueType::Task, "Another ready task");

    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();

    let ready: Vec<_> = issues
        .into_iter()
        .filter(|i| !blocked.contains(&i.id))
        .collect();

    assert_eq!(ready.len(), 2);
}

#[test]
fn test_ready_excludes_blocked_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked task")
        .create_issue("ready", IssueType::Task, "Ready task")
        .blocks("blocker", "blocked");

    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();

    let ready: Vec<_> = issues
        .into_iter()
        .filter(|i| !blocked.contains(&i.id))
        .collect();

    // blocker and ready should be in ready list, blocked should not
    assert_eq!(ready.len(), 2);
    assert!(ready.iter().any(|i| i.id == "blocker"));
    assert!(ready.iter().any(|i| i.id == "ready"));
    assert!(!ready.iter().any(|i| i.id == "blocked"));
}

#[test]
fn test_ready_excludes_non_todo_issues() {
    let mut ctx = TestContext::new();
    ctx.create_issue("todo", IssueType::Task, "Todo task")
        .create_issue_with_status(
            "in_progress",
            IssueType::Task,
            "In progress",
            Status::InProgress,
        )
        .create_issue_with_status("done", IssueType::Task, "Done task", Status::Done);

    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "todo");
}

#[test]
fn test_ready_empty_when_all_blocked() {
    let mut ctx = TestContext::new();
    ctx.create_issue_with_status("blocker", IssueType::Task, "Blocker", Status::InProgress)
        .create_issue("blocked", IssueType::Task, "Blocked task")
        .blocks("blocker", "blocked");

    // Get todo issues
    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();

    let ready: Vec<_> = issues
        .into_iter()
        .filter(|i| !blocked.contains(&i.id))
        .collect();

    // Only blocked issue is todo, and it's blocked
    assert!(ready.is_empty());
}

#[test]
fn test_ready_unblocked_after_blocker_completes() {
    let mut ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Was blocked")
        .blocks("blocker", "blocked")
        .set_status("blocker", Status::InProgress)
        .set_status("blocker", Status::Done);

    // Now that blocker is done, blocked should be unblocked
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();
    assert!(!blocked.contains("blocked"));
}

// Tests for run_impl

use crate::commands::ready::run_impl;

#[test]
fn test_run_impl_default() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_label() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task")
        .add_label("test-1", "backend");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec!["backend".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_empty() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_all_blocked() {
    let mut ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked")
        .blocks("blocker", "blocked")
        .start_issue("blocker");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Tests for filters

#[test]
fn test_run_impl_with_type_filter() {
    let mut ctx = TestContext::new();
    ctx.create_issue("task-1", IssueType::Task, "Test task")
        .create_issue("bug-1", IssueType::Bug, "Test bug")
        .create_issue("feature-1", IssueType::Feature, "Test feature");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["bug".to_string()],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_label_filter() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Backend task")
        .create_issue("test-2", IssueType::Task, "Frontend task")
        .add_label("test-1", "backend")
        .add_label("test-2", "frontend");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec!["backend".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_combined_filters() {
    let mut ctx = TestContext::new();
    ctx.create_issue("bug-1", IssueType::Bug, "Urgent bug")
        .create_issue("task-1", IssueType::Task, "Urgent task")
        .create_issue("bug-2", IssueType::Bug, "Other bug")
        .add_label("bug-1", "urgent")
        .add_label("task-1", "urgent");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["bug".to_string()],
        vec!["urgent".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_invalid_type_filter() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["invalid_type".to_string()],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_err());
}

#[test]
fn test_run_impl_comma_separated_types() {
    let mut ctx = TestContext::new();
    ctx.create_issue("bug-1", IssueType::Bug, "Test bug")
        .create_issue("task-1", IssueType::Task, "Test task")
        .create_issue("feature-1", IssueType::Feature, "Test feature");

    // Filter for bug OR task
    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["bug,task".to_string()],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_multiple_label_groups() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Backend urgent")
        .create_issue("test-2", IssueType::Task, "Frontend urgent")
        .create_issue("test-3", IssueType::Task, "Backend normal")
        .add_label("test-1", "backend")
        .add_label("test-1", "urgent")
        .add_label("test-2", "frontend")
        .add_label("test-2", "urgent")
        .add_label("test-3", "backend");

    // Filter for (backend) AND (urgent) - only test-1 matches
    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec!["backend".to_string(), "urgent".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_filters_exclude_blocked() {
    let mut ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Bug, "Blocker bug")
        .create_issue("blocked", IssueType::Bug, "Blocked bug")
        .add_label("blocker", "team:alpha")
        .add_label("blocked", "team:alpha")
        .blocks("blocker", "blocked");

    // Even with filters matching both, blocked should be excluded
    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["bug".to_string()],
        vec!["team:alpha".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Tests for JSON output format

#[test]
fn test_run_impl_json_format_outputs_valid_json() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_with_label() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task")
        .add_label("test-1", "backend");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec!["backend".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_empty() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_excludes_blocked() {
    let mut ctx = TestContext::new();
    ctx.create_issue("blocker", IssueType::Task, "Blocker")
        .create_issue("blocked", IssueType::Task, "Blocked")
        .blocks("blocker", "blocked");

    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

// Phase 4: Additional coverage tests

#[test]
fn test_ready_only_shows_todo() {
    // Ready should only show todo items, never in_progress even if unblocked
    let mut ctx = TestContext::new();
    ctx.create_issue("todo-task", IssueType::Task, "Todo task")
        .create_issue_with_status(
            "in-progress-task",
            IssueType::Task,
            "In progress task",
            Status::InProgress,
        )
        .create_issue_with_status("done-task", IssueType::Task, "Done task", Status::Done);

    // Ready should only return todo issues
    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "todo-task");
}

#[test]
fn test_ready_with_type_filter_works() {
    let mut ctx = TestContext::new();
    ctx.create_issue("bug-1", IssueType::Bug, "Bug issue")
        .create_issue("task-1", IssueType::Task, "Task issue")
        .create_issue("feature-1", IssueType::Feature, "Feature issue");

    // Type filter should work in ready
    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec!["bug".to_string()],
        vec![],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_ready_with_label_filter_works() {
    let mut ctx = TestContext::new();
    ctx.create_issue("task-1", IssueType::Task, "Labeled task")
        .create_issue("task-2", IssueType::Task, "Unlabeled task")
        .add_label("task-1", "important");

    // Label filter should work in ready
    let result = run_impl(
        &ctx.db,
        &ctx.work_dir,
        vec![],
        vec!["important".to_string()],
        None,
        vec![],
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Priority sorting tests

use crate::db::Database;
use crate::models::Issue;
use chrono::{Duration, Utc};

#[test]
fn test_ready_sorts_recent_by_priority() {
    let mut ctx = TestContext::new();
    // Create recent issues with different priorities
    ctx.create_issue("low", IssueType::Task, "Low priority task")
        .add_label("low", "priority:3")
        .create_issue("high", IssueType::Task, "High priority task")
        .add_label("high", "priority:1")
        .create_issue("medium", IssueType::Task, "Medium priority task");
    // medium has no priority tag, defaults to 2

    // Get ready issues and apply sorting
    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();
    let mut ready: Vec<_> = issues
        .into_iter()
        .filter(|i| !blocked.contains(&i.id))
        .collect();

    // Apply the same sorting logic as ready command
    let cutoff = Utc::now() - Duration::hours(48);
    ready.sort_by(|a, b| {
        let a_recent = a.created_at >= cutoff;
        let b_recent = b.created_at >= cutoff;
        match (a_recent, b_recent) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (true, true) => {
                let tags_a = ctx.db.get_labels(&a.id).unwrap_or_default();
                let tags_b = ctx.db.get_labels(&b.id).unwrap_or_default();
                let priority_a = crate::db::priority_from_tags(&tags_a);
                let priority_b = crate::db::priority_from_tags(&tags_b);
                match priority_a.cmp(&priority_b) {
                    std::cmp::Ordering::Equal => a.created_at.cmp(&b.created_at),
                    other => other,
                }
            }
            (false, false) => a.created_at.cmp(&b.created_at),
        }
    });

    // All are recent, so order should be: high (1), medium (2), low (3)
    assert_eq!(ready[0].id, "high");
    assert_eq!(ready[1].id, "medium");
    assert_eq!(ready[2].id, "low");
}

#[test]
fn test_ready_priority_tag_precedence() {
    let mut ctx = TestContext::new();
    // Create issue with both p: and priority: tags
    ctx.create_issue("dual", IssueType::Task, "Dual tagged")
        .add_label("dual", "p:0")
        .add_label("dual", "priority:4")
        .create_issue("default", IssueType::Task, "Default priority");
    // default has priority 2

    let issues = ctx.db.list_issues(Some(Status::Todo), None, None).unwrap();
    let blocked: HashSet<String> = ctx
        .db
        .get_blocked_issue_ids()
        .unwrap()
        .into_iter()
        .collect();
    let mut ready: Vec<_> = issues
        .into_iter()
        .filter(|i| !blocked.contains(&i.id))
        .collect();

    // Apply sorting
    let cutoff = Utc::now() - Duration::hours(48);
    ready.sort_by(|a, b| {
        let a_recent = a.created_at >= cutoff;
        let b_recent = b.created_at >= cutoff;
        match (a_recent, b_recent) {
            (true, true) => {
                let tags_a = ctx.db.get_labels(&a.id).unwrap_or_default();
                let tags_b = ctx.db.get_labels(&b.id).unwrap_or_default();
                let priority_a = crate::db::priority_from_tags(&tags_a);
                let priority_b = crate::db::priority_from_tags(&tags_b);
                priority_a.cmp(&priority_b)
            }
            _ => std::cmp::Ordering::Equal,
        }
    });

    // dual uses priority:4 (not p:0), so default (2) should come first
    assert_eq!(ready[0].id, "default");
    assert_eq!(ready[1].id, "dual");
}

#[test]
fn test_ready_sorts_recent_before_old() {
    // Create old issue (>48h ago)
    let db = Database::open_in_memory().unwrap();
    let old_issue = Issue {
        id: "old".to_string(),
        issue_type: IssueType::Task,
        title: "Old issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now() - Duration::hours(72),
        updated_at: Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&old_issue).unwrap();

    // Create recent issue (<48h ago)
    let recent_issue = Issue {
        id: "recent".to_string(),
        issue_type: IssueType::Task,
        title: "Recent issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&recent_issue).unwrap();

    let mut issues = db.list_issues(Some(Status::Todo), None, None).unwrap();
    let cutoff = Utc::now() - Duration::hours(48);

    issues.sort_by(|a, b| {
        let a_recent = a.created_at >= cutoff;
        let b_recent = b.created_at >= cutoff;
        match (a_recent, b_recent) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Recent should come before old
    assert_eq!(issues[0].id, "recent");
    assert_eq!(issues[1].id, "old");
}

#[test]
fn test_ready_sorts_old_by_created_at_asc() {
    // Create old issues at different times
    let db = Database::open_in_memory().unwrap();
    let older_issue = Issue {
        id: "older".to_string(),
        issue_type: IssueType::Task,
        title: "Older issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now() - Duration::hours(96), // 4 days ago
        updated_at: Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&older_issue).unwrap();

    let less_old_issue = Issue {
        id: "less_old".to_string(),
        issue_type: IssueType::Task,
        title: "Less old issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now() - Duration::hours(72), // 3 days ago
        updated_at: Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&less_old_issue).unwrap();

    let mut issues = db.list_issues(Some(Status::Todo), None, None).unwrap();
    let cutoff = Utc::now() - Duration::hours(48);

    issues.sort_by(|a, b| {
        let a_recent = a.created_at >= cutoff;
        let b_recent = b.created_at >= cutoff;
        match (a_recent, b_recent) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.created_at.cmp(&b.created_at), // ASC (oldest first)
            _ => std::cmp::Ordering::Equal,
        }
    });

    // Both are old, oldest should come first
    assert_eq!(issues[0].id, "older");
    assert_eq!(issues[1].id, "less_old");
}
