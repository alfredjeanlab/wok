// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Edit command tests
#[test]
fn test_edit_title() {
    let cli = parse(&["wk", "edit", "prj-1234", "title", "New title"]).unwrap();
    match cli.command {
        Command::Edit { id, attr, value } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr, "title");
            assert_eq!(value, "New title");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_description() {
    let cli = parse(&["wk", "edit", "prj-1234", "description", "Updated description"]).unwrap();
    match cli.command {
        Command::Edit { id, attr, value } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr, "description");
            assert_eq!(value, "Updated description");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_type() {
    let cli = parse(&["wk", "edit", "prj-1234", "type", "bug"]).unwrap();
    match cli.command {
        Command::Edit { id, attr, value } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr, "type");
            assert_eq!(value, "bug");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_requires_id() {
    // Missing id should fail
    let result = parse(&["wk", "edit", "title", "New"]);
    // This will parse as id="title", attr="New", missing value
    assert!(result.is_err());
}

#[test]
fn test_edit_requires_all_args() {
    // Just id with no attr and value should fail due to arg_required_else_help
    let result = parse(&["wk", "edit", "prj-1234"]);
    assert!(result.is_err());
}

#[test]
fn test_edit_assignee() {
    let cli = parse(&["wk", "edit", "prj-1234", "assignee", "alice"]).unwrap();
    match cli.command {
        Command::Edit { id, attr, value } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr, "assignee");
            assert_eq!(value, "alice");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_assignee_none() {
    let cli = parse(&["wk", "edit", "prj-1234", "assignee", "none"]).unwrap();
    match cli.command {
        Command::Edit { id, attr, value } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr, "assignee");
            assert_eq!(value, "none");
        }
        _ => panic!("Expected Edit command"),
    }
}
