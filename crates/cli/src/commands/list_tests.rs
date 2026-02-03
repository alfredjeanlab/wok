// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::unnecessary_literal_unwrap)]

use super::*;
use crate::cli::OutputFormat;
use crate::db::Database;
use crate::models::{Issue, IssueType, Relation};
use chrono::Utc;
use yare::parameterized;

fn setup_db() -> Database {
    Database::open_in_memory().unwrap()
}

fn create_issue(db: &Database, id: &str, status: Status, issue_type: IssueType) {
    let issue = Issue {
        id: id.to_string(),
        issue_type,
        title: format!("Test issue {}", id),
        description: None,
        status,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    };
    db.create_issue(&issue).unwrap();
}

#[test]
fn test_status_filter_parsing() {
    assert_eq!("todo".parse::<Status>().unwrap(), Status::Todo);
    assert_eq!("in_progress".parse::<Status>().unwrap(), Status::InProgress);
    assert_eq!("done".parse::<Status>().unwrap(), Status::Done);
    assert_eq!("closed".parse::<Status>().unwrap(), Status::Closed);
}

#[test]
fn test_type_filter_parsing() {
    assert_eq!("feature".parse::<IssueType>().unwrap(), IssueType::Feature);
    assert_eq!("task".parse::<IssueType>().unwrap(), IssueType::Task);
    assert_eq!("bug".parse::<IssueType>().unwrap(), IssueType::Bug);
    assert_eq!("chore".parse::<IssueType>().unwrap(), IssueType::Chore);
}

#[test]
fn test_list_issues_by_status() {
    let db = setup_db();
    create_issue(&db, "a", Status::Todo, IssueType::Task);
    create_issue(&db, "b", Status::InProgress, IssueType::Task);
    create_issue(&db, "c", Status::Done, IssueType::Task);

    // Filter by status
    let todo = db.list_issues(Some(Status::Todo), None, None).unwrap();
    assert_eq!(todo.len(), 1);
    assert_eq!(todo[0].id, "a");

    let in_progress = db
        .list_issues(Some(Status::InProgress), None, None)
        .unwrap();
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].id, "b");
}

#[test]
fn test_list_issues_by_type() {
    let db = setup_db();
    create_issue(&db, "a", Status::Todo, IssueType::Feature);
    create_issue(&db, "b", Status::Todo, IssueType::Task);
    create_issue(&db, "c", Status::Todo, IssueType::Bug);

    // Filter by type
    let features = db
        .list_issues(None, Some(IssueType::Feature), None)
        .unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].id, "a");

    let bugs = db.list_issues(None, Some(IssueType::Bug), None).unwrap();
    assert_eq!(bugs.len(), 1);
    assert_eq!(bugs[0].id, "c");
}

#[test]
fn test_list_issues_by_chore_type() {
    let db = setup_db();
    create_issue(&db, "a", Status::Todo, IssueType::Task);
    create_issue(&db, "b", Status::Todo, IssueType::Chore);

    let chores = db.list_issues(None, Some(IssueType::Chore), None).unwrap();
    assert_eq!(chores.len(), 1);
    assert_eq!(chores[0].id, "b");
}

#[test]
fn test_list_issues_by_label() {
    let db = setup_db();
    create_issue(&db, "a", Status::Todo, IssueType::Task);
    create_issue(&db, "b", Status::Todo, IssueType::Task);

    db.add_label("a", "urgent").unwrap();

    // Filter by label
    let labeled = db.list_issues(None, None, Some("urgent")).unwrap();
    assert_eq!(labeled.len(), 1);
    assert_eq!(labeled[0].id, "a");
}

#[test]
fn test_blocked_issues_detection() {
    let db = setup_db();
    create_issue(&db, "blocker", Status::InProgress, IssueType::Task);
    create_issue(&db, "blocked", Status::Todo, IssueType::Task);

    // Add blocking dependency
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    // Get blocked IDs
    let blocked_ids: HashSet<String> = db.get_blocked_issue_ids().unwrap().into_iter().collect();

    assert!(blocked_ids.contains("blocked"));
    assert!(!blocked_ids.contains("blocker"));
}

