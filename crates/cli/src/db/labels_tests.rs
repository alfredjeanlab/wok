// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::{Issue, IssueType};

#[test]
fn test_add_and_get_labels() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_label("test-1234", "project:auth").unwrap();
    db.add_label("test-1234", "urgent").unwrap();

    let labels = db.get_labels("test-1234").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"project:auth".to_string()));
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_remove_label() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_label("test-1234", "label1").unwrap();
    db.add_label("test-1234", "label2").unwrap();

    db.remove_label("test-1234", "label1").unwrap();

    let labels = db.get_labels("test-1234").unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0], "label2");
}

#[test]
fn test_duplicate_label_ignored() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_label("test-1234", "label1").unwrap();
    db.add_label("test-1234", "label1").unwrap(); // Should not fail

    let labels = db.get_labels("test-1234").unwrap();
    assert_eq!(labels.len(), 1);
}

#[test]
fn test_get_labels_batch() {
    let db = Database::open_in_memory().unwrap();

    // Create multiple issues with different labels
    let issue1 = Issue::new("test-1".to_string(), IssueType::Task, "Test 1".to_string());
    let issue2 = Issue::new("test-2".to_string(), IssueType::Task, "Test 2".to_string());
    let issue3 = Issue::new("test-3".to_string(), IssueType::Task, "Test 3".to_string());
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();
    db.create_issue(&issue3).unwrap();

    db.add_label("test-1", "priority:1").unwrap();
    db.add_label("test-1", "team:backend").unwrap();
    db.add_label("test-2", "priority:2").unwrap();
    // test-3 has no labels

    // Fetch labels for all three issues in one query
    let labels_map = db
        .get_labels_batch(&["test-1", "test-2", "test-3"])
        .unwrap();

    // Verify results
    let labels_1 = labels_map.get("test-1").unwrap();
    assert_eq!(labels_1.len(), 2);
    assert!(labels_1.contains(&"priority:1".to_string()));
    assert!(labels_1.contains(&"team:backend".to_string()));

    let labels_2 = labels_map.get("test-2").unwrap();
    assert_eq!(labels_2.len(), 1);
    assert!(labels_2.contains(&"priority:2".to_string()));

    // Issues without labels won't be in the map
    assert!(!labels_map.contains_key("test-3"));
}

#[test]
fn test_get_labels_batch_empty() {
    let db = Database::open_in_memory().unwrap();

    // Empty input should return empty map
    let labels_map = db.get_labels_batch(&[]).unwrap();
    assert!(labels_map.is_empty());
}

#[test]
fn test_get_labels_batch_nonexistent_ids() {
    let db = Database::open_in_memory().unwrap();

    // Querying nonexistent IDs should return empty map (no error)
    let labels_map = db
        .get_labels_batch(&["nonexistent-1", "nonexistent-2"])
        .unwrap();
    assert!(labels_map.is_empty());
}
