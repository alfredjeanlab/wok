// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Help command tests - converted from tests/specs/cli/unit/help.bats
//!
//! BATS test mapping:
//! - "wok without arguments shows help and exits 0"
//!   -> wok_without_arguments_shows_help
//! - "help displays usage information and available commands"
//!   -> help_displays_usage_and_commands
//! - "--help and -h flags work"
//!   -> help_and_h_flags_work, h_and_help_produce_same_output
//! - "all commands support -h flag"
//!   -> command_supports_h_flag (parameterized)
//! - "all commands support --help flag"
//!   -> command_supports_help_flag (parameterized)
//! - "help <command> works for all commands"
//!   -> help_subcommand_works (parameterized)
//! - "command help shows usage and options"
//!   -> init_help_shows_options, list_help_shows_options, dep_help_shows_options
//! - "help for unknown command fails gracefully"
//!   -> help_unknown_command_fails
//! - "hidden flags not shown in help"
//!   -> hidden_flags_not_shown_in_new_help, hidden_flags_not_shown_in_main_help

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Phase 1: Basic Help Tests
// =============================================================================

#[test]
fn wok_without_arguments_shows_help() {
    wk().assert()
        .success()
        .stdout(predicate::str::contains("wok"))
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn help_displays_usage_and_commands() {
    let output = wk().arg("help").output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
    assert!(stdout.len() > 100, "Help output should be substantial");

    wk().arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("wok"))
        .stdout(predicate::str::contains("collaborative"))
        .stdout(predicate::str::contains("offline-first"))
        .stdout(predicate::str::contains("AI-friendly"))
        .stdout(predicate::str::contains("issue tracker"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("new"))
        .stdout(predicate::str::contains("list"));
}

// =============================================================================
// Phase 2: Help Flag Tests
// =============================================================================

#[test]
fn help_and_h_flags_work() {
    wk().arg("--help").assert().success().stdout(predicate::str::contains("wok"));

    wk().arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("collaborative"))
        .stdout(predicate::str::contains("issue tracker"));
}

#[test]
fn h_and_help_produce_same_output() {
    let h_output = wk().args(["list", "-h"]).output().unwrap();
    let help_output = wk().args(["list", "--help"]).output().unwrap();

    let h_stdout = String::from_utf8_lossy(&h_output.stdout);
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);

    assert_eq!(h_stdout, help_stdout, "-h and --help should produce identical output");
}

// =============================================================================
// Phase 3: Parameterized Command Help Tests
// =============================================================================

// Note: 'help' subcommand excluded from -h/-help tests - it expects a command name, not flags
#[parameterized(
    init = { "init" },
    new = { "new" },
    start = { "start" },
    done = { "done" },
    close = { "close" },
    reopen = { "reopen" },
    edit = { "edit" },
    list = { "list" },
    show = { "show" },
    tree = { "tree" },
    dep = { "dep" },
    undep = { "undep" },
    label = { "label" },
    unlabel = { "unlabel" },
    note = { "note" },
    log = { "log" },
    export = { "export" },
)]
fn command_supports_h_flag(cmd: &str) {
    wk().args([cmd, "-h"]).assert().success().stdout(predicate::str::contains(cmd));
}

#[parameterized(
    init = { "init" },
    new = { "new" },
    start = { "start" },
    done = { "done" },
    close = { "close" },
    reopen = { "reopen" },
    edit = { "edit" },
    list = { "list" },
    show = { "show" },
    tree = { "tree" },
    dep = { "dep" },
    undep = { "undep" },
    label = { "label" },
    unlabel = { "unlabel" },
    note = { "note" },
    log = { "log" },
    export = { "export" },
)]
fn command_supports_help_flag(cmd: &str) {
    wk().args([cmd, "--help"]).assert().success().stdout(predicate::str::contains(cmd));
}

// help <command> works for all commands including 'help' itself
#[parameterized(
    init = { "init" },
    new = { "new" },
    start = { "start" },
    done = { "done" },
    close = { "close" },
    reopen = { "reopen" },
    edit = { "edit" },
    list = { "list" },
    show = { "show" },
    tree = { "tree" },
    dep = { "dep" },
    undep = { "undep" },
    label = { "label" },
    unlabel = { "unlabel" },
    note = { "note" },
    log = { "log" },
    export = { "export" },
    help = { "help" },
)]
fn help_subcommand_works(cmd: &str) {
    wk().args(["help", cmd]).assert().success().stdout(predicate::str::contains(cmd));
}

// =============================================================================
// Phase 4: Command-Specific Help Content Tests
// =============================================================================

#[test]
fn init_help_shows_options() {
    wk().args(["help", "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("prefix").or(predicate::str::contains("--")));
}

#[test]
fn list_help_shows_options() {
    wk().args(["help", "list"]).assert().success().stdout(
        predicate::str::contains("status")
            .or(predicate::str::contains("tag"))
            .or(predicate::str::contains("type"))
            .or(predicate::str::contains("--")),
    );
}

#[test]
fn dep_help_shows_options() {
    wk().args(["help", "dep"]).assert().success().stdout(
        predicate::str::contains("blocks")
            .or(predicate::str::contains("tracks"))
            .or(predicate::str::contains("relationship")),
    );
}

// =============================================================================
// Phase 5: Error Handling Tests
// =============================================================================

#[test]
fn help_unknown_command_fails() {
    wk().args(["help", "nonexistent"]).assert().failure();
}

// =============================================================================
// Phase 6: Hidden Flags Tests
// =============================================================================

#[test]
fn hidden_flags_not_shown_in_new_help() {
    // --priority and --description hidden in new --help
    wk().args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("priority").not())
        .stdout(predicate::str::contains("--description").not())
        // but --note is shown
        .stdout(predicate::str::contains("--note"))
        .stdout(predicate::str::contains("-n"));

    // hidden in help new too
    wk().args(["help", "new"])
        .assert()
        .success()
        .stdout(predicate::str::contains("priority").not())
        .stdout(predicate::str::contains("--description").not());
}

#[test]
fn hidden_flags_not_shown_in_main_help() {
    wk().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("priority").not())
        .stdout(predicate::str::contains("--description").not());
}