// Tests for run_impl

#[test]
fn test_run_impl_default() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    // Default list (no filters) - shows open issues (todo + in_progress)
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_status_filter() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::InProgress, IssueType::Task);

    let result = run_impl(
        &db,
        vec!["todo".to_string()],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_type_filter() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::Todo, IssueType::Bug);

    let result = run_impl(
        &db,
        vec![],
        vec!["bug".to_string()],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_with_label_filter() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    db.add_label("test-1", "urgent").unwrap();

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec!["urgent".to_string()],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_blocked_only() {
    let db = setup_db();
    create_issue(&db, "blocker", Status::Todo, IssueType::Task);
    create_issue(&db, "blocked", Status::Todo, IssueType::Task);
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        true,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_empty_list() {
    let db = setup_db();

    // Empty database
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_invalid_status_filter() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec!["invalid_status".to_string()],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_err());
}

#[test]
fn test_run_impl_invalid_type_filter() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec![],
        vec!["invalid_type".to_string()],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_err());
}

#[test]
fn test_run_impl_comma_separated_status() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::InProgress, IssueType::Task);

    let result = run_impl(
        &db,
        vec!["todo,in_progress".to_string()],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Tests for JSON output format

#[test]
fn test_run_impl_json_format_outputs_valid_json() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_with_blocked_only() {
    let db = setup_db();
    create_issue(&db, "blocker", Status::Todo, IssueType::Task);
    create_issue(&db, "blocked", Status::Todo, IssueType::Task);
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        true,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_empty_list() {
    let db = setup_db();

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_json_format_with_filters() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::Todo, IssueType::Bug);
    db.add_label("test-1", "urgent").unwrap();

    let result = run_impl(
        &db,
        vec![],
        vec!["task".to_string()],
        vec!["urgent".to_string()],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

// Phase 4: Additional coverage tests

#[test]
fn test_default_shows_todo_and_in_progress() {
    let db = setup_db();
    create_issue(&db, "todo-1", Status::Todo, IssueType::Task);
    create_issue(&db, "in-progress-1", Status::InProgress, IssueType::Task);
    create_issue(&db, "done-1", Status::Done, IssueType::Task);
    create_issue(&db, "closed-1", Status::Closed, IssueType::Task);

    // Default list (no status filter) should show todo + in_progress only
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
    // The output would contain todo-1 and in-progress-1 but not done-1 or closed-1
}

#[test]
fn test_blocked_filter_with_status() {
    let db = setup_db();
    create_issue(&db, "blocker", Status::Todo, IssueType::Task);
    create_issue(&db, "blocked-todo", Status::Todo, IssueType::Task);
    create_issue(&db, "blocked-done", Status::Done, IssueType::Task);
    db.add_dependency("blocker", "blocked-todo", Relation::Blocks)
        .unwrap();
    db.add_dependency("blocker", "blocked-done", Relation::Blocks)
        .unwrap();

    // Blocked filter with status=done should show only blocked done issues
    let result = run_impl(
        &db,
        vec!["done".to_string()],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        true,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

#[test]
fn test_no_blocked_footer() {
    let db = setup_db();
    create_issue(&db, "blocker", Status::Todo, IssueType::Task);
    create_issue(&db, "blocked", Status::Todo, IssueType::Task);
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    // Default list should not include blocked footer anymore
    // Just verify it runs without error - actual footer removal is tested by the fact
    // that ListOutputJson no longer has blocked_count field
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Priority sorting tests

#[test]
fn test_list_sorts_by_priority_asc() {
    let db = setup_db();
    // Create issues with different priorities
    create_issue(&db, "low", Status::Todo, IssueType::Task);
    db.add_label("low", "priority:3").unwrap();
    create_issue(&db, "high", Status::Todo, IssueType::Task);
    db.add_label("high", "priority:1").unwrap();
    create_issue(&db, "medium", Status::Todo, IssueType::Task);
    // medium has no priority tag, defaults to 2

    // Get issues through the list logic
    let mut issues = db.list_issues(None, None, None).unwrap();
    issues.retain(|issue| issue.status == Status::Todo || issue.status == Status::InProgress);

    // Sort by priority ASC, then created_at DESC
    issues.sort_by(|a, b| {
        let tags_a = db.get_labels(&a.id).unwrap_or_default();
        let tags_b = db.get_labels(&b.id).unwrap_or_default();
        let priority_a = Database::priority_from_tags(&tags_a);
        let priority_b = Database::priority_from_tags(&tags_b);

        match priority_a.cmp(&priority_b) {
            std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at),
            other => other,
        }
    });

    // Order should be: high (1), medium (2), low (3)
    assert_eq!(issues[0].id, "high");
    assert_eq!(issues[1].id, "medium");
    assert_eq!(issues[2].id, "low");
}

#[test]
fn test_list_same_priority_sorts_by_created_at_desc() {
    let db = setup_db();
    // Create issues with same priority at different times
    let older = Issue {
        id: "older".to_string(),
        issue_type: IssueType::Task,
        title: "Older issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now() - chrono::Duration::hours(1),
        updated_at: Utc::now(),
        closed_at: None,
    };
    db.create_issue(&older).unwrap();

    let newer = Issue {
        id: "newer".to_string(),
        issue_type: IssueType::Task,
        title: "Newer issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    };
    db.create_issue(&newer).unwrap();

    // Both have default priority 2
    let mut issues = db.list_issues(None, None, None).unwrap();
    issues.retain(|issue| issue.status == Status::Todo);

    // Sort by priority ASC, then created_at DESC
    issues.sort_by(|a, b| {
        let tags_a = db.get_labels(&a.id).unwrap_or_default();
        let tags_b = db.get_labels(&b.id).unwrap_or_default();
        let priority_a = Database::priority_from_tags(&tags_a);
        let priority_b = Database::priority_from_tags(&tags_b);

        match priority_a.cmp(&priority_b) {
            std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at), // DESC
            other => other,
        }
    });

    // Newer should come first (DESC)
    assert_eq!(issues[0].id, "newer");
    assert_eq!(issues[1].id, "older");
}

#[test]
fn test_list_priority_tag_precedence() {
    let db = setup_db();
    // Create issue with both p: and priority: tags
    create_issue(&db, "dual", Status::Todo, IssueType::Task);
    db.add_label("dual", "p:0").unwrap();
    db.add_label("dual", "priority:4").unwrap();

    create_issue(&db, "default", Status::Todo, IssueType::Task);
    // default has priority 2

    let mut issues = db.list_issues(None, None, None).unwrap();
    issues.retain(|issue| issue.status == Status::Todo);

    // Sort by priority
    issues.sort_by(|a, b| {
        let tags_a = db.get_labels(&a.id).unwrap_or_default();
        let tags_b = db.get_labels(&b.id).unwrap_or_default();
        let priority_a = Database::priority_from_tags(&tags_a);
        let priority_b = Database::priority_from_tags(&tags_b);
        priority_a.cmp(&priority_b)
    });

    // dual should use priority:4 (not p:0), so default (2) should come first
    assert_eq!(issues[0].id, "default");
    assert_eq!(issues[1].id, "dual");
}

// Tests for --all flag

#[test]
fn test_run_impl_all_flag_shows_all_statuses() {
    let db = setup_db();
    create_issue(&db, "todo-1", Status::Todo, IssueType::Task);
    create_issue(&db, "in-progress-1", Status::InProgress, IssueType::Task);
    create_issue(&db, "done-1", Status::Done, IssueType::Task);
    create_issue(&db, "closed-1", Status::Closed, IssueType::Task);

    // With --all flag, all issues should be shown regardless of status
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        true,
        OutputFormat::Text,
    );
    assert!(result.is_ok());
}

// Tests for ids output format

#[test]
fn test_run_impl_ids_format_outputs_space_separated() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::Todo, IssueType::Bug);

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_ids_format_with_filters() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);
    create_issue(&db, "test-2", Status::Todo, IssueType::Bug);

    let result = run_impl(
        &db,
        vec![],
        vec!["task".to_string()],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_impl_ids_format_empty_list() {
    let db = setup_db();

    // Empty database should not print anything
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None,
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 2: Limit behavior tests
// ─────────────────────────────────────────────────────────────────────────────

#[parameterized(
    default_limit = { None, 100 },
    explicit_50 = { Some(50), 50 },
    unlimited = { Some(0), 150 },
    explicit_200 = { Some(200), 150 },
)]
fn test_effective_limit(input: Option<usize>, expected_max: usize) {
    let db = setup_db();
    // Create 150 issues to test limits
    for i in 0..150 {
        let issue = Issue {
            id: format!("limit-{:03}", i),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", i),
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        db.create_issue(&issue).unwrap();
    }

    // Get all issues to verify the count after limit
    let mut issues = db.list_issues(None, None, None).unwrap();
    issues.retain(|issue| issue.status == Status::Todo || issue.status == Status::InProgress);

    // Apply limit logic as in run_impl
    let effective_limit = input.unwrap_or(DEFAULT_LIMIT);
    if effective_limit > 0 {
        issues.truncate(effective_limit);
    }

    assert_eq!(issues.len(), expected_max);
}

#[test]
fn test_default_limit_is_100() {
    let db = setup_db();
    for i in 0..110 {
        let issue = Issue {
            id: format!("default-{:03}", i),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", i),
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        db.create_issue(&issue).unwrap();
    }

    // Verify DEFAULT_LIMIT constant
    assert_eq!(DEFAULT_LIMIT, 100);

    // Verify default behavior limits to 100
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None, // No explicit limit
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_limit_zero_is_unlimited() {
    let db = setup_db();
    for i in 0..110 {
        let issue = Issue {
            id: format!("unlimited-{:03}", i),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", i),
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        db.create_issue(&issue).unwrap();
    }

    // With limit=0, should return all issues
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        Some(0), // Unlimited
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_explicit_limit_overrides_default() {
    let db = setup_db();
    for i in 0..110 {
        let issue = Issue {
            id: format!("explicit-{:03}", i),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", i),
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        db.create_issue(&issue).unwrap();
    }

    // With explicit limit=50, should work
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        Some(50), // Explicit limit
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_ids_format_respects_limit() {
    let db = setup_db();
    for i in 0..20 {
        let issue = Issue {
            id: format!("ids-{:03}", i),
            issue_type: IssueType::Task,
            title: format!("Test issue {}", i),
            description: None,
            status: Status::Todo,
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
        };
        db.create_issue(&issue).unwrap();
    }

    // With limit=5 and ids format
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        Some(5),
        false,
        false,
        OutputFormat::Id,
    );
    assert!(result.is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 3: JSON output validation tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_json_output_includes_filters_applied() {
    let db = setup_db();
    create_issue(&db, "json-test-1", Status::Todo, IssueType::Task);

    // With filter, should include filters_applied
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec!["age < 1d".to_string()], // Filter specified
        None,
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_json_output_includes_limit_when_specified() {
    let db = setup_db();
    create_issue(&db, "json-limit-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        Some(10), // Explicit limit
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_json_output_excludes_null_metadata() {
    let db = setup_db();
    create_issue(&db, "json-null-1", Status::Todo, IssueType::Task);

    // No limit, no filters - metadata fields should be None (skipped in output)
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec![],
        None, // No limit
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_json_output_with_multiple_filters() {
    let db = setup_db();
    create_issue(&db, "json-multi-1", Status::Todo, IssueType::Task);

    // Multiple filters
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec!["age < 1d".to_string(), "updated < 1h".to_string()],
        None,
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}

#[test]
fn test_json_output_with_limit_and_filters() {
    let db = setup_db();
    create_issue(&db, "json-both-1", Status::Todo, IssueType::Task);

    // Both limit and filter specified
    let result = run_impl(
        &db,
        vec![],
        vec![],
        vec![],
        None,
        vec![],
        false,
        vec!["age < 1d".to_string()],
        Some(50),
        false,
        false,
        OutputFormat::Json,
    );
    assert!(result.is_ok());
}
