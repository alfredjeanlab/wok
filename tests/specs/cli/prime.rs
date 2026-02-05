// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk prime` command.
//! Converted from tests/specs/cli/unit/prime.bats
//!
//! BATS test mapping:
//! - "prime outputs template without initialization"
//!   -> outputs_template_without_initialization
//! - "prime outputs template content"
//!   -> outputs_template_content
//! - "prime output contains wok commands"
//!   -> output_contains_wok_command (parameterized)
//! - "prime documents list default shows open items"
//!   -> documents_list_default_shows_open_items
//! - "prime output is not empty when initialized"
//!   -> output_is_not_empty_when_initialized
//! - "prime output contains workflow sections"
//!   -> output_contains_workflow_section (parameterized)
//! - "prime help shows description"
//!   -> help_shows_description
//! - "prime output is valid markdown"
//!   -> output_is_valid_markdown
//! - "prime with --help shows usage"
//!   -> help_shows_usage (parameterized)
//! - "prime ignores extra arguments"
//!   -> ignores_extra_arguments_or_errors
//! - "prime works from subdirectory"
//!   -> works_from_subdirectory

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Basic Output Tests
// =============================================================================

#[test]
fn outputs_template_without_initialization() {
    // prime should always output template content even without .wok
    let temp = TempDir::new().unwrap();

    // Verify .wok does not exist
    assert!(!temp.path().join(".wok").exists());

    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("## Core Rules"));
}

#[test]
fn outputs_template_content() {
    let temp = init_temp();

    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("## Core Rules"))
        .stdout(predicate::str::contains("## Finding Work"));
}

#[test]
fn output_is_not_empty_when_initialized() {
    let temp = init_temp();

    let output = wk().arg("prime").current_dir(temp.path()).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "prime output should not be empty");
}

// =============================================================================
// Content Tests - Parameterized
// =============================================================================

#[parameterized(
    wok_list = { "wok list" },
    wok_new = { "wok new" },
    wok_start = { "wok start" },
    wok_done = { "wok done" },
)]
fn output_contains_wok_command(command: &str) {
    let temp = init_temp();

    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(command));
}

#[parameterized(
    creating_updating = { "## Creating & Updating" },
    dependencies = { "## Dependencies" },
    common_workflows = { "## Common Workflows" },
)]
fn output_contains_workflow_section(section: &str) {
    let temp = init_temp();

    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(section));
}

#[test]
fn documents_list_default_shows_open_items() {
    let temp = init_temp();

    // Verify documentation says list shows todo AND in_progress by default
    wk().arg("prime")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("todo + in_progress"));
}

// =============================================================================
// Markdown Validity Tests
// =============================================================================

#[test]
fn output_is_valid_markdown() {
    let temp = init_temp();

    let output = wk().arg("prime").current_dir(temp.path()).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for proper markdown header syntax
    assert!(
        stdout.contains("# "),
        "Output should contain markdown headers"
    );

    // Check code blocks are properly opened and closed (count must be even)
    let count = stdout.matches("```").count();
    assert!(
        count % 2 == 0,
        "Unbalanced code blocks: {} (should be even)",
        count
    );
}

// =============================================================================
// Help Tests - Parameterized
// =============================================================================

#[test]
fn help_shows_description() {
    wk().args(["prime", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("onboarding"));
}

#[parameterized(
    usage = { "Usage:" },
    prime = { "prime" },
    help_flag = { "-h, --help" },
)]
fn help_shows_usage(expected: &str) {
    wk().args(["prime", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn ignores_extra_arguments_or_errors() {
    // prime should succeed even with extra arguments (clap ignores them or errors gracefully)
    let temp = init_temp();

    let output = wk()
        .args(["prime", "extra", "arg1", "arg2"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    // Either succeeds (ignores args) or fails gracefully with exit code 2 (clap error)
    assert!(
        output.status.success() || output.status.code() == Some(2),
        "Expected success or clap error (exit code 2), got: {:?}",
        output.status.code()
    );
}

#[test]
fn works_from_subdirectory() {
    let temp = init_temp();

    // Create nested subdirectory
    let subdir = temp.path().join("subdir/nested");
    std::fs::create_dir_all(&subdir).unwrap();

    wk().arg("prime")
        .current_dir(&subdir)
        .assert()
        .success()
        .stdout(predicate::str::contains("## Core Rules"));
}
