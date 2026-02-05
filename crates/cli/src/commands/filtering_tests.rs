// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::models::Status;

#[test]
fn test_matches_prefix_none() {
    assert!(matches_prefix(&None, "oj-123"));
}

#[test]
fn test_matches_prefix_matching() {
    let prefix = Some("oj".to_string());
    assert!(matches_prefix(&prefix, "oj-123"));
    assert!(matches_prefix(&prefix, "oj-abc-def"));
}

#[test]
fn test_matches_prefix_not_matching() {
    let prefix = Some("oj".to_string());
    assert!(!matches_prefix(&prefix, "proj-123"));
    assert!(!matches_prefix(&prefix, "other-456"));
}

#[test]
fn test_matches_prefix_no_hyphen() {
    let prefix = Some("oj".to_string());
    assert!(!matches_prefix(&prefix, "nohyphen"));
    assert!(matches_prefix(&Some("nohyphen".to_string()), "nohyphen"));
}

#[test]
fn test_matches_prefix_empty_id() {
    let prefix = Some("oj".to_string());
    assert!(!matches_prefix(&prefix, ""));
}

#[test]
fn test_parse_filter_groups_empty() {
    let result = parse_filter_groups::<Status, _>(&[], |s| Ok(s.parse()?)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_parse_filter_groups_single_value() {
    let values = vec!["todo".to_string()];
    let result = parse_filter_groups(&values, |s| Ok(s.parse::<Status>()?)).unwrap();
    assert_eq!(result, Some(vec![vec![Status::Todo]]));
}

#[test]
fn test_parse_filter_groups_comma_separated() {
    let values = vec!["todo,in_progress".to_string()];
    let result = parse_filter_groups(&values, |s| Ok(s.parse::<Status>()?)).unwrap();
    assert_eq!(result, Some(vec![vec![Status::Todo, Status::InProgress]]));
}

#[test]
fn test_parse_filter_groups_multiple_entries() {
    let values = vec!["todo".to_string(), "in_progress".to_string()];
    let result = parse_filter_groups(&values, |s| Ok(s.parse::<Status>()?)).unwrap();
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
