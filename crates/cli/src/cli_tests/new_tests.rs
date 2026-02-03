// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// New command tests
#[test]
fn test_new_with_title_only() {
    let cli = parse(&["wok", "new", "My issue title"]).unwrap();
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
            output,
            prefix,
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
            assert!(matches!(output, OutputFormat::Text));
            assert!(prefix.is_none());
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_type_and_title() {
    let cli = parse(&["wok", "new", "bug", "Fix the crash"]).unwrap();
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
        "wok",
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
        "wok",
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
    let result = parse(&["wok", "new", "task", "Do something", "-n", "Initial note"]);
    assert!(result.is_err());
}

// Hidden --description flag tests

#[test]
fn test_new_with_description() {
    let cli = parse(&[
        "wok",
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
        "wok",
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
        "wok",
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
    let result = parse(&["wok", "new", "task", "My task", "-d", "Desc"]);
    assert!(result.is_err());
}

// Priority flag tests (hidden argument)
#[test]
fn test_new_with_priority() {
    let cli = parse(&["wok", "new", "task", "My task", "--priority", "2"]).unwrap();
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
        "wok",
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
    assert!(parse(&["wok", "new", "task", "Test", "--priority", "0"]).is_ok());
    assert!(parse(&["wok", "new", "task", "Test", "--priority", "4"]).is_ok());

    // Invalid bounds
    assert!(parse(&["wok", "new", "task", "Test", "--priority", "5"]).is_err());
    assert!(parse(&["wok", "new", "task", "Test", "--priority", "-1"]).is_err());
}

#[test]
fn test_new_priority_non_numeric() {
    assert!(parse(&["wok", "new", "task", "Test", "--priority", "high"]).is_err());
}

// Empty title validation tests (at clap level)
#[test]
fn test_new_empty_title_rejected() {
    let result = parse(&["wok", "new", ""]);
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
    let result = parse(&["wok", "new", "   "]);
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
    let result = parse(&["wok", "new", "task", ""]);
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
    let result = parse(&["wok", "new", "task", "   "]);
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
    let cli = parse(&["wok", "new", "bug", "Fix crash", "--blocks", "task-1"]).unwrap();
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
        "wok",
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
    let cli = parse(&["wok", "new", "task", "Test", "--blocked-by", "blocker-1"]).unwrap();
    match cli.command {
        Command::New { blocked_by, .. } => {
            assert_eq!(blocked_by, vec!["blocker-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_tracks() {
    let cli = parse(&["wok", "new", "feature", "Epic", "--tracks", "subtask-1"]).unwrap();
    match cli.command {
        Command::New { tracks, .. } => {
            assert_eq!(tracks, vec!["subtask-1"]);
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_tracked_by() {
    let cli = parse(&["wok", "new", "task", "Subtask", "--tracked-by", "feature-1"]).unwrap();
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
        "wok",
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

// Parameterized tests for IssueType parsing at CLI level

#[parameterized(
    task_lower = { "task", "Task title" },
    task_upper = { "TASK", "Task title" },
    task_mixed = { "Task", "Task title" },
    feature_lower = { "feature", "Feature title" },
    feature_upper = { "FEATURE", "Feature title" },
    feature_mixed = { "Feature", "Feature title" },
    bug_lower = { "bug", "Bug title" },
    bug_upper = { "BUG", "Bug title" },
    bug_mixed = { "Bug", "Bug title" },
    chore_lower = { "chore", "Chore title" },
    chore_upper = { "CHORE", "Chore title" },
    chore_mixed = { "Chore", "Chore title" },
    idea_lower = { "idea", "Idea title" },
    idea_upper = { "IDEA", "Idea title" },
    idea_mixed = { "Idea", "Idea title" },
)]
fn test_new_type_parsing(type_str: &str, title: &str) {
    let cli = parse(&["wok", "new", type_str, title]).unwrap();
    match cli.command {
        Command::New {
            type_or_title,
            title: parsed_title,
            ..
        } => {
            assert_eq!(type_or_title, type_str);
            assert_eq!(parsed_title, Some(title.to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[parameterized(
    epic = { "epic" },
    story = { "story" },
    invalid = { "invalid" },
)]
fn test_new_invalid_type_passes_cli_parsing(type_str: &str) {
    // Invalid types pass CLI parsing (validation happens in run_impl)
    let cli = parse(&["wok", "new", type_str, "Title"]);
    assert!(
        cli.is_ok(),
        "CLI parsing should succeed for any non-empty type"
    );
}

#[test]
fn test_new_empty_type_rejected_at_cli() {
    // Empty type is rejected by clap's non_empty_string validator
    let cli = parse(&["wok", "new", "", "Title"]);
    assert!(cli.is_err());
}

// Parameterized tests for output format

#[parameterized(
    text_default = { &["wok", "new", "task", "Test"] as &[&str] },
)]
fn test_new_output_format_default_text(args: &[&str]) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::New { output, .. } => {
            assert!(matches!(output, OutputFormat::Text));
        }
        _ => panic!("Expected New command"),
    }
}

#[parameterized(
    id_short = { &["wok", "new", "task", "Test", "-o", "id"] as &[&str] },
    id_long = { &["wok", "new", "task", "Test", "--output", "id"] as &[&str] },
)]
fn test_new_output_format_id(args: &[&str]) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::New { output, .. } => {
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected New command"),
    }
}

#[parameterized(
    ids_short = { &["wok", "new", "task", "Test", "-o", "ids"] as &[&str] },
    ids_long = { &["wok", "new", "task", "Test", "--output", "ids"] as &[&str] },
)]
fn test_new_output_format_ids_alias(args: &[&str]) {
    // "ids" is an alias for "id" for backwards compatibility
    let cli = parse(args).unwrap();
    match cli.command {
        Command::New { output, .. } => {
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected New command"),
    }
}

#[parameterized(
    json_short = { &["wok", "new", "task", "Test", "-o", "json"] as &[&str] },
    json_long = { &["wok", "new", "task", "Test", "--output", "json"] as &[&str] },
)]
fn test_new_output_format_json(args: &[&str]) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::New { output, .. } => {
            assert!(matches!(output, OutputFormat::Json));
        }
        _ => panic!("Expected New command"),
    }
}

// Prefix flag tests

#[test]
fn test_new_with_prefix_long() {
    let cli = parse(&["wok", "new", "task", "Test", "--prefix", "custom"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert_eq!(prefix, Some("custom".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_prefix_short() {
    let cli = parse(&["wok", "new", "task", "Test", "-p", "short"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert_eq!(prefix, Some("short".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_prefix_default_none() {
    let cli = parse(&["wok", "new", "task", "Test"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert!(prefix.is_none());
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_prefix_with_other_flags() {
    let cli = parse(&[
        "wok",
        "new",
        "bug",
        "Fix crash",
        "--prefix",
        "api",
        "-l",
        "urgent",
        "-o",
        "id",
    ])
    .unwrap();
    match cli.command {
        Command::New {
            type_or_title,
            prefix,
            label,
            output,
            ..
        } => {
            assert_eq!(type_or_title, "bug");
            assert_eq!(prefix, Some("api".to_string()));
            assert_eq!(label, vec!["urgent"]);
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected New command"),
    }
}
