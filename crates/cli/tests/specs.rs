// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust-based integration specs for the wk CLI.
//!
//! Run with: cargo test --test specs
//!
//! These complement the BATS specs in tests/specs/ and are useful for:
//! - Tests requiring complex setup or teardown
//! - Tests that benefit from Rust's type system
//! - Performance-sensitive test suites

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

#[path = "specs_prelude.rs"]
mod prelude;

use prelude::*;

#[test]
fn smoke_test_wk_version() {
    let output = Wk::new().arg("--version").output();
    output.success().stdout(predicates::str::contains("wok"));
}

#[test]
fn smoke_test_wk_help() {
    let output = Wk::new().arg("--help").output();
    output.success().stdout(predicates::str::contains("Usage:"));
}

#[test]
fn project_lifecycle() {
    let project = Project::new("demo");

    // Create an issue
    let id = project.create_issue("task", "Integration test");
    assert!(id.starts_with("demo-"));

    // List shows the issue
    project
        .wk()
        .arg("list")
        .output()
        .success()
        .stdout(predicates::str::contains(&id));

    // Show displays details
    project
        .wk()
        .args(["show", &id])
        .output()
        .success()
        .stdout(predicates::str::contains("Integration test"));

    // Start the issue
    project
        .wk()
        .args(["start", &id])
        .output()
        .success();

    // Complete the issue
    project
        .wk()
        .args(["done", &id])
        .output()
        .success();
}
