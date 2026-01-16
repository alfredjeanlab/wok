// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Lifecycle commands
#[test]
fn test_start_command() {
    let cli = parse(&["wk", "start", "prj-1234"]).unwrap();
    match cli.command {
        Command::Start { ids } => assert_eq!(ids, vec!["prj-1234"]),
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_start_command_multiple() {
    let cli = parse(&["wk", "start", "prj-1", "prj-2", "prj-3"]).unwrap();
    match cli.command {
        Command::Start { ids } => assert_eq!(ids, vec!["prj-1", "prj-2", "prj-3"]),
        _ => panic!("Expected Start command"),
    }
}

#[test]
fn test_done_command() {
    let cli = parse(&["wk", "done", "prj-1234"]).unwrap();
    match cli.command {
        Command::Done { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert!(reason.is_none());
        }
        _ => panic!("Expected Done command"),
    }
}

#[test]
fn test_done_command_multiple() {
    let cli = parse(&["wk", "done", "prj-1", "prj-2"]).unwrap();
    match cli.command {
        Command::Done { ids, reason } => {
            assert_eq!(ids, vec!["prj-1", "prj-2"]);
            assert!(reason.is_none());
        }
        _ => panic!("Expected Done command"),
    }
}

#[test]
fn test_done_with_reason() {
    let cli = parse(&["wk", "done", "prj-1234", "-r", "Already complete"]).unwrap();
    match cli.command {
        Command::Done { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(reason, Some("Already complete".to_string()));
        }
        _ => panic!("Expected Done command"),
    }
}

#[test]
fn test_done_multiple_with_reason() {
    let cli = parse(&["wk", "done", "prj-1", "prj-2", "-r", "upstream"]).unwrap();
    match cli.command {
        Command::Done { ids, reason } => {
            assert_eq!(ids, vec!["prj-1", "prj-2"]);
            assert_eq!(reason, Some("upstream".to_string()));
        }
        _ => panic!("Expected Done command"),
    }
}

#[test]
fn test_close_command() {
    let cli = parse(&["wk", "close", "prj-1234", "-r", "wontfix"]).unwrap();
    match cli.command {
        Command::Close { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(reason, Some("wontfix".to_string()));
        }
        _ => panic!("Expected Close command"),
    }
}

#[test]
fn test_close_command_multiple() {
    let cli = parse(&["wk", "close", "prj-1", "prj-2", "-r", "duplicate"]).unwrap();
    match cli.command {
        Command::Close { ids, reason } => {
            assert_eq!(ids, vec!["prj-1", "prj-2"]);
            assert_eq!(reason, Some("duplicate".to_string()));
        }
        _ => panic!("Expected Close command"),
    }
}

#[test]
fn test_close_without_reason() {
    // Reason is now optional (auto-populated for human interactive sessions)
    let cli = parse(&["wk", "close", "prj-1234"]).unwrap();
    match cli.command {
        Command::Close { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert!(reason.is_none());
        }
        _ => panic!("Expected Close command"),
    }
}

#[test]
fn test_reopen_command() {
    let cli = parse(&["wk", "reopen", "prj-1234", "--reason", "regression"]).unwrap();
    match cli.command {
        Command::Reopen { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(reason, Some("regression".to_string()));
        }
        _ => panic!("Expected Reopen command"),
    }
}

#[test]
fn test_reopen_command_multiple() {
    let cli = parse(&["wk", "reopen", "prj-1", "prj-2", "--reason", "regression"]).unwrap();
    match cli.command {
        Command::Reopen { ids, reason } => {
            assert_eq!(ids, vec!["prj-1", "prj-2"]);
            assert_eq!(reason, Some("regression".to_string()));
        }
        _ => panic!("Expected Reopen command"),
    }
}

#[test]
fn test_reopen_without_reason() {
    // Reason is now optional (auto-populated for human interactive sessions)
    let cli = parse(&["wk", "reopen", "prj-1234"]).unwrap();
    match cli.command {
        Command::Reopen { ids, reason } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert!(reason.is_none());
        }
        _ => panic!("Expected Reopen command"),
    }
}
