// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! CLI argument parsing tests for the `init` command.
//!
//! These tests verify that clap correctly parses all init command options:
//! - `--prefix` - Custom prefix for issue IDs
//! - `--path` - Target directory for initialization
//! - `--workspace` - Link to an existing workspace
//! - `--remote` - Set up remote sync configuration
//! - `--local` - Backwards compatibility flag (no-op)

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
    workspace: Option<&'static str>,
    remote: Option<&'static str>,
    local: bool,
}

impl InitExpected {
    const fn new(
        prefix: Option<&'static str>,
        path: Option<&'static str>,
        workspace: Option<&'static str>,
        remote: Option<&'static str>,
        local: bool,
    ) -> Self {
        Self {
            prefix,
            path,
            workspace,
            remote,
            local,
        }
    }
}

// Parameterized init argument parsing tests
#[parameterized(
    no_args = {
        &["wk", "init"],
        InitExpected::new(None, None, None, None, false)
    },
    prefix_only = {
        &["wk", "init", "--prefix", "prj"],
        InitExpected::new(Some("prj"), None, None, None, false)
    },
    path_only = {
        &["wk", "init", "--path", "/tmp/test"],
        InitExpected::new(None, Some("/tmp/test"), None, None, false)
    },
    workspace_only = {
        &["wk", "init", "--workspace", "/some/workspace"],
        InitExpected::new(None, None, Some("/some/workspace"), None, false)
    },
    remote_only = {
        &["wk", "init", "--remote", "."],
        InitExpected::new(None, None, None, Some("."), false)
    },
    prefix_and_path = {
        &["wk", "init", "--prefix", "prj", "--path", "/tmp/test"],
        InitExpected::new(Some("prj"), Some("/tmp/test"), None, None, false)
    },
    workspace_and_prefix = {
        &["wk", "init", "--workspace", "/some/workspace", "--prefix", "prj"],
        InitExpected::new(Some("prj"), None, Some("/some/workspace"), None, false)
    },
    workspace_and_path = {
        &["wk", "init", "--workspace", "/some/workspace", "--path", "/target"],
        InitExpected::new(None, Some("/target"), Some("/some/workspace"), None, false)
    },
    all_options = {
        &["wk", "init", "--prefix", "prj", "--workspace", "/ws", "--remote", "."],
        InitExpected::new(Some("prj"), None, Some("/ws"), Some("."), false)
    },
    local_flag = {
        &["wk", "init", "--prefix", "prj", "--local"],
        InitExpected::new(Some("prj"), None, None, None, true)
    },
    remote_with_url = {
        &["wk", "init", "--prefix", "prj", "--remote", "git@github.com:user/repo.git"],
        InitExpected::new(Some("prj"), None, None, Some("git@github.com:user/repo.git"), false)
    },
)]
fn should_parse_init_args(args: &[&str], expected: InitExpected) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            local,
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
                workspace.as_deref(),
                expected.workspace,
                "workspace mismatch for {:?}",
                args
            );
            assert_eq!(
                remote.as_deref(),
                expected.remote,
                "remote mismatch for {:?}",
                args
            );
            assert_eq!(local, expected.local, "local flag mismatch for {:?}", args);
        }
        _ => panic!("Expected Init command"),
    }
}

// Keep individual test for backwards compatibility verification
#[test]
fn should_accept_local_flag_for_backwards_compatibility() {
    // --local flag should parse without error (backwards compatibility)
    let cli = parse(&["wk", "init", "--prefix", "test", "--local"]).unwrap();
    match cli.command {
        Command::Init { local, .. } => {
            assert!(local, "--local flag should be true");
        }
        _ => panic!("Expected Init command"),
    }
}
