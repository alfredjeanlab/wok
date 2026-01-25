// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

#[test]
fn test_create_and_get_issue() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1234".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();
    let retrieved = db.get_issue("test-1234").unwrap();

    assert_eq!(retrieved.id, issue.id);
    assert_eq!(retrieved.title, issue.title);
    assert_eq!(retrieved.issue_type, issue.issue_type);
    assert_eq!(retrieved.status, Status::Todo);
}

#[test]
fn test_update_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1234".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();
    db.update_issue_status("test-1234", Status::InProgress)
        .unwrap();

    let retrieved = db.get_issue("test-1234").unwrap();
    assert_eq!(retrieved.status, Status::InProgress);
}

#[test]
fn test_issue_not_found() {
    let db = Database::open_in_memory().unwrap();
    let result = db.get_issue("nonexistent");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn test_chore_type_roundtrip() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "chore-1234".to_string(),
        IssueType::Chore,
        "Update dependencies".to_string(),
    );

    db.create_issue(&issue).unwrap();
    let retrieved = db.get_issue("chore-1234").unwrap();

    assert_eq!(retrieved.id, issue.id);
    assert_eq!(retrieved.title, issue.title);
    assert_eq!(retrieved.issue_type, IssueType::Chore);
    assert_eq!(retrieved.status, Status::Todo);
}

// Priority extraction tests

#[test]
fn priority_from_tags_numeric() {
    assert_eq!(Database::priority_from_tags(&["priority:0".into()]), 0);
    assert_eq!(Database::priority_from_tags(&["priority:1".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["priority:2".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:3".into()]), 3);
    assert_eq!(Database::priority_from_tags(&["priority:4".into()]), 4);
}

#[test]
fn priority_from_tags_named() {
    assert_eq!(
        Database::priority_from_tags(&["priority:highest".into()]),
        0
    );
    assert_eq!(Database::priority_from_tags(&["priority:high".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["priority:medium".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:med".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:low".into()]), 3);
    assert_eq!(Database::priority_from_tags(&["priority:lowest".into()]), 4);
}

#[test]
fn priority_from_tags_prefers_priority_over_p() {
    // priority: tag should take precedence over p: tag
    let tags = vec!["p:0".into(), "priority:4".into()];
    assert_eq!(Database::priority_from_tags(&tags), 4);

    // Even if p: comes after priority:
    let tags2 = vec!["priority:3".into(), "p:1".into()];
    assert_eq!(Database::priority_from_tags(&tags2), 3);
}

#[test]
fn priority_from_tags_default() {
    // Empty tags should default to 2 (medium)
    assert_eq!(Database::priority_from_tags(&[]), 2);

    // Unrelated tags should default to 2
    assert_eq!(Database::priority_from_tags(&["unrelated".into()]), 2);
    assert_eq!(
        Database::priority_from_tags(&["team:backend".into(), "urgent".into()]),
        2
    );
}

#[test]
fn priority_from_tags_p_fallback() {
    // p: tag should work when no priority: tag exists
    assert_eq!(Database::priority_from_tags(&["p:0".into()]), 0);
    assert_eq!(Database::priority_from_tags(&["p:1".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["p:4".into()]), 4);
}

#[test]
fn priority_from_tags_invalid_values_ignored() {
    // Invalid values should be ignored, falling through to default
    assert_eq!(
        Database::priority_from_tags(&["priority:invalid".into()]),
        2
    );
    assert_eq!(Database::priority_from_tags(&["priority:5".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:-1".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:".into()]), 2);
}

#[test]
fn priority_from_tags_first_valid_wins() {
    // First valid priority: tag should win
    let tags = vec!["priority:1".into(), "priority:4".into()];
    assert_eq!(Database::priority_from_tags(&tags), 1);
}

#[test]
fn priority_from_tags_with_other_tags() {
    // Priority should be extracted correctly when mixed with other tags
    let tags = vec!["team:backend".into(), "priority:0".into(), "urgent".into()];
    assert_eq!(Database::priority_from_tags(&tags), 0);
}

// Search tests

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

    let results = db.search_issues("login").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");
}

#[test]
fn search_finds_description_match() {
    let db = Database::open_in_memory().unwrap();
    let mut issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Generic task".to_string(),
    );
    issue.description = Some("Implement OAuth2 flow".to_string());

    db.create_issue(&issue).unwrap();

    let results = db.search_issues("OAuth2").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");
}

#[test]
fn search_finds_label_match() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Important task".to_string(),
    );

    db.create_issue(&issue).unwrap();
    db.add_label("test-1", "priority:high").unwrap();

    let results = db.search_issues("priority:high").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");
}

#[test]
fn search_is_case_insensitive() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Authentication Module".to_string(),
    );

    db.create_issue(&issue).unwrap();

    let results = db.search_issues("authentication").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");
}

#[test]
fn search_with_no_matches_returns_empty() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Some task".to_string(),
    );

    db.create_issue(&issue).unwrap();

    let results = db.search_issues("nonexistent").unwrap();
    assert!(results.is_empty());
}

#[test]
fn search_with_special_characters() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-1".to_string(),
        IssueType::Task,
        "Test with % and _".to_string(),
    );

    db.create_issue(&issue).unwrap();

    // Search with special characters should find the exact match
    let results = db.search_issues("%").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");

    let results2 = db.search_issues("_").unwrap();
    assert_eq!(results2.len(), 1);
    assert_eq!(results2[0].id, "test-1");
}

// resolve_id tests

#[test]
fn resolve_id_exact_match() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-abc12345".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();

    // Exact match should work
    let resolved = db.resolve_id("test-abc12345").unwrap();
    assert_eq!(resolved, "test-abc12345");
}

#[test]
fn resolve_id_prefix_match() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-abc12345".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();

    // Prefix match with 8 characters should work
    let resolved = db.resolve_id("test-abc").unwrap();
    assert_eq!(resolved, "test-abc12345");
}

#[test]
fn resolve_id_ambiguous() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = Issue::new(
        "test-abc12345".to_string(),
        IssueType::Task,
        "First issue".to_string(),
    );
    let issue2 = Issue::new(
        "test-abc67890".to_string(),
        IssueType::Task,
        "Second issue".to_string(),
    );

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    // Prefix that matches multiple issues should return error
    let result = db.resolve_id("test-abc");
    assert!(matches!(result, Err(Error::AmbiguousId { .. })));

    if let Err(Error::AmbiguousId { prefix, matches }) = result {
        assert_eq!(prefix, "test-abc");
        assert_eq!(matches.len(), 2);
        assert!(matches.contains(&"test-abc12345".to_string()));
        assert!(matches.contains(&"test-abc67890".to_string()));
    }
}

#[test]
fn resolve_id_not_found() {
    let db = Database::open_in_memory().unwrap();

    // Non-existent ID should return error
    let result = db.resolve_id("test-nonexistent");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn resolve_id_too_short() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "test-abc12345".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();

    // Prefix shorter than 3 characters should fail with not found
    let result = db.resolve_id("te");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn resolve_id_unique_prefix_at_minimum_length() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new(
        "abc-12345678".to_string(),
        IssueType::Task,
        "Test issue".to_string(),
    );

    db.create_issue(&issue).unwrap();

    // Prefix at exactly 3 characters should work if unambiguous
    let resolved = db.resolve_id("abc").unwrap();
    assert_eq!(resolved, "abc-12345678");
}
