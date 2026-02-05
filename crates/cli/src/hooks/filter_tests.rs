// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use chrono::Utc;

fn make_issue(id: &str, issue_type: IssueType, status: Status) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type,
        title: "Test issue".to_string(),
        description: None,
        status,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    }
}

#[test]
fn parse_empty_filter() {
    let filter = HookFilter::parse("").unwrap();
    assert!(filter.is_empty());
}

#[test]
fn parse_type_filter() {
    let filter = HookFilter::parse("-t bug").unwrap();
    assert!(filter.types.is_some());
    assert_eq!(filter.types.as_ref().unwrap().len(), 1);
    assert_eq!(filter.types.as_ref().unwrap()[0].len(), 1);
    assert_eq!(filter.types.as_ref().unwrap()[0][0], IssueType::Bug);
}

#[test]
fn parse_multiple_types() {
    let filter = HookFilter::parse("-t bug,task").unwrap();
    assert!(filter.types.is_some());
    let types = filter.types.as_ref().unwrap();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].len(), 2);
}

#[test]
fn parse_label_filter() {
    let filter = HookFilter::parse("-l urgent").unwrap();
    assert!(filter.labels.is_some());
    assert_eq!(filter.labels.as_ref().unwrap().len(), 1);
}

#[test]
fn parse_negated_label() {
    let filter = HookFilter::parse("-l !wip").unwrap();
    assert!(filter.labels.is_some());
}

#[test]
fn parse_status_filter() {
    let filter = HookFilter::parse("-s todo").unwrap();
    assert!(filter.statuses.is_some());
    assert_eq!(filter.statuses.as_ref().unwrap().len(), 1);
}

#[test]
fn parse_assignee_filter() {
    let filter = HookFilter::parse("-a alice").unwrap();
    assert!(filter.assignees.is_some());
    assert_eq!(filter.assignees.as_ref().unwrap()[0][0], "alice");
}

#[test]
fn parse_prefix_filter() {
    let filter = HookFilter::parse("-p proj").unwrap();
    assert_eq!(filter.prefix, Some("proj".to_string()));
}

#[test]
fn parse_combined_filters() {
    let filter = HookFilter::parse("-t bug -l urgent -s todo").unwrap();
    assert!(filter.types.is_some());
    assert!(filter.labels.is_some());
    assert!(filter.statuses.is_some());
}

#[test]
fn parse_long_form_flags() {
    let filter = HookFilter::parse("--type bug --label urgent --status todo").unwrap();
    assert!(filter.types.is_some());
    assert!(filter.labels.is_some());
    assert!(filter.statuses.is_some());
}

#[test]
fn parse_invalid_type_returns_error() {
    let result = HookFilter::parse("-t invalid");
    assert!(result.is_err());
}

#[test]
fn parse_invalid_status_returns_error() {
    let result = HookFilter::parse("-s invalid");
    assert!(result.is_err());
}

#[test]
fn parse_unknown_flag_returns_error() {
    let result = HookFilter::parse("--unknown value");
    assert!(result.is_err());
}

#[test]
fn parse_missing_value_returns_error() {
    let result = HookFilter::parse("-t");
    assert!(result.is_err());
}

#[test]
fn matches_type_filter() {
    let filter = HookFilter::parse("-t bug").unwrap();
    let bug = make_issue("test-1", IssueType::Bug, Status::Todo);
    let task = make_issue("test-2", IssueType::Task, Status::Todo);

    assert!(filter.matches(&bug, &[]));
    assert!(!filter.matches(&task, &[]));
}

#[test]
fn matches_status_filter() {
    let filter = HookFilter::parse("-s in_progress").unwrap();
    let in_progress = make_issue("test-1", IssueType::Bug, Status::InProgress);
    let todo = make_issue("test-2", IssueType::Bug, Status::Todo);

    assert!(filter.matches(&in_progress, &[]));
    assert!(!filter.matches(&todo, &[]));
}

#[test]
fn matches_label_filter() {
    let filter = HookFilter::parse("-l urgent").unwrap();
    let issue = make_issue("test-1", IssueType::Bug, Status::Todo);

    assert!(filter.matches(&issue, &["urgent".to_string()]));
    assert!(!filter.matches(&issue, &["normal".to_string()]));
    assert!(!filter.matches(&issue, &[]));
}

#[test]
fn matches_negated_label() {
    let filter = HookFilter::parse("-l !wip").unwrap();
    let issue = make_issue("test-1", IssueType::Bug, Status::Todo);

    assert!(filter.matches(&issue, &[]));
    assert!(filter.matches(&issue, &["urgent".to_string()]));
    assert!(!filter.matches(&issue, &["wip".to_string()]));
}

#[test]
fn matches_prefix_filter() {
    let filter = HookFilter::parse("-p proj").unwrap();
    let proj_issue = make_issue("proj-abc", IssueType::Bug, Status::Todo);
    let other_issue = make_issue("other-abc", IssueType::Bug, Status::Todo);

    assert!(filter.matches(&proj_issue, &[]));
    assert!(!filter.matches(&other_issue, &[]));
}

#[test]
fn empty_filter_matches_all() {
    let filter = HookFilter::parse("").unwrap();
    let issue = make_issue("test-1", IssueType::Bug, Status::Todo);

    assert!(filter.matches(&issue, &[]));
    assert!(filter.matches(&issue, &["any".to_string()]));
}

#[test]
fn combined_filter_requires_all() {
    let filter = HookFilter::parse("-t bug -l urgent").unwrap();
    let bug_urgent = make_issue("test-1", IssueType::Bug, Status::Todo);
    let bug_normal = make_issue("test-2", IssueType::Bug, Status::Todo);
    let task_urgent = make_issue("test-3", IssueType::Task, Status::Todo);

    assert!(filter.matches(&bug_urgent, &["urgent".to_string()]));
    assert!(!filter.matches(&bug_normal, &["normal".to_string()]));
    assert!(!filter.matches(&task_urgent, &["urgent".to_string()]));
}
