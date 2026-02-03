// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Positional edit command tests

#[test]
fn test_edit_title() {
    let cli = parse(&["wok", "edit", "prj-1234", "title", "New title"]).unwrap();
    match cli.command {
        Command::Edit {
            id, attr, value, ..
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr.unwrap(), "title");
            assert_eq!(value.unwrap(), "New title");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_description() {
    let cli = parse(&[
        "wok",
        "edit",
        "prj-1234",
        "description",
        "Updated description",
    ])
    .unwrap();
    match cli.command {
        Command::Edit {
            id, attr, value, ..
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr.unwrap(), "description");
            assert_eq!(value.unwrap(), "Updated description");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_type() {
    let cli = parse(&["wok", "edit", "prj-1234", "type", "bug"]).unwrap();
    match cli.command {
        Command::Edit {
            id, attr, value, ..
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr.unwrap(), "type");
            assert_eq!(value.unwrap(), "bug");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_requires_id() {
    // "wk edit title New" parses as id="title", attr=Some("New"), value=None
    // This is accepted by the parser but will fail at dispatch (missing value).
    // With only one arg, "wk edit title" parses as id="title" with no attr/value,
    // which is also accepted by the parser but fails at dispatch.
    // Verify that "wk edit" (no id at all) fails at parse level.
    let result = parse(&["wok", "edit"]);
    assert!(result.is_err());
}

#[test]
fn test_edit_assignee() {
    let cli = parse(&["wok", "edit", "prj-1234", "assignee", "alice"]).unwrap();
    match cli.command {
        Command::Edit {
            id, attr, value, ..
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr.unwrap(), "assignee");
            assert_eq!(value.unwrap(), "alice");
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_assignee_none() {
    let cli = parse(&["wok", "edit", "prj-1234", "assignee", "none"]).unwrap();
    match cli.command {
        Command::Edit {
            id, attr, value, ..
        } => {
            assert_eq!(id, "prj-1234");
            assert_eq!(attr.unwrap(), "assignee");
            assert_eq!(value.unwrap(), "none");
        }
        _ => panic!("Expected Edit command"),
    }
}

// Hidden flag variant tests

#[test]
fn test_edit_flag_title() {
    let cli = parse(&["wok", "edit", "prj-1", "--title", "New title"]).unwrap();
    match cli.command {
        Command::Edit {
            id,
            flag_title,
            attr,
            ..
        } => {
            assert_eq!(id, "prj-1");
            assert_eq!(flag_title.unwrap(), "New title");
            assert!(attr.is_none());
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_flag_description() {
    let cli = parse(&["wok", "edit", "prj-1", "--description", "Desc"]).unwrap();
    match cli.command {
        Command::Edit {
            id,
            flag_description,
            attr,
            ..
        } => {
            assert_eq!(id, "prj-1");
            assert_eq!(flag_description.unwrap(), "Desc");
            assert!(attr.is_none());
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_flag_type() {
    let cli = parse(&["wok", "edit", "prj-1", "--type", "bug"]).unwrap();
    match cli.command {
        Command::Edit {
            id,
            flag_type,
            attr,
            ..
        } => {
            assert_eq!(id, "prj-1");
            assert_eq!(flag_type.unwrap(), "bug");
            assert!(attr.is_none());
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_flag_assignee() {
    let cli = parse(&["wok", "edit", "prj-1", "--assignee", "alice"]).unwrap();
    match cli.command {
        Command::Edit {
            id,
            flag_assignee,
            attr,
            ..
        } => {
            assert_eq!(id, "prj-1");
            assert_eq!(flag_assignee.unwrap(), "alice");
            assert!(attr.is_none());
        }
        _ => panic!("Expected Edit command"),
    }
}

#[test]
fn test_edit_flag_conflicts_with_positional() {
    let result = parse(&["wok", "edit", "prj-1", "--title", "X", "title", "Y"]);
    assert!(result.is_err());
}

#[test]
fn test_edit_help_hides_flags() {
    let result = parse(&["wok", "edit", "--help"]);
    let help = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("--help should return an error"),
    };
    assert!(!help.contains("--title"), "flag --title should be hidden");
    assert!(
        !help.contains("--description"),
        "flag --description should be hidden"
    );
    assert!(!help.contains("--type"), "flag --type should be hidden");
    assert!(
        !help.contains("--assignee"),
        "flag --assignee should be hidden"
    );
}
