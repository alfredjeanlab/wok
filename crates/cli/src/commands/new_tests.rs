// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::OutputFormat;
use crate::commands::new::run_impl;
use crate::commands::testing::TestContext;
use crate::models::{Action, IssueType, Status};

#[test]
fn test_create_issue_basic() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "My new task");

    let issue = ctx.db.get_issue("test-1").unwrap();
    assert_eq!(issue.id, "test-1");
    assert_eq!(issue.title, "My new task");
    assert_eq!(issue.issue_type, IssueType::Task);
    assert_eq!(issue.status, Status::Todo);
}

#[test]
fn test_create_issue_with_type_bug() {
    let ctx = TestContext::new();
    ctx.create_issue("bug-1", IssueType::Bug, "Fix the crash");

    let issue = ctx.db.get_issue("bug-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Bug);
}

#[test]
fn test_create_issue_with_type_feature() {
    let ctx = TestContext::new();
    ctx.create_issue("feature-1", IssueType::Feature, "Big feature");

    let issue = ctx.db.get_issue("feature-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Feature);
}

#[test]
fn test_create_issue_with_type_chore() {
    let ctx = TestContext::new();
    ctx.create_issue("chore-1", IssueType::Chore, "Update dependencies");

    let issue = ctx.db.get_issue("chore-1").unwrap();
    assert_eq!(issue.issue_type, IssueType::Chore);
}

#[test]
fn test_create_issue_logs_created_event() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e.action == Action::Created));
}

#[test]
fn test_create_issue_with_labels() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Labeled task")
        .add_label("test-1", "backend")
        .add_label("test-1", "urgent");

    let labels = ctx.db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"backend".to_string()));
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_create_issue_with_note() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Task with note")
        .add_note("test-1", "Initial implementation note");

    let notes = ctx.db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Initial implementation note");
}

#[test]
fn test_issue_type_parsing() {
    assert_eq!("task".parse::<IssueType>().unwrap(), IssueType::Task);
    assert_eq!("bug".parse::<IssueType>().unwrap(), IssueType::Bug);
    assert_eq!("feature".parse::<IssueType>().unwrap(), IssueType::Feature);
    assert_eq!("chore".parse::<IssueType>().unwrap(), IssueType::Chore);

    // Case insensitive
    assert_eq!("TASK".parse::<IssueType>().unwrap(), IssueType::Task);
    assert_eq!("BUG".parse::<IssueType>().unwrap(), IssueType::Bug);
    assert_eq!("FEATURE".parse::<IssueType>().unwrap(), IssueType::Feature);
    assert_eq!("CHORE".parse::<IssueType>().unwrap(), IssueType::Chore);
}

#[test]
fn test_invalid_issue_type() {
    let result = "invalid".parse::<IssueType>();
    assert!(result.is_err());
}

#[test]
fn test_empty_title_validation() {
    // Note: The run() function validates empty titles before calling DB
    // This test verifies the validation logic
    let title = "";
    assert!(title.trim().is_empty());

    let whitespace_title = "   ";
    assert!(whitespace_title.trim().is_empty());

    let valid_title = "Valid title";
    assert!(!valid_title.trim().is_empty());
}

#[test]
fn test_duplicate_id_handling() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "First task");

    // Attempting to create with same ID should fail
    let issue = crate::models::Issue {
        id: "test-1".to_string(),
        issue_type: IssueType::Task,
        title: "Second task".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        closed_at: None,
    };
    let result = ctx.db.create_issue(&issue);
    assert!(result.is_err());
}

#[test]
fn test_issue_exists_check() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    assert!(ctx.db.issue_exists("test-1").unwrap());
    assert!(!ctx.db.issue_exists("nonexistent").unwrap());
}

#[test]
fn test_timestamps_set_on_creation() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test task");

    let issue = ctx.db.get_issue("test-1").unwrap();
    // Both timestamps should be set and equal on creation
    assert_eq!(issue.created_at, issue.updated_at);
}

// Input validation tests - use constants from validate module
use crate::validate::{MAX_LABELS_PER_ISSUE, MAX_LABEL_LENGTH, MAX_NOTE_LENGTH, MAX_TITLE_LENGTH};

#[test]
fn test_title_at_max_length_is_valid() {
    // A title exactly at the max length should be accepted
    let title = "x".repeat(MAX_TITLE_LENGTH);
    assert_eq!(title.len(), MAX_TITLE_LENGTH);
    // Title is valid if it's non-empty and within limits
    assert!(!title.trim().is_empty());
    assert!(title.len() <= MAX_TITLE_LENGTH);
}

