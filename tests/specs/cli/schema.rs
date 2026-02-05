// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk schema` command.
//! Converted from tests/specs/cli/unit/schema.bats
//!
//! BATS test mapping:
//! - "schema requires subcommand"
//!   -> schema_requires_subcommand
//! - "schema list outputs valid JSON"
//!   -> schema_list_outputs_valid_json
//! - "schema list contains expected structure"
//!   -> schema_list_contains_expected_structure
//! - "schema show outputs valid JSON"
//!   -> schema_show_outputs_valid_json
//! - "schema show includes nested types"
//!   -> schema_show_includes_nested_types
//! - "schema ready outputs valid JSON"
//!   -> schema_ready_outputs_valid_json
//! - "schema search outputs valid JSON"
//!   -> schema_search_outputs_valid_json
//! - "schema search is array type"
//!   -> schema_search_is_array_type
//! - "all schemas have $schema field"
//!   -> schema_subcommand_has_schema_field (parameterized)
//! - "schema -h shows help"
//!   -> schema_h_shows_help
//! - "schema help shows examples"
//!   -> schema_help_shows_examples
//! - "schema list includes issue type enum values"
//!   -> schema_list_includes_issue_type_enum_values
//! - "schema list includes status enum values"
//!   -> schema_list_includes_status_enum_values
//! - "schema show includes datetime format"
//!   -> schema_show_includes_datetime_format

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::common::*;
use yare::parameterized;

// =============================================================================
// Basic Schema Command Tests
// =============================================================================

#[test]
fn schema_requires_subcommand() {
    wk().arg("schema")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn schema_list_outputs_valid_json() {
    let output = wk().args(["schema", "list"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("schema list should output valid JSON");
}

#[test]
fn schema_list_contains_expected_structure() {
    wk().args(["schema", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""$schema""#))
        .stdout(predicate::str::contains(r#""array""#))
        .stdout(predicate::str::contains(r#""IssueJson""#));
}

#[test]
fn schema_show_outputs_valid_json() {
    let output = wk().args(["schema", "show"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("schema show should output valid JSON");
}

#[test]
fn schema_show_includes_nested_types() {
    wk().args(["schema", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""Note""#))
        .stdout(predicate::str::contains(r#""Link""#))
        .stdout(predicate::str::contains(r#""Event""#));
}

#[test]
fn schema_ready_outputs_valid_json() {
    let output = wk().args(["schema", "ready"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("schema ready should output valid JSON");
}

#[test]
fn schema_search_outputs_valid_json() {
    let output = wk().args(["schema", "search"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<serde_json::Value>(&stdout)
        .expect("schema search should output valid JSON");
}

#[test]
fn schema_search_is_array_type() {
    wk().args(["schema", "search"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""array""#))
        .stdout(predicate::str::contains(r#""IssueJson""#));
}

// =============================================================================
// Parameterized Schema Tests
// =============================================================================

#[parameterized(
    list = { "list" },
    show = { "show" },
    ready = { "ready" },
    search = { "search" },
)]
fn schema_subcommand_has_schema_field(subcmd: &str) {
    wk().args(["schema", subcmd])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""$schema""#));
}

// =============================================================================
// Help Tests
// =============================================================================

#[test]
fn schema_h_shows_help() {
    wk().args(["schema", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("ready"))
        .stdout(predicate::str::contains("search"));
}

#[test]
fn schema_help_shows_examples() {
    wk().args(["schema", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wok schema list"))
        .stdout(predicate::str::contains("wok schema show"));
}

// =============================================================================
// Schema Content Tests
// =============================================================================

#[test]
fn schema_list_includes_issue_type_enum_values() {
    wk().args(["schema", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""feature""#))
        .stdout(predicate::str::contains(r#""task""#))
        .stdout(predicate::str::contains(r#""bug""#))
        .stdout(predicate::str::contains(r#""chore""#))
        .stdout(predicate::str::contains(r#""idea""#));
}

#[test]
fn schema_list_includes_status_enum_values() {
    wk().args(["schema", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""todo""#))
        .stdout(predicate::str::contains(r#""in_progress""#))
        .stdout(predicate::str::contains(r#""done""#))
        .stdout(predicate::str::contains(r#""closed""#));
}

#[test]
fn schema_show_includes_datetime_format() {
    wk().args(["schema", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""date-time""#));
}
