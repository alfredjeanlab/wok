// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Helper to parse CLI args
fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(args)
}

// Init command tests
#[test]
fn test_init_with_prefix() {
    let cli = parse(&["wk", "init", "--prefix", "prj"]).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert_eq!(prefix, Some("prj".to_string()));
            assert!(path.is_none());
            assert!(workspace.is_none());
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_init_with_path() {
    let cli = parse(&["wk", "init", "--prefix", "prj", "--path", "/tmp/test"]).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert_eq!(prefix, Some("prj".to_string()));
            assert_eq!(path, Some("/tmp/test".to_string()));
            assert!(workspace.is_none());
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_init_without_prefix() {
    // Prefix is now optional - will be derived from directory name
    let cli = parse(&["wk", "init"]).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert!(prefix.is_none());
            assert!(path.is_none());
            assert!(workspace.is_none());
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_init_with_workspace() {
    let cli = parse(&["wk", "init", "--workspace", "/some/workspace"]).unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert!(prefix.is_none());
            assert!(path.is_none());
            assert_eq!(workspace, Some("/some/workspace".to_string()));
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_init_with_workspace_and_prefix() {
    let cli = parse(&[
        "wk",
        "init",
        "--workspace",
        "/some/workspace",
        "--prefix",
        "prj",
    ])
    .unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert_eq!(prefix, Some("prj".to_string()));
            assert!(path.is_none());
            assert_eq!(workspace, Some("/some/workspace".to_string()));
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_init_with_workspace_and_path() {
    let cli = parse(&[
        "wk",
        "init",
        "--workspace",
        "/some/workspace",
        "--path",
        "/target",
    ])
    .unwrap();
    match cli.command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            ..
        } => {
            assert!(prefix.is_none());
            assert_eq!(path, Some("/target".to_string()));
            assert_eq!(workspace, Some("/some/workspace".to_string()));
            assert!(remote.is_none());
        }
        _ => panic!("Expected Init command"),
    }
}
