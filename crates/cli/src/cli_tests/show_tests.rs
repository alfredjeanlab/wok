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
    let cli = parse(&["wok", "show", "prj-1234"]).unwrap();
    match cli.command {
        Command::Show { ids, output } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(output, "text"); // default output
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_with_json_output() {
    let cli = parse(&["wok", "show", "prj-1234", "--output", "json"]).unwrap();
    match cli.command {
        Command::Show { ids, output } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(output, "json");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_with_output_short() {
    let cli = parse(&["wok", "show", "prj-1234", "-o", "json"]).unwrap();
    match cli.command {
        Command::Show { ids, output } => {
            assert_eq!(ids, vec!["prj-1234"]);
            assert_eq!(output, "json");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_multiple_ids() {
    let cli = parse(&["wok", "show", "prj-1", "prj-2", "prj-3"]).unwrap();
    match cli.command {
        Command::Show { ids, output } => {
            assert_eq!(ids, vec!["prj-1", "prj-2", "prj-3"]);
            assert_eq!(output, "text");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_show_command_multiple_ids_with_json() {
    let cli = parse(&["wok", "show", "prj-1", "prj-2", "-o", "json"]).unwrap();
    match cli.command {
        Command::Show { ids, output } => {
            assert_eq!(ids, vec!["prj-1", "prj-2"]);
            assert_eq!(output, "json");
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_tree_command() {
    let cli = parse(&["wok", "tree", "prj-1234"]).unwrap();
    match cli.command {
        Command::Tree { id } => assert_eq!(id, "prj-1234"),
        _ => panic!("Expected Tree command"),
    }
}
