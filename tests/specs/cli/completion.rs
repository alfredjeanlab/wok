// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Completion command tests - converted from tests/specs/cli/unit/completion.bats
//!
//! BATS test mapping:
//! - "completion command exists and generates valid bash script"
//!   -> completion_bash_generates_valid_script, completion_bash_contains_commands
//! - "completion zsh generates valid script"
//!   -> completion_zsh_generates_valid_script
//! - "completion fish generates valid script"
//!   -> completion_fish_generates_valid_script
//! - "completion error handling"
//!   -> completion_without_shell_shows_help, completion_invalid_shell_fails

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;

fn wk() -> Command {
    cargo_bin_cmd!("wok")
}

// =============================================================================
// Parameterized tests for shell completion generation
// =============================================================================

#[yare::parameterized(
    bash = { "bash" },
    zsh = { "zsh" },
    fish = { "fish" },
)]
fn completion_generates_non_empty_output(shell: &str) {
    let output = wk().args(["completion", shell]).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Completion output should not be empty");
}

// =============================================================================
// Bash completion tests
// =============================================================================

#[test]
fn completion_bash_generates_valid_script() {
    let output = wk().args(["completion", "bash"]).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output is not empty
    assert!(
        !stdout.is_empty(),
        "Bash completion output should not be empty"
    );

    // Verify it's valid bash syntax by checking for bash-specific patterns
    // that indicate a valid completion script
    assert!(
        stdout.contains("complete") || stdout.contains("_wok"),
        "Bash completion should contain completion commands"
    );
}

#[test]
fn completion_bash_contains_commands() {
    let output = wk().args(["completion", "bash"]).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();

    // Should include references to wok commands
    let has_commands = stdout.contains("init")
        || stdout.contains("new")
        || stdout.contains("list")
        || stdout.contains("wok");

    assert!(
        has_commands,
        "Bash completion should reference wok commands"
    );
}

// =============================================================================
// Zsh completion tests
// =============================================================================

#[test]
fn completion_zsh_generates_valid_script() {
    let output = wk().args(["completion", "zsh"]).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output is not empty
    assert!(
        !stdout.is_empty(),
        "Zsh completion output should not be empty"
    );

    // Has zsh-specific syntax
    let has_zsh_syntax = stdout.contains("compdef")
        || stdout.contains("_arguments")
        || stdout.contains("_wok")
        || stdout.contains("#compdef");

    assert!(
        has_zsh_syntax,
        "Zsh completion should contain zsh-specific syntax"
    );
}

// =============================================================================
// Fish completion tests
// =============================================================================

#[test]
fn completion_fish_generates_valid_script() {
    let output = wk().args(["completion", "fish"]).output().unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output is not empty
    assert!(
        !stdout.is_empty(),
        "Fish completion output should not be empty"
    );

    // Has fish-specific syntax
    assert!(
        stdout.contains("complete"),
        "Fish completion should contain 'complete' commands"
    );
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn completion_without_shell_shows_help_or_fails() {
    let output = wk().arg("completion").output().unwrap();

    // Should either show help or fail gracefully with a useful message
    // Both are acceptable behaviors
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // If it fails, it should have some output (help or error message)
    if !output.status.success() {
        assert!(
            !stderr.is_empty() || !stdout.is_empty(),
            "Should provide help or error message when shell type is missing"
        );
    }
}

#[test]
fn completion_invalid_shell_fails() {
    wk().args(["completion", "invalid_shell"])
        .assert()
        .failure();
}
