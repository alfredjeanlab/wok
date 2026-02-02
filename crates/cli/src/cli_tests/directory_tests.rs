// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use clap::Parser;

#[test]
fn parse_dash_c_before_subcommand() {
    let cli = Cli::try_parse_from(["wk", "-C", "/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_dash_c_with_equals() {
    let cli = Cli::try_parse_from(["wk", "-C=/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_dash_c_no_space() {
    let cli = Cli::try_parse_from(["wk", "-C/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_long_directory_flag() {
    let cli = Cli::try_parse_from(["wk", "--directory", "/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_long_directory_with_equals() {
    let cli = Cli::try_parse_from(["wk", "--directory=/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_no_dash_c() {
    let cli = Cli::try_parse_from(["wk", "list"]).unwrap();
    assert_eq!(cli.directory, None);
}

#[test]
fn parse_dash_c_after_subcommand() {
    // global = true allows -C after the subcommand
    let cli = Cli::try_parse_from(["wk", "list", "-C", "/tmp"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}
