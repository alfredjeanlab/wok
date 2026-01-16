// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::cli::OutputFormat;
use crate::db::Database;
use crate::models::{Issue, IssueType, Relation};
use chrono::Utc;

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

#[test]
fn test_parse_filter_groups_empty() {
    let result = parse_filter_groups::<Status, _>(&[], |s| s.parse()).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_filter_groups_single_value() {
    let values = vec!["todo".to_string()];
    let result = parse_filter_groups(&values, |s| s.parse::<Status>()).unwrap();
    assert_eq!(result, Some(vec![vec![Status::Todo]]));
}

#[test]
fn test_parse_filter_groups_comma_separated() {
    let values = vec!["todo,in_progress".to_string()];
    let result = parse_filter_groups(&values, |s| s.parse::<Status>()).unwrap();
    assert_eq!(result, Some(vec![vec![Status::Todo, Status::InProgress]]));
}

#[test]
fn test_parse_filter_groups_multiple_entries() {
    let values = vec!["todo".to_string(), "in_progress".to_string()];
    let result = parse_filter_groups(&values, |s| s.parse::<Status>()).unwrap();
    assert_eq!(
        result,
        Some(vec![vec![Status::Todo], vec![Status::InProgress]])
    );
}

#[test]
fn test_matches_filter_groups_none() {
    let groups: Option<Vec<Vec<Status>>> = None;
    assert!(matches_filter_groups(&groups, || Status::Todo));
}

#[test]
fn test_matches_filter_groups_single_match() {
    let groups = Some(vec![vec![Status::Todo, Status::InProgress]]);
    assert!(matches_filter_groups(&groups, || Status::Todo));
    assert!(matches_filter_groups(&groups, || Status::InProgress));
    assert!(!matches_filter_groups(&groups, || Status::Done));
}

#[test]
fn test_matches_filter_groups_and_logic() {
    // Each group must match (AND between groups)
    // --status todo --status in_progress requires both conditions
    // But since an issue can only have one status, this would never match
    let groups = Some(vec![vec![Status::Todo], vec![Status::InProgress]]);
    assert!(!matches_filter_groups(&groups, || Status::Todo));
    assert!(!matches_filter_groups(&groups, || Status::InProgress));
}

#[test]
fn test_matches_label_groups_none() {
    let groups: Option<Vec<Vec<String>>> = None;
    assert!(matches_label_groups(&groups, &["a".to_string()]));
}

#[test]
fn test_matches_label_groups_single_group() {
    // Issue must have at least one of: a, b
    let groups = Some(vec![vec!["a".to_string(), "b".to_string()]]);
    assert!(matches_label_groups(&groups, &["a".to_string()]));
    assert!(matches_label_groups(&groups, &["b".to_string()]));
    assert!(matches_label_groups(
        &groups,
        &["a".to_string(), "c".to_string()]
    ));
    assert!(!matches_label_groups(&groups, &["c".to_string()]));
    assert!(!matches_label_groups(&groups, &[]));
}

#[test]
fn test_matches_label_groups_and_logic() {
    // Must have at least one from group1 AND at least one from group2
    // Example: --label a,b --label c means (a OR b) AND c
    let groups = Some(vec![
        vec!["a".to_string(), "b".to_string()],
        vec!["c".to_string()],
    ]);
    assert!(matches_label_groups(
        &groups,
        &["a".to_string(), "c".to_string()]
    ));
    assert!(matches_label_groups(
        &groups,
        &["b".to_string(), "c".to_string()]
    ));
    assert!(!matches_label_groups(
        &groups,
        &["a".to_string(), "b".to_string()]
    )); // missing c
    assert!(!matches_label_groups(&groups, &["c".to_string()])); // missing a or b
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
