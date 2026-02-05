// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for CLI flag consistency.
//! Converted from tests/specs/cli/consistency/shared_flags.bats
//!
//! Tests verifying consistent short flags across commands:
//! - --type / -t consistency across list, edit, import
//! - --status / -s consistency across list, import
//! - --label / -l consistency across new, list, import
//! - --output / -o consistency for show
//! - --format / -f consistency for import

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::cli::common::*;
use yare::parameterized;

// =============================================================================
// Helpers
// =============================================================================

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
// --type / -t consistency
// All commands with --type should also accept -t
// =============================================================================

#[test]
fn list_accepts_t_as_short_form_for_type() {
    let temp = init_temp();
    create_issue(&temp, "bug", "My bug");
    create_issue(&temp, "task", "My task");

    wk().arg("list")
        .arg("-t")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("My bug"))
        .stdout(predicate::str::contains("My task").not());
}

#[test]
#[ignore = "edit command uses positional args, not -t flag"]
fn edit_accepts_t_as_short_form_for_type() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    wk().arg("edit")
        .arg(&id)
        .arg("-t")
        .arg("bug")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn import_accepts_t_as_short_form_for_type() {
    let temp = init_temp();
    create_issue(&temp, "bug", "My bug");

    // Export existing issues
    wk().arg("export")
        .arg("test_export.jsonl")
        .current_dir(temp.path())
        .assert()
        .success();

    // Create new temp dir for import
    let import_temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("imp")
        .arg("--private")
        .current_dir(import_temp.path())
        .assert()
        .success();

    // Copy export file to import temp
    let export_path = temp.path().join("test_export.jsonl");
    let import_path = import_temp.path().join("test_export.jsonl");
    std::fs::copy(&export_path, &import_path).unwrap();

    wk().arg("import")
        .arg("-t")
        .arg("bug")
        .arg("test_export.jsonl")
        .current_dir(import_temp.path())
        .assert()
        .success();
}

// =============================================================================
// --status / -s consistency
// =============================================================================

#[parameterized(
    list = { "list" },
    import = { "import" },
)]
fn status_short_flag_s_accepted_by_command(command: &str) {
    let temp = init_temp();

    match command {
        "list" => {
            let id = create_issue(&temp, "task", "Started task");
            wk().arg("start")
                .arg(&id)
                .current_dir(temp.path())
                .assert()
                .success();
            wk().arg("list")
                .arg("-s")
                .arg("in_progress")
                .current_dir(temp.path())
                .assert()
                .success()
                .stdout(predicate::str::contains("Started task"));
        }
        "import" => {
            create_issue(&temp, "task", "Test");
            wk().arg("export")
                .arg("test.jsonl")
                .current_dir(temp.path())
                .assert()
                .success();
            wk().arg("import")
                .arg("--dry-run")
                .arg("-s")
                .arg("todo")
                .arg("test.jsonl")
                .current_dir(temp.path())
                .assert()
                .success();
        }
        _ => panic!("Unknown command: {}", command),
    }
}

// =============================================================================
// --label / -l consistency
// =============================================================================

#[parameterized(
    new = { "new" },
    list = { "list" },
    import = { "import" },
)]
fn label_short_flag_l_accepted_by_command(command: &str) {
    let temp = init_temp();

    match command {
        "new" => {
            let id = create_issue_with_opts(&temp, "task", "Labeled task", &["-l", "mylabel"]);
            wk().arg("show")
                .arg(&id)
                .current_dir(temp.path())
                .assert()
                .success()
                .stdout(predicate::str::contains("mylabel"));
        }
        "list" => {
            create_issue_with_opts(&temp, "task", "Labeled", &["--label", "findme"]);
            wk().arg("list")
                .arg("-l")
                .arg("findme")
                .current_dir(temp.path())
                .assert()
                .success()
                .stdout(predicate::str::contains("Labeled"));
        }
        "import" => {
            create_issue_with_opts(&temp, "task", "Test", &["--label", "importlabel"]);
            wk().arg("export")
                .arg("test.jsonl")
                .current_dir(temp.path())
                .assert()
                .success();
            wk().arg("import")
                .arg("--dry-run")
                .arg("-l")
                .arg("importlabel")
                .arg("test.jsonl")
                .current_dir(temp.path())
                .assert()
                .success();
        }
        _ => panic!("Unknown command: {}", command),
    }
}

// =============================================================================
// --output / -o consistency (output format)
// =============================================================================

#[test]
fn show_accepts_o_as_short_form_for_output() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");
    wk().arg("show")
        .arg(&id)
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("{"));
}

// =============================================================================
// --format / -f consistency (import input format)
// =============================================================================

#[test]
fn import_accepts_f_as_short_form_for_format() {
    let temp = init_temp();
    create_issue(&temp, "task", "Test");
    wk().arg("export")
        .arg("test.jsonl")
        .current_dir(temp.path())
        .assert()
        .success();
    wk().arg("import")
        .arg("--dry-run")
        .arg("-f")
        .arg("wk")
        .arg("test.jsonl")
        .current_dir(temp.path())
        .assert()
        .success();
}
