// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

#[test]
fn test_note_new() {
    let note = Note::new(
        "issue-123".to_string(),
        Status::InProgress,
        "Working on this".to_string(),
    );

    assert_eq!(note.id, 0); // Default, set by database
    assert_eq!(note.issue_id, "issue-123");
    assert_eq!(note.status, Status::InProgress);
    assert_eq!(note.content, "Working on this");
}

#[test]
fn test_note_new_different_statuses() {
    let todo_note = Note::new("a".to_string(), Status::Todo, "planned".to_string());
    let progress_note = Note::new("b".to_string(), Status::InProgress, "working".to_string());
    let done_note = Note::new("c".to_string(), Status::Done, "finished".to_string());
    let closed_note = Note::new("d".to_string(), Status::Closed, "abandoned".to_string());

    assert_eq!(todo_note.status, Status::Todo);
    assert_eq!(progress_note.status, Status::InProgress);
    assert_eq!(done_note.status, Status::Done);
    assert_eq!(closed_note.status, Status::Closed);
}

#[test]
fn test_note_with_empty_content() {
    let note = Note::new("issue-123".to_string(), Status::Todo, "".to_string());
    assert_eq!(note.content, "");
}

#[test]
fn test_note_with_multiline_content() {
    let content = "Line 1\nLine 2\nLine 3".to_string();
    let note = Note::new("issue-123".to_string(), Status::InProgress, content.clone());
    assert_eq!(note.content, content);
}
