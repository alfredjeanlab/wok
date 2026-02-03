// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! CLI argument parsing tests for the `init` command.
//!
//! These tests verify that clap correctly parses all init command options:
//! - `--prefix` - Custom prefix for issue IDs
//! - `--path` - Target directory for initialization
//! - `--private` - Use private mode (project-local database, no daemon)

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

/// Helper to parse CLI args into a Cli struct.
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

/// Expected values for init command arguments.
struct InitExpected {
    prefix: Option<&'static str>,
    path: Option<&'static str>,
    private: bool,
}

impl InitExpected {
    const fn new(prefix: Option<&'static str>, path: Option<&'static str>, private: bool) -> Self {
        Self {
            prefix,
            path,
            private,
        }
    }
}

// Parameterized init argument parsing tests
#[parameterized(
    no_args = {
        &["wok", "init"],
        InitExpected::new(None, None, false)
    },
    prefix_only = {
        &["wok", "init", "--prefix", "prj"],
        InitExpected::new(Some("prj"), None, false)
    },
    path_only = {
        &["wok", "init", "--path", "/tmp/test"],
        InitExpected::new(None, Some("/tmp/test"), false)
    },
    prefix_and_path = {
        &["wok", "init", "--prefix", "prj", "--path", "/tmp/test"],
        InitExpected::new(Some("prj"), Some("/tmp/test"), false)
    },
    private_flag = {
        &["wok", "init", "--prefix", "prj", "--private"],
        InitExpected::new(Some("prj"), None, true)
    },
    private_with_path = {
        &["wok", "init", "--private", "--path", "/tmp/test"],
        InitExpected::new(None, Some("/tmp/test"), true)
    },
    all_options = {
        &["wok", "init", "--prefix", "prj", "--path", "/tmp/test", "--private"],
        InitExpected::new(Some("prj"), Some("/tmp/test"), true)
    },
)]
fn should_parse_init_args(args: &[&str], expected: InitExpected) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            private,
        } => {
            assert_eq!(
                prefix.as_deref(),
                expected.prefix,
                "prefix mismatch for {:?}",
                args
            );
            assert_eq!(
                path.as_deref(),
                expected.path,
                "path mismatch for {:?}",
                args
            );
            assert_eq!(
                private, expected.private,
                "private flag mismatch for {:?}",
                args
            );
        }
        _ => panic!("Expected Init command"),
    }
}

// Keep individual test for private mode verification
#[test]
fn should_accept_private_flag() {
    let cli = parse(&["wok", "init", "--prefix", "test", "--private"]).unwrap();
    match cli.command {
        Command::Init { private, .. } => {
            assert!(private, "--private flag should be true");
        }
        _ => panic!("Expected Init command"),
    }
}
