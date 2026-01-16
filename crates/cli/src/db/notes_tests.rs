// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::{Issue, IssueType};

#[test]
fn test_add_and_get_notes() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_note("test-1234", Status::Todo, "First note")
        .unwrap();
    db.add_note("test-1234", Status::Todo, "Second note")
        .unwrap();
    db.add_note("test-1234", Status::InProgress, "Progress note")
        .unwrap();

    let notes = db.get_notes("test-1234").unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].content, "First note");
    assert_eq!(notes[2].status, Status::InProgress);
}

#[test]
fn test_get_notes_by_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_note("test-1234", Status::Todo, "Todo note 1")
        .unwrap();
    db.add_note("test-1234", Status::Todo, "Todo note 2")
        .unwrap();
    db.add_note("test-1234", Status::InProgress, "Progress note")
        .unwrap();

    let grouped = db.get_notes_by_status("test-1234").unwrap();
    assert_eq!(grouped.len(), 2);

    let (status, notes) = &grouped[0];
    assert_eq!(*status, Status::Todo);
    assert_eq!(notes.len(), 2);
}

#[test]
fn test_replace_note() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_note("test-1234", Status::Todo, "Original note")
        .unwrap();

    let result = db.replace_note("test-1234", Status::Todo, "Replaced note");
    assert!(result.is_ok());

    let notes = db.get_notes("test-1234").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Replaced note");
}

#[test]
fn test_replace_note_no_existing_notes() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    // Try to replace when there are no notes
    let result = db.replace_note("test-1234", Status::Todo, "New content");
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("no notes to replace"));
    }
}

#[test]
fn test_get_notes_by_status_empty() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    // No notes added
    let grouped = db.get_notes_by_status("test-1234").unwrap();
    assert!(grouped.is_empty());
}

#[test]
fn test_replace_note_updates_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = Issue::new("test-1234".to_string(), IssueType::Task, "Test".to_string());
    db.create_issue(&issue).unwrap();

    db.add_note("test-1234", Status::Todo, "Original note")
        .unwrap();

    // Replace with different status
    let result = db.replace_note("test-1234", Status::InProgress, "Updated note");
    assert!(result.is_ok());

    let notes = db.get_notes("test-1234").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].status, Status::InProgress);
    assert_eq!(notes[0].content, "Updated note");
}
