// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// New command tests
#[test]
fn test_new_with_title_only() {
    let cli = parse(&["wk", "new", "My issue title"]).unwrap();
    match cli.command {
        Command::New {
            type_or_title,
            title,
            label,
            note,
            link,
            assignee,
            priority,
            description,
            blocks,
            blocked_by,
            tracks,
            tracked_by,
        } => {
            assert_eq!(type_or_title, "My issue title");
            assert!(title.is_none());
            assert!(label.is_empty());
            assert!(note.is_none());
            assert!(link.is_empty());
            assert!(assignee.is_none());
            assert!(priority.is_none());
            assert!(description.is_none());
            assert!(blocks.is_empty());
            assert!(blocked_by.is_empty());
            assert!(tracks.is_empty());
            assert!(tracked_by.is_empty());
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_type_and_title() {
    let cli = parse(&["wk", "new", "bug", "Fix the crash"]).unwrap();
    match cli.command {
        Command::New {
            type_or_title,
            title,
            ..
        } => {
            assert_eq!(type_or_title, "bug");
            assert_eq!(title, Some("Fix the crash".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_labels() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "Do something",
        "-l",
        "urgent",
        "-l",
        "frontend",
    ])
    .unwrap();
    match cli.command {
        Command::New { label, .. } => {
            assert_eq!(label, vec!["urgent", "frontend"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_note() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "Do something",
        "--note",
        "Initial note",
    ])
    .unwrap();
    match cli.command {
        Command::New { note, .. } => {
            assert_eq!(note, Some("Initial note".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_note_rejects_n_shorthand() {
    // -n short flag was removed for --note
    let result = parse(&["wk", "new", "task", "Do something", "-n", "Initial note"]);
    assert!(result.is_err());
}

// Hidden --description flag tests

#[test]
fn test_new_with_description() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "My task",
        "--description",
        "Initial context",
    ])
    .unwrap();
    match cli.command {
        Command::New { description, .. } => {
            assert_eq!(description, Some("Initial context".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_both_note_and_description() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "My task",
        "--note",
        "Note text",
        "--description",
        "Desc text",
    ])
    .unwrap();
    match cli.command {
        Command::New {
            note, description, ..
        } => {
            assert_eq!(note, Some("Note text".to_string()));
            assert_eq!(description, Some("Desc text".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_description_with_label() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "My task",
        "--description",
        "Context",
        "-l",
        "urgent",
    ])
    .unwrap();
    match cli.command {
        Command::New {
            description, label, ..
        } => {
            assert_eq!(description, Some("Context".to_string()));
            assert_eq!(label, vec!["urgent"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_description_no_short_flag() {
    // -d should not work as description (it's not defined as a short flag)
    let result = parse(&["wk", "new", "task", "My task", "-d", "Desc"]);
    assert!(result.is_err());
}

// Priority flag tests (hidden argument)
#[test]
fn test_new_with_priority() {
    let cli = parse(&["wk", "new", "task", "My task", "--priority", "2"]).unwrap();
    match cli.command {
        Command::New { priority, .. } => {
            assert_eq!(priority, Some(2));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_priority_with_label() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "My task",
        "--priority",
        "1",
        "-l",
        "urgent",
    ])
    .unwrap();
    match cli.command {
        Command::New {
            priority, label, ..
        } => {
            assert_eq!(priority, Some(1));
            assert_eq!(label, vec!["urgent"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_priority_bounds() {
    // Valid bounds
    assert!(parse(&["wk", "new", "task", "Test", "--priority", "0"]).is_ok());
    assert!(parse(&["wk", "new", "task", "Test", "--priority", "4"]).is_ok());

    // Invalid bounds
    assert!(parse(&["wk", "new", "task", "Test", "--priority", "5"]).is_err());
    assert!(parse(&["wk", "new", "task", "Test", "--priority", "-1"]).is_err());
}

#[test]
fn test_new_priority_non_numeric() {
    assert!(parse(&["wk", "new", "task", "Test", "--priority", "high"]).is_err());
}

// Empty title validation tests (at clap level)
#[test]
fn test_new_empty_title_rejected() {
    let result = parse(&["wk", "new", ""]);
    match result {
        Err(e) => {
            let err = e.to_string();
            assert!(
                err.contains("cannot be empty"),
                "Expected 'cannot be empty' error, got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for empty title"),
    }
}

#[test]
fn test_new_whitespace_only_title_rejected() {
    let result = parse(&["wk", "new", "   "]);
    match result {
        Err(e) => {
            let err = e.to_string();
            assert!(
                err.contains("cannot be empty"),
                "Expected 'cannot be empty' error, got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for whitespace-only title"),
    }
}

#[test]
fn test_new_type_and_empty_title_rejected() {
    let result = parse(&["wk", "new", "task", ""]);
    match result {
        Err(e) => {
            let err = e.to_string();
            assert!(
                err.contains("cannot be empty"),
                "Expected 'cannot be empty' error, got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for empty title with type"),
    }
}

#[test]
fn test_new_type_and_whitespace_only_title_rejected() {
    let result = parse(&["wk", "new", "task", "   "]);
    match result {
        Err(e) => {
            let err = e.to_string();
            assert!(
                err.contains("cannot be empty"),
                "Expected 'cannot be empty' error, got: {}",
                err
            );
        }
        Ok(_) => panic!("Expected error for whitespace-only title with type"),
    }
}

// Dependency flag tests
#[test]
fn test_new_with_blocks() {
    let cli = parse(&["wk", "new", "bug", "Fix crash", "--blocks", "task-1"]).unwrap();
    match cli.command {
        Command::New { blocks, .. } => {
            assert_eq!(blocks, vec!["task-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_multiple_blocks() {
    let cli = parse(&[
        "wk",
        "new",
        "bug",
        "Fix crash",
        "--blocks",
        "task-1",
        "--blocks",
        "task-2",
    ])
    .unwrap();
    match cli.command {
        Command::New { blocks, .. } => {
            assert_eq!(blocks, vec!["task-1", "task-2"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_blocked_by() {
    let cli = parse(&["wk", "new", "task", "Test", "--blocked-by", "blocker-1"]).unwrap();
    match cli.command {
        Command::New { blocked_by, .. } => {
            assert_eq!(blocked_by, vec!["blocker-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_tracks() {
    let cli = parse(&["wk", "new", "feature", "Epic", "--tracks", "subtask-1"]).unwrap();
    match cli.command {
        Command::New { tracks, .. } => {
            assert_eq!(tracks, vec!["subtask-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_tracked_by() {
    let cli = parse(&["wk", "new", "task", "Subtask", "--tracked-by", "feature-1"]).unwrap();
    match cli.command {
        Command::New { tracked_by, .. } => {
            assert_eq!(tracked_by, vec!["feature-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_all_dependency_flags() {
    let cli = parse(&[
        "wk",
        "new",
        "task",
        "Complex task",
        "--blocks",
        "a",
        "--blocked-by",
        "b",
        "--tracks",
        "c",
        "--tracked-by",
        "d",
    ])
    .unwrap();
    match cli.command {
        Command::New {
            blocks,
            blocked_by,
            tracks,
            tracked_by,
            ..
        } => {
            assert_eq!(blocks, vec!["a"]);
            assert_eq!(blocked_by, vec!["b"]);
            assert_eq!(tracks, vec!["c"]);
            assert_eq!(tracked_by, vec!["d"]);
        }
        _ => panic!("Expected New command"),
    }
}
