// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

#[test]
fn test_hooks_install_with_y_flag() {
    let cli = parse(&["wk", "hooks", "install", "-y"]).unwrap();
    match cli.command {
        Command::Hooks(HooksCommand::Install {
            scope,
            interactive,
            yes,
        }) => {
            assert!(scope.is_none());
            assert!(!interactive);
            assert!(yes);
        }
        _ => panic!("Expected Hooks Install command"),
    }
}

#[test]
fn test_hooks_install_with_y_flag_and_scope() {
    let cli = parse(&["wk", "hooks", "install", "-y", "local"]).unwrap();
    match cli.command {
        Command::Hooks(HooksCommand::Install {
            scope,
            interactive,
            yes,
        }) => {
            assert_eq!(scope, Some("local".to_string()));
            assert!(!interactive);
            assert!(yes);
        }
        _ => panic!("Expected Hooks Install command"),
    }
}

#[test]
fn test_hooks_install_with_yes_long_flag() {
    let cli = parse(&["wk", "hooks", "install", "--yes"]).unwrap();
    match cli.command {
        Command::Hooks(HooksCommand::Install {
            scope,
            interactive,
            yes,
        }) => {
            assert!(scope.is_none());
            assert!(!interactive);
            assert!(yes);
        }
        _ => panic!("Expected Hooks Install command"),
    }
}

#[test]
fn test_hooks_install_rejects_q_shorthand() {
    // -q short flag was renamed to -y
    let result = parse(&["wk", "hooks", "install", "-q"]);
    assert!(result.is_err());
}

#[test]
fn test_hooks_install_rejects_quiet_long_flag() {
    // --quiet was renamed to --yes
    let result = parse(&["wk", "hooks", "install", "--quiet"]);
    assert!(result.is_err());
}

#[test]
fn test_hooks_install_i_and_y_conflict() {
    // -i and -y should conflict
    let result = parse(&["wk", "hooks", "install", "-i", "-y"]);
    assert!(result.is_err());
}
