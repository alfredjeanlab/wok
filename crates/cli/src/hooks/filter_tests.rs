// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use chrono::Utc;

fn make_test_issue(issue_type: IssueType, status: Status, assignee: Option<&str>) -> Issue {
    Issue {
        id: "test-123".to_string(),
        issue_type,
        title: "Test issue".to_string(),
        description: None,
        status,
        assignee: assignee.map(String::from),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    }
}

#[test]
fn test_parse_empty_filter() {
    let filter = HookFilter::parse("").unwrap();
    assert!(filter.types.is_none());
    assert!(filter.labels.is_none());
    assert!(filter.statuses.is_none());
    assert!(filter.assignees.is_none());
    assert!(filter.prefix.is_none());
}

#[test]
fn test_parse_type_filter() {
    let filter = HookFilter::parse("-t bug").unwrap();
    assert!(filter.types.is_some());
    let types = filter.types.unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0], vec![IssueType::Bug]);
}

#[test]
fn test_parse_type_filter_comma_separated() {
    let filter = HookFilter::parse("-t bug,task").unwrap();
    let types = filter.types.unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0], vec![IssueType::Bug, IssueType::Task]);
}

#[test]
fn test_parse_label_filter() {
    let filter = HookFilter::parse("-l urgent").unwrap();
    assert!(filter.labels.is_some());
}

#[test]
fn test_parse_label_filter_negated() {
    let filter = HookFilter::parse("-l !wip").unwrap();
    let labels = filter.labels.unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].len(), 1);
    assert!(matches!(labels[0][0], LabelMatcher::NotHas(_)));
}

#[test]
fn test_parse_status_filter() {
    let filter = HookFilter::parse("-s todo,in_progress").unwrap();
    let statuses = filter.statuses.unwrap();
    assert_eq!(statuses.len(), 1);
    assert_eq!(statuses[0], vec![Status::Todo, Status::InProgress]);
}

#[test]
fn test_parse_assignee_filter() {
    let filter = HookFilter::parse("-a alice").unwrap();
    let assignees = filter.assignees.unwrap();
    assert_eq!(assignees.len(), 1);
    assert_eq!(assignees[0], vec!["alice".to_string()]);
}

#[test]
fn test_parse_prefix_filter() {
    let filter = HookFilter::parse("-p proj").unwrap();
    assert_eq!(filter.prefix, Some("proj".to_string()));
}

#[test]
fn test_parse_combined_filters() {
    let filter = HookFilter::parse("-t bug -l urgent -s todo").unwrap();
    assert!(filter.types.is_some());
    assert!(filter.labels.is_some());
    assert!(filter.statuses.is_some());
}

#[test]
fn test_parse_long_flags() {
    let filter = HookFilter::parse("--type bug --label urgent --status todo").unwrap();
    assert!(filter.types.is_some());
    assert!(filter.labels.is_some());
    assert!(filter.statuses.is_some());
}

#[test]
fn test_parse_unknown_flag() {
    let result = HookFilter::parse("-x unknown");
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_value() {
    let result = HookFilter::parse("-t");
    assert!(result.is_err());
}

#[test]
fn test_matches_type_filter() {
    let filter = HookFilter::parse("-t bug").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);
    assert!(filter.matches(&issue, &[]));

    let issue = make_test_issue(IssueType::Task, Status::Todo, None);
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn test_matches_status_filter() {
    let filter = HookFilter::parse("-s todo").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);
    assert!(filter.matches(&issue, &[]));

    let issue = make_test_issue(IssueType::Bug, Status::InProgress, None);
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn test_matches_label_filter() {
    let filter = HookFilter::parse("-l urgent").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);

    assert!(filter.matches(&issue, &["urgent".to_string()]));
    assert!(!filter.matches(&issue, &["other".to_string()]));
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn test_matches_label_filter_negated() {
    let filter = HookFilter::parse("-l !wip").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);

    assert!(filter.matches(&issue, &[]));
    assert!(filter.matches(&issue, &["urgent".to_string()]));
    assert!(!filter.matches(&issue, &["wip".to_string()]));
}

#[test]
fn test_matches_assignee_filter() {
    let filter = HookFilter::parse("-a alice").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, Some("alice"));
    assert!(filter.matches(&issue, &[]));

    let issue = make_test_issue(IssueType::Bug, Status::Todo, Some("bob"));
    assert!(!filter.matches(&issue, &[]));

    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn test_matches_prefix_filter() {
    let filter = HookFilter::parse("-p test").unwrap();
    let mut issue = make_test_issue(IssueType::Bug, Status::Todo, None);
    issue.id = "test-123".to_string();
    assert!(filter.matches(&issue, &[]));

    issue.id = "proj-123".to_string();
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn test_matches_combined_filters_all_must_match() {
    let filter = HookFilter::parse("-t bug -l urgent").unwrap();
    let issue = make_test_issue(IssueType::Bug, Status::Todo, None);

    // Both type and label must match
    assert!(filter.matches(&issue, &["urgent".to_string()]));
    assert!(!filter.matches(&issue, &[])); // missing label

    let issue = make_test_issue(IssueType::Task, Status::Todo, None);
    assert!(!filter.matches(&issue, &["urgent".to_string()])); // wrong type
}

#[test]
fn test_tokenize_simple() {
    let tokens = tokenize("-t bug -l urgent").unwrap();
    assert_eq!(tokens, vec!["-t", "bug", "-l", "urgent"]);
}

#[test]
fn test_tokenize_quoted() {
    let tokens = tokenize("-l \"label with space\"").unwrap();
    assert_eq!(tokens, vec!["-l", "label with space"]);
}

#[test]
fn test_tokenize_unclosed_quote() {
    let result = tokenize("-l \"unclosed");
    assert!(result.is_err());
}