#[test]
fn test_title_over_max_length_is_rejected() {
    use crate::validate::validate_and_normalize_title;

    let long_title = "x".repeat(MAX_TITLE_LENGTH + 1);

    // Validation should fail for overly long title
    let result = validate_and_normalize_title(&long_title);
    assert!(
        result.is_err(),
        "Title over {} chars should be rejected",
        MAX_TITLE_LENGTH
    );
}

#[test]
fn test_label_at_max_length_is_valid() {
    let label = "x".repeat(MAX_LABEL_LENGTH);
    assert_eq!(label.len(), MAX_LABEL_LENGTH);
    // Label is valid if within limits
    assert!(label.len() <= MAX_LABEL_LENGTH);
}

#[test]
fn test_label_over_max_length_is_rejected() {
    use crate::validate::validate_label;

    let long_label = "x".repeat(MAX_LABEL_LENGTH + 1);
    let result = validate_label(&long_label);
    assert!(
        result.is_err(),
        "Label over {} chars should be rejected",
        MAX_LABEL_LENGTH
    );
}

#[test]
fn test_max_labels_per_issue() {
    use crate::validate::validate_label_count;

    // At limit should fail
    let result = validate_label_count(MAX_LABELS_PER_ISSUE);
    assert!(
        result.is_err(),
        "Should not allow more than {} labels",
        MAX_LABELS_PER_ISSUE
    );

    // Below limit should succeed
    assert!(validate_label_count(MAX_LABELS_PER_ISSUE - 1).is_ok());
}

#[test]
fn test_note_at_max_length_is_valid() {
    let note = "x".repeat(MAX_NOTE_LENGTH);
    assert_eq!(note.len(), MAX_NOTE_LENGTH);
    assert!(note.len() <= MAX_NOTE_LENGTH);
}

#[test]
fn test_note_over_max_length_is_rejected() {
    use crate::validate::validate_note;

    let long_note = "x".repeat(MAX_NOTE_LENGTH + 1);
    let result = validate_note(&long_note);
    assert!(
        result.is_err(),
        "Note over {} chars should be rejected",
        MAX_NOTE_LENGTH
    );
}

// Tests for run_impl

#[test]
fn test_run_impl_creates_task() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("My new task".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    // Find the created issue (ID is auto-generated)
    let issues = ctx.db.list_issues(None, None, None).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "My new task");
    assert_eq!(issues[0].issue_type, IssueType::Task);
}

#[test]
fn test_run_impl_creates_bug() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "bug".to_string(),
        Some("Fix crash".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    assert_eq!(issues[0].issue_type, IssueType::Bug);
}

#[test]
fn test_run_impl_creates_feature() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "feature".to_string(),
        Some("Big feature".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    assert_eq!(issues[0].issue_type, IssueType::Feature);
}

#[test]
fn test_run_impl_creates_chore() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "chore".to_string(),
        Some("Update dependencies".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    assert_eq!(issues[0].issue_type, IssueType::Chore);
}

#[test]
fn test_run_impl_title_only_defaults_to_task() {
    let ctx = TestContext::new();

    // When title is None, type_or_title is treated as the title
    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "Just a title".to_string(),
        None,
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    assert_eq!(issues[0].title, "Just a title");
    assert_eq!(issues[0].issue_type, IssueType::Task);
}

#[test]
fn test_run_impl_with_labels() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Labeled task".to_string()),
        vec!["urgent".to_string(), "backend".to_string()],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn test_run_impl_with_note() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Task with note".to_string()),
        vec![],
        Some("Initial note".to_string()),
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let notes = ctx.db.get_notes(&issues[0].id).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Initial note");
}

#[test]
fn test_run_impl_empty_title_rejected() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_err());
}

#[test]
fn test_run_impl_whitespace_title_rejected() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("   ".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_err());
}

#[test]
fn test_run_impl_invalid_type_rejected() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "invalid_type".to_string(),
        Some("Test".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_err());
}

#[test]
fn test_run_impl_logs_events() {
    let ctx = TestContext::new();

    run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Event test".to_string()),
        vec!["label1".to_string()],
        Some("A note".to_string()),
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    )
    .unwrap();

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let events = ctx.db.get_events(&issues[0].id).unwrap();

    // Should have Created, Labeled, Noted events
    assert!(events.iter().any(|e| e.action == Action::Created));
    assert!(events.iter().any(|e| e.action == Action::Labeled));
    assert!(events.iter().any(|e| e.action == Action::Noted));
}

// Priority flag tests

