// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for the `wk export` command.
//! Converted from tests/specs/cli/unit/export.bats

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use std::fs;

use super::common::*;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    create_issue_with_opts(temp, type_, title, &[])
}

fn create_issue_with_opts(temp: &TempDir, type_: &str, title: &str, opts: &[&str]) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title);
    for opt in opts {
        cmd.arg(opt);
    }
    cmd.arg("-o").arg("id");

    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// =============================================================================
// Basic export functionality
// =============================================================================

#[test]
fn export_creates_file() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportBasic Test task");

    let export_path = temp.path().join("export.jsonl");

    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    assert!(export_path.exists(), "Export file should be created");
}

#[test]
fn export_produces_valid_jsonl() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportBasic JSONL task");

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    for line in content.lines() {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|_| panic!("Invalid JSON: {}", line));
    }
}

#[test]
fn export_empty_database_succeeds() {
    let temp = init_temp();
    let export_path = temp.path().join("export.jsonl");

    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();
}

#[test]
fn export_requires_filepath() {
    let temp = init_temp();

    wk().arg("export").current_dir(temp.path()).assert().failure();
}

// =============================================================================
// Export includes issue data
// =============================================================================

#[test]
fn export_includes_title() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportData My test task");

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("My test task"), "Export should contain issue title");
}

#[test]
fn export_includes_all_issue_types() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportData Task 1");
    create_issue(&temp, "bug", "ExportData Bug 1");
    create_issue(&temp, "feature", "ExportData Feature 1");

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("Task 1"), "Export should contain task");
    assert!(content.contains("Bug 1"), "Export should contain bug");
    assert!(content.contains("Feature 1"), "Export should contain feature");
}

#[test]
fn export_includes_issue_type() {
    let temp = init_temp();
    create_issue(&temp, "bug", "ExportData Test bug");

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("bug"), "Export should contain issue type");
}

#[test]
fn export_includes_issue_status() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ExportData Status task");
    wk().arg("start").arg(&id).current_dir(temp.path()).assert().success();

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("in_progress"), "Export should contain status");
}

#[test]
fn export_includes_labels() {
    let temp = init_temp();
    create_issue_with_opts(&temp, "task", "ExportData Labeled task", &["--label", "mylabel"]);

    let export_path = temp.path().join("export.jsonl");
    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("mylabel"), "Export should contain label");
}

#[test]
fn export_overwrites_existing_file() {
    let temp = init_temp();
    let export_path = temp.path().join("export.jsonl");

    fs::write(&export_path, "old content").unwrap();
    create_issue(&temp, "task", "ExportData New task");

    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.contains("old content"), "Export should overwrite existing file");
}

// =============================================================================
// Path format tests
// =============================================================================

#[test]
fn export_accepts_absolute_path() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportPath Test task");

    let export_path = temp.path().join("absolute_export.jsonl");

    wk().arg("export").arg(&export_path).current_dir(temp.path()).assert().success();

    assert!(export_path.exists(), "Export should create file at absolute path");
}

#[test]
fn export_accepts_relative_path() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportPath Test task");

    wk().arg("export").arg("export.jsonl").current_dir(temp.path()).assert().success();

    assert!(
        temp.path().join("export.jsonl").exists(),
        "Export should create file at relative path"
    );
}

#[test]
fn export_accepts_subdirectory_path() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportPath Test task");

    fs::create_dir(temp.path().join("subdir")).unwrap();

    wk().arg("export").arg("subdir/export.jsonl").current_dir(temp.path()).assert().success();

    assert!(
        temp.path().join("subdir/export.jsonl").exists(),
        "Export should create file in subdirectory"
    );
}

#[test]
fn export_accepts_dotdot_path() {
    let temp = init_temp();
    create_issue(&temp, "task", "ExportPath Test task");

    fs::create_dir(temp.path().join("subdir")).unwrap();

    wk().arg("export").arg("subdir/../export2.jsonl").current_dir(temp.path()).assert().success();

    assert!(temp.path().join("export2.jsonl").exists(), "Export should resolve .. in path");
}
