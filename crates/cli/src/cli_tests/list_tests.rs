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

// ─────────────────────────────────────────────────────────────────────────────
// Phase 4: Parameterized output format tests
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_list_output_format_text_default() {
    let cli = parse(&["wk", "list"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Text));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_json_long() {
    let cli = parse(&["wk", "list", "--output", "json"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Json));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_json_short() {
    let cli = parse(&["wk", "list", "-o", "json"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Json));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_ids_long() {
    let cli = parse(&["wk", "list", "--output", "ids"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_ids_short() {
    let cli = parse(&["wk", "list", "-o", "ids"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_id_alias() {
    let cli = parse(&["wk", "list", "-o", "id"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Id));
        }
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_list_output_format_text_explicit() {
    let cli = parse(&["wk", "list", "-o", "text"]).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, OutputFormat::Text));
        }
        _ => panic!("Expected List command"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 5: Parameterized filter flag tests
// ─────────────────────────────────────────────────────────────────────────────

#[parameterized(
    single_filter_short = { &["wk", "list", "-q", "age < 1d"], vec!["age < 1d"] },
    single_filter_long = { &["wk", "list", "--filter", "closed < 1w"], vec!["closed < 1w"] },
    multiple_filters = { &["wk", "list", "-q", "age < 1d", "-q", "updated < 1h"], vec!["age < 1d", "updated < 1h"] },
)]
fn test_list_filter_parsing(args: &[&str], expected: Vec<&str>) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::List { filter, .. } => {
            let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
            assert_eq!(filter, expected);
        }
        _ => panic!("Expected List command"),
    }
}

#[parameterized(
    limit_short = { &["wk", "list", "-n", "50"], Some(50) },
    limit_long = { &["wk", "list", "--limit", "100"], Some(100) },
    no_limit_flag = { &["wk", "list", "--no-limit"], None },
)]
fn test_list_limit_parsing(args: &[&str], expected_limit: Option<usize>) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::List { limits, .. } => {
            if expected_limit.is_none() {
                // --no-limit flag was used
                assert!(limits.no_limit);
            } else {
                assert_eq!(limits.limit, expected_limit);
            }
        }
        _ => panic!("Expected List command"),
    }
}
