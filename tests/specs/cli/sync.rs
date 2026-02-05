// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Remote/sync command tests - converted from tests/specs/cli/unit/sync.bats
//!
//! BATS test mapping:
//! - "remote status in local mode shows not applicable"
//!   -> remote_status_in_local_mode_shows_not_applicable
//! - "remote sync in local mode is silent"
//!   -> remote_sync_in_local_mode_is_silent
//! - "remote status provides configuration hint"
//!   -> remote_status_provides_configuration_hint
//! - "remote help shows subcommands"
//!   -> remote_help_shows_subcommands (parameterized: help remote, remote --help)
//! - "remote --help shows subcommands"
//!   -> remote_help_shows_subcommands (parameterized)
//! - "remote appears in main help"
//!   -> remote_appears_in_main_help

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Remote Status Tests (Local Mode)
// =============================================================================

#[test]
#[ignore = "remote subcommand not yet implemented"]
fn remote_status_in_local_mode_shows_not_applicable() {
    let temp = init_temp();

    wk().args(["remote", "status"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not"));
}

#[test]
#[ignore = "remote subcommand not yet implemented"]
fn remote_sync_in_local_mode_is_silent() {
    let temp = init_temp_private();

    wk().args(["remote", "sync"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
#[ignore = "remote subcommand not yet implemented"]
fn remote_status_provides_configuration_hint() {
    let temp = init_temp_private();

    wk().args(["remote", "status"])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[remote]"))
        .stdout(predicate::str::contains("url"));
}

// =============================================================================
// Remote Help Tests
// =============================================================================

#[ignore = "remote subcommand not yet implemented"]
#[parameterized(
    help_remote = { &["help", "remote"] },
    remote_help_flag = { &["remote", "--help"] },
)]
fn remote_help_shows_subcommands(args: &[&str]) {
    wk().args(args)
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("sync"));
}

#[test]
#[ignore = "remote subcommand not yet implemented"]
fn remote_appears_in_main_help() {
    wk().arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("remote"));
}
