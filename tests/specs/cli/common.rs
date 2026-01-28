// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

// Allow unused items: test helpers are shared across multiple test files,
// and not every test file uses every helper.
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(clippy::panic)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::Command;

pub use predicates::prelude::*;
pub use tempfile::TempDir;

pub fn wk() -> Command {
    #[allow(deprecated)]
    Command::cargo_bin("wk").unwrap()
}

/// Helper to create an initialized temp directory (local mode, default)
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

/// Helper to create an initialized temp directory in remote mode (git:.)
pub fn init_temp_remote() -> TempDir {
    let temp = TempDir::new().unwrap();

    // Initialize git repo first
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");

    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--remote")
        .arg(".")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}
