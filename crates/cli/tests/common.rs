// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

// Allow unused items: test helpers are shared across multiple test binaries,
// and not every test file uses every helper.
#![allow(dead_code)]
#![allow(unused_imports)]

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;

pub use predicates::prelude::*;
pub use tempfile::TempDir;

pub fn wk() -> Command {
    cargo_bin_cmd!("wk")
}

/// Helper to create an initialized temp directory (remote mode, default)
pub fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

/// Helper to create an initialized temp directory in local mode
pub fn init_temp_local() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--local")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

/// Helper to create an issue and return its ID
pub fn create_issue(temp: &TempDir, title: &str) -> String {
    let output = wk()
        .arg("new")
        .arg(title)
        .current_dir(temp.path())
        .output()
        .unwrap();

    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .find(|s| s.starts_with("test-"))
        .unwrap()
        .trim_end_matches(':')
        .to_string()
}
