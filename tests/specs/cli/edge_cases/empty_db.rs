// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Empty database edge case tests - converted from tests/specs/cli/edge_cases/empty_db.bats
//!
//! Tests verifying commands handle empty database gracefully.
//!
//! BATS test mapping:
//! - "read commands succeed on empty database"
//!   -> read_command_succeeds_on_empty_db (parameterized)
//! - "export succeeds on empty database"
//!   -> export_succeeds_on_empty_db
//! - "commands on nonexistent issues fail"
//!   -> command_on_nonexistent_issue_fails (parameterized)

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;
use yare::parameterized;

// =============================================================================
// Read Commands Succeed on Empty Database
// =============================================================================

#[parameterized(
    list = { &["list"] },
    list_blocked = { &["list", "--blocked"] },
    list_all = { &["list", "--all"] },
    list_status_in_progress = { &["list", "--status", "in_progress"] },
    list_type_bug = { &["list", "--type", "bug"] },
    list_label_nonexistent = { &["list", "--label", "nonexistent"] },
    log = { &["log"] },
)]
fn read_command_succeeds_on_empty_db(args: &[&str]) {
    let temp = init_temp();
    wk().args(args).current_dir(temp.path()).assert().success();
}

// =============================================================================
// Export Succeeds on Empty Database
// =============================================================================

#[test]
fn export_succeeds_on_empty_db() {
    let temp = init_temp();
    let export_path = temp.path().join("empty.jsonl");
    wk().args(["export", export_path.to_str().unwrap()])
        .current_dir(temp.path())
        .assert()
        .success();
}

// =============================================================================
// Commands on Nonexistent Issues Fail
// =============================================================================

#[parameterized(
    show = { &["show", "test-nonexistent"] },
    start = { &["start", "test-nonexistent"] },
)]
fn command_on_nonexistent_issue_fails(args: &[&str]) {
    let temp = init_temp();
    wk().args(args).current_dir(temp.path()).assert().failure();
}
