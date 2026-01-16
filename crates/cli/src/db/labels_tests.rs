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
