// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Version flag tests - converted from tests/specs/cli/unit/version.bats
//!
//! BATS test mapping:
//! - "--version outputs version"
//!   -> version_flag_outputs_version (parameterized: --version, -v, -V)
//! - "-v outputs version"
//!   -> version_flag_outputs_version (parameterized)
//! - "-V outputs version (silent alias)"
//!   -> version_flag_outputs_version (parameterized)
//! - "-v and --version produce identical output"
//!   -> v_and_version_produce_identical_output
//! - "-V produces same output as -v"
//!   -> big_v_produces_same_output_as_small_v
//! - "-v is documented in help"
//!   -> version_flags_documented_in_help
//! - "-V is NOT documented in help"
//!   -> big_v_not_documented_in_help
//! - "version subcommand does not exist"
//!   -> version_subcommand_does_not_exist

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Phase 1: Version Flag Output Tests
// =============================================================================

#[parameterized(
    long_version = { "--version" },
    short_v = { "-v" },
    silent_v = { "-V" },
)]
fn version_flag_outputs_version(flag: &str) {
    wk().arg(flag)
        .assert()
        .success()
        .stdout(predicate::str::contains("wok"))
        .stdout(predicate::str::is_match(r"[0-9]+\.[0-9]+\.[0-9]+").unwrap());
}

// =============================================================================
// Phase 2: Output Equivalence Tests
// =============================================================================

#[test]
fn v_and_version_produce_identical_output() {
    let v_output = wk().arg("-v").output().unwrap();
    let version_output = wk().arg("--version").output().unwrap();

    let v_stdout = String::from_utf8_lossy(&v_output.stdout);
    let version_stdout = String::from_utf8_lossy(&version_output.stdout);

    assert_eq!(
        v_stdout, version_stdout,
        "-v and --version should produce identical output"
    );
}

#[test]
fn big_v_produces_same_output_as_small_v() {
    let v_output = wk().arg("-v").output().unwrap();
    let big_v_output = wk().arg("-V").output().unwrap();

    let v_stdout = String::from_utf8_lossy(&v_output.stdout);
    let big_v_stdout = String::from_utf8_lossy(&big_v_output.stdout);

    assert_eq!(
        v_stdout, big_v_stdout,
        "-V and -v should produce identical output"
    );
}

// =============================================================================
// Phase 3: Help Documentation Tests
// =============================================================================

#[test]
fn version_flags_documented_in_help() {
    wk().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("-v"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn big_v_not_documented_in_help() {
    let output = wk().arg("--help").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // -V should be hidden: check for patterns like " -V," or " -V " or "[-V"
    assert!(
        !stdout.contains(" -V,"),
        "-V should not be documented in help"
    );
    assert!(
        !stdout.contains(" -V "),
        "-V should not be documented in help"
    );
    assert!(
        !stdout.contains("[-V"),
        "-V should not be documented in help"
    );
}

// =============================================================================
// Phase 4: Negative Tests
// =============================================================================

#[test]
fn version_subcommand_does_not_exist() {
    wk().arg("version").assert().failure();
}
