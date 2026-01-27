// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// List command tests
#[test]
fn test_list_default() {
    let cli = parse(&["wk", "list"]).unwrap();
    match cli.command {
        Command::List {
            status,
            type_label,
            blocked,
            ..
        } => {
            assert!(status.is_empty());
            assert!(type_label.r#type.is_empty());
            assert!(type_label.label.is_empty());
            assert!(!blocked);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_with_status() {
    let cli = parse(&["wk", "list", "-s", "in_progress"]).unwrap();
    match cli.command {
        Command::List { status, .. } => {
            assert_eq!(status, vec!["in_progress".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_with_type() {
    let cli = parse(&["wk", "list", "-t", "bug"]).unwrap();
    match cli.command {
        Command::List { type_label, .. } => {
            assert_eq!(type_label.r#type, vec!["bug".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_with_type_long_flag() {
    let cli = parse(&["wk", "list", "--type", "bug"]).unwrap();
    match cli.command {
        Command::List { type_label, .. } => {
            assert_eq!(type_label.r#type, vec!["bug".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_old_type_flag_fails() {
    // -T should no longer work (changed to -t)
    let result = parse(&["wk", "list", "-T", "bug"]);
    assert!(result.is_err());
}

#[test]
fn test_list_with_label() {
    let cli = parse(&["wk", "list", "-l", "urgent"]).unwrap();
    match cli.command {
        Command::List { type_label, .. } => {
            assert_eq!(type_label.label, vec!["urgent".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_blocked_flag() {
    let cli = parse(&["wk", "list", "--blocked"]).unwrap();
    match cli.command {
        Command::List { blocked, .. } => {
            assert!(blocked);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_blocked_short_flag_fails() {
    // -b short flag was removed for --blocked
    let result = parse(&["wk", "list", "-b"]);
    assert!(result.is_err());
}

#[test]
fn test_list_with_comma_separated_status() {
    let cli = parse(&["wk", "list", "-s", "todo,in_progress"]).unwrap();
    match cli.command {
        Command::List { status, .. } => {
            assert_eq!(status, vec!["todo,in_progress".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_with_repeated_labels() {
    let cli = parse(&["wk", "list", "-l", "a", "-l", "b"]).unwrap();
    match cli.command {
        Command::List { type_label, .. } => {
            assert_eq!(type_label.label, vec!["a".to_string(), "b".to_string()]);
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_with_mixed_filters() {
    let cli = parse(&[
        "wk",
        "list",
        "-s",
        "todo",
        "-l",
        "urgent,bug",
        "-l",
        "backend",
    ])
    .unwrap();
    match cli.command {
        Command::List {
            status, type_label, ..
        } => {
            assert_eq!(status, vec!["todo".to_string()]);
            assert_eq!(
                type_label.label,
                vec!["urgent,bug".to_string(), "backend".to_string()]
            );
        }
        _ => panic!("Expected List command"),
    }
}

// Ready command tests
#[test]
fn test_ready_default() {
    let cli = parse(&["wk", "ready"]).unwrap();
    match cli.command {
        Command::Ready {
            type_label,
            assignee,
            unassigned,
            all_assignees,
            output,
        } => {
            assert!(type_label.r#type.is_empty());
            assert!(type_label.label.is_empty());
            assert!(assignee.is_empty());
            assert!(!unassigned);
            assert!(!all_assignees);
            assert!(matches!(output, OutputFormat::Text));
        }
        _ => panic!("Expected Ready command"),
    }
}

#[test]
fn test_ready_with_type_filter() {
    let cli = parse(&["wk", "ready", "-t", "bug"]).unwrap();
    match cli.command {
        Command::Ready { type_label, .. } => {
            assert_eq!(type_label.r#type, vec!["bug".to_string()]);
        }
        _ => panic!("Expected Ready command"),
    }
}

#[test]
fn test_ready_with_label_filter() {
    let cli = parse(&["wk", "ready", "-l", "urgent"]).unwrap();
    match cli.command {
        Command::Ready { type_label, .. } => {
            assert_eq!(type_label.label, vec!["urgent".to_string()]);
        }
        _ => panic!("Expected Ready command"),
    }
}

#[test]
fn test_ready_with_combined_filters() {
    let cli = parse(&["wk", "ready", "-t", "bug", "-l", "urgent"]).unwrap();
    match cli.command {
        Command::Ready { type_label, .. } => {
            assert_eq!(type_label.r#type, vec!["bug".to_string()]);
            assert_eq!(type_label.label, vec!["urgent".to_string()]);
        }
        _ => panic!("Expected Ready command"),
    }
}

#[test]
fn test_ready_does_not_accept_all_flag() {
    let result = parse(&["wk", "ready", "--all"]);
    assert!(result.is_err());
}

#[test]
fn test_ready_does_not_accept_blocked_flag() {
    let result = parse(&["wk", "ready", "--blocked"]);
    assert!(result.is_err());
}

#[test]
fn test_ready_accepts_label_flag() {
    // --label is now the correct flag (--tag was renamed to --label)
    let cli = parse(&["wk", "ready", "--label", "foo"]).unwrap();
    match cli.command {
        Command::Ready { type_label, .. } => {
            assert_eq!(type_label.label, vec!["foo".to_string()]);
        }
        _ => panic!("Expected Ready command"),
    }
}
