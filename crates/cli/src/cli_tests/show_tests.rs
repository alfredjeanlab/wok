// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Show and Tree commands
#[test]
fn test_show_command() {
    let cli = parse(&["wk", "show", "prj-1234"]).unwrap();
    match cli.command {
        Command::Show { id, format } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(format, "text"); // default format
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_with_json_format() {
    let cli = parse(&["wk", "show", "prj-1234", "--format", "json"]).unwrap();
    match cli.command {
        Command::Show { id, format } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(format, "json");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_with_format_short() {
    let cli = parse(&["wk", "show", "prj-1234", "-f", "json"]).unwrap();
    match cli.command {
        Command::Show { id, format } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(format, "json");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_tree_command() {
    let cli = parse(&["wk", "tree", "prj-1234"]).unwrap();
    match cli.command {
        Command::Tree { id } => assert_eq!(id, "prj-1234"),
        _ => panic!("Expected Tree command"),
    }
}
