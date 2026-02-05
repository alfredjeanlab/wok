// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for verifying only canonical command names work.
//! Converted from tests/specs/cli/edge_cases/no_aliases.bats
//!
//! Tests verifying that aliases and unsupported commands are rejected.

#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::super::common::*;
use yare::parameterized;

fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(type_).arg(title).arg("-o").arg("id");
    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

// Tests for common aliases that do not exist
#[parameterized(
    create = { "create", &["Test task"] },
    ls = { "ls", &[] },
    add = { "add", &["Test task"] },
    view = { "view", &["test-x"] },
    get = { "get", &["test-x"] },
    begin = { "begin", &["test-x"] },
    finish = { "finish", &["test-x"] },
    complete = { "complete", &["test-x"] },
    modify = { "modify", &["test-x"] },
    update = { "update", &["test-x"] },
    comment = { "comment", &["test-x", "note"] },
    history = { "history", &[] },
    events = { "events", &[] },
    backup = { "backup", &["file.jsonl"] },
    dump = { "dump", &["file.jsonl"] },
)]
fn common_aliases_do_not_exist(cmd: &str, args: &[&str]) {
    let temp = init_temp();
    wk().arg(cmd)
        .args(args)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// Tests for aliases that require issues - rm, del, link, unlink
#[parameterized(
    rm = { "rm" },
    del = { "del" },
    link = { "link" },
    unlink = { "unlink" },
)]
fn aliases_requiring_issues_do_not_exist(cmd: &str) {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");
    wk().arg(cmd)
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .failure();
}

// Tests for unsupported commands
#[parameterized(
    delete = { "delete", &["test-abc"] },
    rm = { "rm", &["test-abc"] },
    remove = { "remove", &["test-abc"] },
    status = { "status", &[] },
    add = { "add", &["Test task"] },
    create = { "create", &["Test task"] },
    open = { "open", &["test-abc"] },
    update = { "update", &["test-abc"] },
    get = { "get", &["test-abc"] },
    push = { "push", &[] },
    pull = { "pull", &[] },
    version = { "version", &[] },
    info = { "info", &[] },
    find = { "find", &["test"] },
    archive = { "archive", &["test-abc"] },
)]
fn unsupported_commands_fail(cmd: &str, args: &[&str]) {
    let temp = init_temp();
    wk().arg(cmd)
        .args(args)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn help_does_not_mention_unsupported_commands() {
    let temp = init_temp();
    let output = wk().arg("help").current_dir(temp.path()).assert().success();

    // These should not appear anywhere in help
    output
        .stdout(predicate::str::contains("delete").not())
        .stdout(predicate::str::contains("create").not())
        .stdout(predicate::str::contains("update").not())
        .stdout(predicate::str::contains("push").not())
        .stdout(predicate::str::contains("pull").not())
        .stdout(predicate::str::contains("archive").not());

    // Check help output doesn't have these as commands (they could appear in descriptions)
    let result = wk().arg("help").current_dir(temp.path()).output().unwrap();
    let stdout = String::from_utf8_lossy(&result.stdout);

    // These need more careful matching - they shouldn't appear as commands (with leading whitespace)
    for line in stdout.lines() {
        let trimmed = line.trim_start();
        // Check that these don't appear as command names at start of lines
        assert!(
            !trimmed.starts_with("add "),
            "help should not list 'add' as a command"
        );
        assert!(
            !trimmed.starts_with("status "),
            "help should not list 'status' as a command"
        );
        assert!(
            !trimmed.starts_with("rm "),
            "help should not list 'rm' as a command"
        );
        assert!(
            !trimmed.starts_with("version "),
            "help should not list 'version' as a command"
        );
        // "remove" can appear in descriptions, only check it doesn't start as a command
        assert!(
            !trimmed.starts_with("remove "),
            "help should not list 'remove' as a command"
        );
    }
}

#[parameterized(
    delete = { "delete" },
    add = { "add" },
    create = { "create" },
    status = { "status" },
    version = { "version" },
)]
fn help_unsupported_fails(cmd: &str) {
    let temp = init_temp();
    wk().arg("help")
        .arg(cmd)
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn canonical_commands_work() {
    let temp = init_temp();

    // new
    wk().args(["new", "task", "Test task"])
        .current_dir(temp.path())
        .assert()
        .success();

    // list
    wk().arg("list").current_dir(temp.path()).assert().success();

    // show, start, done, tree
    let id = create_issue(&temp, "task", "Lifecycle test");
    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["tree", &id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["done", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    // reopen (from done)
    wk().args(["reopen", &id, "--reason", "reopened"])
        .current_dir(temp.path())
        .assert()
        .success();

    // close
    let id2 = create_issue(&temp, "task", "Close test");
    wk().args(["close", &id2, "--reason", "closed"])
        .current_dir(temp.path())
        .assert()
        .success();

    // edit
    wk().args(["edit", &id, "title", "New title"])
        .current_dir(temp.path())
        .assert()
        .success();

    // dep, undep
    let id3 = create_issue(&temp, "task", "A");
    let id4 = create_issue(&temp, "task", "B");
    wk().args(["dep", &id3, "blocks", &id4])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["undep", &id3, "blocks", &id4])
        .current_dir(temp.path())
        .assert()
        .success();

    // label, unlabel
    wk().args(["label", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();
    wk().args(["unlabel", &id, "mylabel"])
        .current_dir(temp.path())
        .assert()
        .success();

    // note
    wk().args(["note", &id, "My note"])
        .current_dir(temp.path())
        .assert()
        .success();

    // log
    wk().arg("log").current_dir(temp.path()).assert().success();

    // export
    let export_path = temp.path().join("export.jsonl");
    wk().args(["export", export_path.to_str().unwrap()])
        .current_dir(temp.path())
        .assert()
        .success();
}
