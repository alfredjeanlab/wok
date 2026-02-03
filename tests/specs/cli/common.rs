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
    Command::cargo_bin("wok").unwrap()
}

/// Helper to create an initialized temp directory in private mode.
/// Uses --private for test isolation (each test gets its own database).
pub fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

/// Helper to create an initialized temp directory in private mode
pub fn init_temp_private() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .arg("--private")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}
