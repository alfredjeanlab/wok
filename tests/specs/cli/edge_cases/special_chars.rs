// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust specs for special character handling in titles, notes, and labels.
//! Converted from tests/specs/cli/edge_cases/special_chars.bats
//!
//! Tests verifying that various special characters, unicode, and edge cases
//! are handled correctly.

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

#[test]
fn title_with_double_quotes() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg(r#"Fix the "bug" issue"#)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""bug""#));
}

#[test]
fn title_with_single_quotes() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("User's profile page")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("User's"));
}

#[test]
fn title_with_backticks() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Fix `code` formatting")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn title_with_unicode_emoji() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Fix emoji handling 🚀")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("🚀"));
}

#[test]
fn title_with_unicode_characters() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Café résumé naïve")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Café"));
}

#[test]
fn title_with_cjk_characters() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("日本語タイトル")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("日本語"));
}

#[test]
fn title_with_special_shell_characters() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Task with $dollar and &ampersand")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn title_with_parentheses_and_brackets() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Fix (critical) [bug] issue")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("(critical)"))
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn note_with_newlines_preserved() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");

    wk().arg("note")
        .arg(&id)
        .arg("Line 1\nLine 2\nLine 3")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn note_with_special_characters() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");

    wk().arg("note")
        .arg(&id)
        .arg(r#"Note with "quotes" and 'apostrophes'"#)
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn label_with_colon() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test");

    wk().arg("label").arg(&id).arg("namespace:value").current_dir(temp.path()).assert().success();

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("namespace:value"));
}

#[test]
fn title_with_leading_trailing_whitespace() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("  Trimmed title  ")
        .current_dir(temp.path())
        .assert()
        .success();
}

#[test]
fn very_long_title() {
    let temp = init_temp();
    let long_title = "x".repeat(500);
    wk().arg("new").arg("task").arg(&long_title).current_dir(temp.path()).assert().success();
}

#[test]
fn title_with_only_numbers() {
    let temp = init_temp();
    wk().arg("new").arg("task").arg("12345").current_dir(temp.path()).assert().success();
}

#[test]
fn hyphen_in_title() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("Fix foo-bar-baz issue")
        .current_dir(temp.path())
        .assert()
        .success();
}

// Parameterized tests for unicode character handling
#[parameterized(
    emoji = { "Fix 🚀 issue", "🚀" },
    accented = { "Café résumé", "Café" },
    cjk = { "日本語タイトル", "日本語" },
    cyrillic = { "Привет мир", "Привет" },
    arabic = { "مرحبا بالعالم", "مرحبا" },
)]
fn unicode_in_title_preserved(title: &str, expected: &str) {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg(title)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}

// Parameterized tests for various special characters in titles
#[parameterized(
    percent = { "100% complete" },
    at_sign = { "user@example.com" },
    hash = { "Issue #123" },
    asterisk = { "Important * item" },
    tilde = { "Home ~/ directory" },
    pipe = { "A | B" },
    less_greater = { "a < b > c" },
)]
fn special_chars_in_title(title: &str) {
    let temp = init_temp();
    wk().arg("new").arg("task").arg(title).current_dir(temp.path()).assert().success();
}
