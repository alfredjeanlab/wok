// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use crate::cli::SchemaCommand;
use crate::schema::{list, ready, search, show};

#[test]
fn schema_list_produces_valid_json() {
    let schema = schemars::schema_for!(list::ListOutputJson);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    assert!(json.contains("\"$schema\""));
    assert!(json.contains("\"ListOutputJson\""));
    assert!(json.contains("\"issues\""));
}

#[test]
fn schema_list_has_required_fields() {
    let schema = schemars::schema_for!(list::ListOutputJson);
    let json = serde_json::to_string(&schema).unwrap();

    // issues is always present (required)
    assert!(json.contains("\"required\""));
}

#[test]
fn schema_show_includes_nested_types() {
    let schema = schemars::schema_for!(show::IssueDetails);
    let json = serde_json::to_string(&schema).unwrap();

    // Should include definitions for nested types
    assert!(json.contains("\"Note\""));
    assert!(json.contains("\"Link\""));
    assert!(json.contains("\"Event\""));
}

#[test]
fn schema_ready_produces_valid_json() {
    let schema = schemars::schema_for!(ready::ReadyOutputJson);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    assert!(json.contains("\"$schema\""));
    assert!(json.contains("\"ReadyOutputJson\""));
    assert!(json.contains("\"issues\""));
}

#[test]
fn schema_search_produces_valid_json() {
    let schema = schemars::schema_for!(search::SearchOutputJson);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    assert!(json.contains("\"$schema\""));
    assert!(json.contains("\"SearchOutputJson\""));
    assert!(json.contains("\"issues\""));
    assert!(json.contains("\"more\""));
}

#[test]
fn all_schema_commands_produce_output() {
    for cmd in [
        SchemaCommand::List,
        SchemaCommand::Show,
        SchemaCommand::Ready,
        SchemaCommand::Search,
    ] {
        // Verify no panic - actual output tested in e2e
        let _schema = match cmd {
            SchemaCommand::List => schemars::schema_for!(list::ListOutputJson),
            SchemaCommand::Show => schemars::schema_for!(show::IssueDetails),
            SchemaCommand::Ready => schemars::schema_for!(ready::ReadyOutputJson),
            SchemaCommand::Search => schemars::schema_for!(search::SearchOutputJson),
        };
    }
}

#[test]
fn schema_issue_json_has_all_fields() {
    let schema = schemars::schema_for!(crate::schema::IssueJson);
    let json = serde_json::to_string(&schema).unwrap();

    // Check all required fields
    assert!(json.contains("\"id\""));
    assert!(json.contains("\"issue_type\""));
    assert!(json.contains("\"status\""));
    assert!(json.contains("\"title\""));
    assert!(json.contains("\"labels\""));
    // Optional field
    assert!(json.contains("\"assignee\""));
}

#[test]
fn schema_show_has_datetime_fields() {
    let schema = schemars::schema_for!(show::IssueDetails);
    let json = serde_json::to_string(&schema).unwrap();

    assert!(json.contains("\"created_at\""));
    assert!(json.contains("\"updated_at\""));
    assert!(json.contains("\"closed_at\""));
    // DateTime should produce format: date-time
    assert!(json.contains("\"date-time\""));
}

#[test]
fn schema_issue_type_has_all_variants() {
    let schema = schemars::schema_for!(crate::schema::IssueType);
    let json = serde_json::to_string(&schema).unwrap();

    assert!(json.contains("\"feature\""));
    assert!(json.contains("\"task\""));
    assert!(json.contains("\"bug\""));
    assert!(json.contains("\"chore\""));
    assert!(json.contains("\"idea\""));
}

#[test]
fn schema_status_has_all_variants() {
    let schema = schemars::schema_for!(crate::schema::Status);
    let json = serde_json::to_string(&schema).unwrap();

    assert!(json.contains("\"todo\""));
    assert!(json.contains("\"in_progress\""));
    assert!(json.contains("\"done\""));
    assert!(json.contains("\"closed\""));
}
