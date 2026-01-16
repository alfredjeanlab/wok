// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Dep commands
#[test]
fn test_dep_single_target() {
    let cli = parse(&["wk", "dep", "prj-a", "blocks", "prj-b"]).unwrap();
    match cli.command {
        Command::Dep {
            from_id,
            rel,
            to_ids,
        } => {
            assert_eq!(from_id, "prj-a");
            assert_eq!(rel, "blocks");
            assert_eq!(to_ids, vec!["prj-b"]);
        }
        _ => panic!("Expected Dep command"),
    }
}

#[test]
fn test_dep_multiple_targets() {
    let cli = parse(&["wk", "dep", "prj-a", "tracks", "prj-b", "prj-c"]).unwrap();
    match cli.command {
        Command::Dep {
            from_id,
            rel,
            to_ids,
        } => {
            assert_eq!(from_id, "prj-a");
            assert_eq!(rel, "tracks");
            assert_eq!(to_ids, vec!["prj-b", "prj-c"]);
        }
        _ => panic!("Expected Dep command"),
    }
}

#[test]
fn test_dep_requires_target() {
    let result = parse(&["wk", "dep", "prj-a", "blocks"]);
    assert!(result.is_err());
}

#[test]
fn test_undep_command() {
    let cli = parse(&["wk", "undep", "prj-a", "blocks", "prj-b"]).unwrap();
    match cli.command {
        Command::Undep {
            from_id,
            rel,
            to_ids,
        } => {
            assert_eq!(from_id, "prj-a");
            assert_eq!(rel, "blocks");
            assert_eq!(to_ids, vec!["prj-b"]);
        }
        _ => panic!("Expected Undep command"),
    }
}

// Label commands
#[test]
fn test_label_command() {
    let cli = parse(&["wk", "label", "prj-1234", "urgent"]).unwrap();
    match cli.command {
        Command::Label { args } => {
            assert_eq!(args, vec!["prj-1234", "urgent"]);
        }
        _ => panic!("Expected Label command"),
    }
}

#[test]
fn test_label_command_multiple() {
    let cli = parse(&["wk", "label", "prj-1", "prj-2", "prj-3", "urgent"]).unwrap();
    match cli.command {
        Command::Label { args } => {
            assert_eq!(args, vec!["prj-1", "prj-2", "prj-3", "urgent"]);
        }
        _ => panic!("Expected Label command"),
    }
}

#[test]
fn test_label_requires_id_and_label() {
    // Need at least 2 arguments (1 ID + 1 label)
    let result = parse(&["wk", "label", "prj-1234"]);
    assert!(result.is_err());
}

#[test]
fn test_unlabel_command() {
    let cli = parse(&["wk", "unlabel", "prj-1234", "urgent"]).unwrap();
    match cli.command {
        Command::Unlabel { args } => {
            assert_eq!(args, vec!["prj-1234", "urgent"]);
        }
        _ => panic!("Expected Unlabel command"),
    }
}

#[test]
fn test_unlabel_command_multiple() {
    let cli = parse(&["wk", "unlabel", "prj-1", "prj-2", "urgent"]).unwrap();
    match cli.command {
        Command::Unlabel { args } => {
            assert_eq!(args, vec!["prj-1", "prj-2", "urgent"]);
        }
        _ => panic!("Expected Unlabel command"),
    }
}

#[test]
fn test_unlabel_requires_id_and_label() {
    // Need at least 2 arguments (1 ID + 1 label)
    let result = parse(&["wk", "unlabel", "prj-1234"]);
    assert!(result.is_err());
}

// Note command
#[test]
fn test_note_command() {
    let cli = parse(&["wk", "note", "prj-1234", "This is a note"]).unwrap();
    match cli.command {
        Command::Note {
            id,
            content,
            replace,
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(content, "This is a note");
            assert!(!replace);
        }
        _ => panic!("Expected Note command"),
    }
}

#[test]
fn test_note_command_with_replace() {
    let cli = parse(&["wk", "note", "prj-1234", "Updated note", "--replace"]).unwrap();
    match cli.command {
        Command::Note {
            id,
            content,
            replace,
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(content, "Updated note");
            assert!(replace);
        }
        _ => panic!("Expected Note command"),
    }
}

#[test]
fn test_note_command_rejects_r_shorthand() {
    // -r short flag was removed from 'note' command
    let result = parse(&["wk", "note", "prj-1234", "-r", "Note text"]);
    assert!(result.is_err());
}
