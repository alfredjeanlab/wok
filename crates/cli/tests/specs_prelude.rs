// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Test prelude with helpers for integration specs.
//!
//! # Core Types
//! - [`Project`] - Isolated test project with temp directory
//! - [`Wk`] - CLI command builder
//!
//! # Re-exports
//! - `predicates` - For output matching
//! - `similar_asserts` - For diff-based assertions

// These are intentionally exported for use in tests, even if not used in all tests
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::Output;

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use tempfile::TempDir;

// Re-export for use in tests
pub use predicates;
pub use similar_asserts::assert_eq;

/// An isolated test project with its own temp directory.
///
/// # Example
/// ```ignore
/// let project = Project::new("test");
/// project.wk().args(["new", "task", "My task"]).success();
/// ```
pub struct Project {
    _temp: TempDir,
    path: PathBuf,
}

impl Project {
    /// Create a new isolated project with the given prefix.
    pub fn new(prefix: &str) -> Self {
        let temp = TempDir::new().expect("failed to create temp dir");
        let path = temp.path().to_path_buf();

        // Initialize the project
        cargo_bin_cmd!("wok")
            .current_dir(&path)
            .env("HOME", &path)
            .args(["init", "--prefix", prefix, "--private"])
            .assert()
            .success();

        Self { _temp: temp, path }
    }

    /// Get a command builder for wk in this project's directory.
    pub fn wk(&self) -> Wk {
        Wk::in_dir(&self.path)
    }

    /// Get the project's working directory path.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Create an issue and return its ID.
    pub fn create_issue(&self, kind: &str, title: &str) -> String {
        let output = self
            .wk()
            .args(["new", kind, title])
            .output()
            .get_output()
            .clone();

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Extract ID from "Created demo-a3f2" or similar
        stdout
            .lines()
            .find_map(|line| {
                line.split_whitespace()
                    .find(|word| word.contains('-') && word.chars().any(|c| c.is_ascii_hexdigit()))
            })
            .map(|s| s.trim_end_matches(':').to_string())
            .expect("failed to extract issue ID from output")
    }
}

/// CLI command builder for the wk binary.
///
/// # Example
/// ```ignore
/// Wk::new().arg("--version").output().success();
/// ```
pub struct Wk {
    cmd: Command,
}

impl Wk {
    /// Create a new wk command (uses current directory).
    pub fn new() -> Self {
        let cmd = cargo_bin_cmd!("wok");
        Self { cmd }
    }

    /// Create a wk command that runs in the specified directory.
    pub fn in_dir(path: &PathBuf) -> Self {
        let mut cmd = cargo_bin_cmd!("wok");
        cmd.current_dir(path);
        cmd.env("HOME", path);
        Self { cmd }
    }

    /// Add a single argument.
    pub fn arg(mut self, arg: &str) -> Self {
        self.cmd.arg(arg);
        self
    }

    /// Add multiple arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.cmd.args(args);
        self
    }

    /// Set an environment variable.
    #[allow(dead_code)]
    pub fn env(mut self, key: &str, val: &str) -> Self {
        self.cmd.env(key, val);
        self
    }

    /// Write to stdin.
    #[allow(dead_code)]
    pub fn stdin(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.cmd.write_stdin(input);
        self
    }

    /// Execute and get assert handle.
    pub fn output(mut self) -> assert_cmd::assert::Assert {
        self.cmd.assert()
    }

    /// Execute and get raw output (for extracting values).
    #[allow(dead_code)]
    pub fn run(mut self) -> Output {
        self.cmd.output().expect("failed to execute wk")
    }
}

impl Default for Wk {
    fn default() -> Self {
        Self::new()
    }
}

/// Assert that two strings are equal with diff output on failure.
#[macro_export]
macro_rules! assert_output {
    ($actual:expr, $expected:expr) => {
        similar_asserts::assert_eq!($actual.trim(), $expected.trim());
    };
}