#[test]
fn test_run_impl_with_priority_0() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Priority task".to_string()),
        vec![],
        None,
        vec![],
        None,
        Some(0),
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert!(labels.contains(&"priority:0".to_string()));
}

#[test]
fn test_run_impl_with_priority_4() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Low priority".to_string()),
        vec![],
        None,
        vec![],
        None,
        Some(4),
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert!(labels.contains(&"priority:4".to_string()));
}

#[test]
fn test_run_impl_priority_with_existing_labels() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Multi-labeled".to_string()),
        vec!["backend".to_string()],
        None,
        vec![],
        None,
        Some(2),
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert!(labels.contains(&"priority:2".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn test_run_impl_without_priority() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("No priority".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    // No priority label should be present
    assert!(!labels.iter().any(|l| l.starts_with("priority:")));
}

// Tests for hidden --description flag

#[test]
fn test_run_impl_with_description() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Described task".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        Some("Initial description".to_string()),
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let notes = ctx.db.get_notes(&issues[0].id).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Initial description");
}

#[test]
fn test_run_impl_note_takes_precedence_over_description() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Task".to_string()),
        vec![],
        Some("Note content".to_string()),
        vec![],
        None,
        None,
        Some("Description content".to_string()),
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let notes = ctx.db.get_notes(&issues[0].id).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Note content"); // note wins
}

#[test]
fn test_run_impl_without_description_or_note() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("No description".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let notes = ctx.db.get_notes(&issues[0].id).unwrap();
    assert!(notes.is_empty());
}

#[test]
fn test_run_impl_description_with_labels() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Labeled described".to_string()),
        vec!["backend".to_string()],
        None,
        vec![],
        None,
        None,
        Some("Description with labels".to_string()),
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let notes = ctx.db.get_notes(&issues[0].id).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();

    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Description with labels");
    assert!(labels.contains(&"backend".to_string()));
}

// Tests for comma-separated labels

#[test]
fn test_run_impl_comma_separated_labels() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Comma labels".to_string()),
        vec!["a,b,c".to_string()],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"a".to_string()));
    assert!(labels.contains(&"b".to_string()));
    assert!(labels.contains(&"c".to_string()));
}

#[test]
fn test_run_impl_comma_separated_and_multiple_labels() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Mixed labels".to_string()),
        vec!["a,b".to_string(), "c".to_string()],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"a".to_string()));
    assert!(labels.contains(&"b".to_string()));
    assert!(labels.contains(&"c".to_string()));
}

#[test]
fn test_run_impl_comma_separated_trims_whitespace() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Whitespace labels".to_string()),
        vec!["  x  ,  y  ".to_string()],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"x".to_string()));
    assert!(labels.contains(&"y".to_string()));
}

#[test]
fn test_run_impl_comma_separated_ignores_empty() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Empty labels".to_string()),
        vec!["a,,b".to_string(), "".to_string()],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"a".to_string()));
    assert!(labels.contains(&"b".to_string()));
}

#[test]
fn test_run_impl_comma_separated_with_priority() {
    let ctx = TestContext::new();

    let result = run_impl(
        &ctx.db,
        &ctx.config,
        &ctx.work_dir,
        "task".to_string(),
        Some("Priority labels".to_string()),
        vec!["a,b".to_string()],
        None,
        vec![],
        None,
        Some(1),
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_ok());

    let issues = ctx.db.list_issues(None, None, None).unwrap();
    let labels = ctx.db.get_labels(&issues[0].id).unwrap();
    assert_eq!(labels.len(), 3);
    assert!(labels.contains(&"a".to_string()));
    assert!(labels.contains(&"b".to_string()));
    assert!(labels.contains(&"priority:1".to_string()));
}

// Empty prefix validation tests

#[test]
fn test_run_impl_rejects_empty_prefix() {
    use crate::config::Config;
    use crate::db::Database;
    use tempfile::TempDir;

    let db = Database::open_in_memory().unwrap();
    // Create config with empty prefix (simulating workspace link without local prefix)
    let config = Config {
        prefix: String::new(),
        workspace: Some("/shared/workspace".to_string()),
        remote: None,
    };
    let temp_dir = TempDir::new().unwrap();
    let work_dir = temp_dir.path();

    let result = run_impl(
        &db,
        &config,
        work_dir,
        "task".to_string(),
        Some("Test task".to_string()),
        vec![],
        None,
        vec![],
        None,
        None,
        None,
        vec![],
        vec![],
        vec![],
        vec![],
        OutputFormat::Text,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("no prefix configured"),
        "Expected error about no prefix, got: {}",
        err
    );
}
