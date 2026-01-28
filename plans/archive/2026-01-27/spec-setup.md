# Rust Spec Infrastructure Setup

## Overview

Set up Rust-based integration test infrastructure alongside the existing BATS specs. This provides a `cargo test --test specs` entry point with reusable helpers for creating test projects, building CLI commands, and asserting output.

## Project Structure

```
wok/
├── crates/cli/Cargo.toml         # Add similar_asserts dev-dependency
├── tests/
│   ├── specs.rs                  # NEW: Integration test entry point
│   └── specs/
│       ├── CLAUDE.md             # UPDATE: Add Rust spec conventions
│       ├── prelude.rs            # NEW: Test helpers module
│       ├── cli/                  # Existing BATS specs
│       ├── remote/               # Existing BATS specs
│       └── helpers/              # Existing BATS helpers
```

## Dependencies

Add to `crates/cli/Cargo.toml` under `[dev-dependencies]`:

```toml
similar_asserts = "1"
```

Existing dev-dependencies already provide:
- `assert_cmd = "2"` - Command execution and assertions
- `tempfile = "3"` - Temporary directories for test isolation
- `predicates = "3"` - Flexible output matching

## Implementation Phases

### Phase 1: Add similar_asserts Dependency

**Goal**: Add the missing dev-dependency.

**Files**: `crates/cli/Cargo.toml`

**Changes**:
```toml
[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
similar_asserts = "1"   # Add this line
yare = "3"
criterion = { version = "0.8", features = ["html_reports"] }
```

**Verification**: `cargo check` succeeds.

---

### Phase 2: Create tests/specs.rs Entry Point

**Goal**: Create the integration test entry point that cargo discovers.

**File**: `tests/specs.rs`

**Content**:
```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Rust-based integration specs for the wk CLI.
//!
//! Run with: cargo test --test specs
//!
//! These complement the BATS specs in tests/specs/ and are useful for:
//! - Tests requiring complex setup or teardown
//! - Tests that benefit from Rust's type system
//! - Performance-sensitive test suites

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

mod specs {
    #[path = "specs/prelude.rs"]
    pub mod prelude;
}

use specs::prelude::*;

#[test]
fn smoke_test_wk_version() {
    let output = Wk::new().arg("--version").output();
    output.success().stdout(predicates::str::contains("wk"));
}

#[test]
fn smoke_test_wk_help() {
    let output = Wk::new().arg("--help").output();
    output.success().stdout(predicates::str::contains("Usage:"));
}
```

**Verification**: `cargo test --test specs` compiles and runs.

---

### Phase 3: Create tests/specs/prelude.rs with Helpers

**Goal**: Provide reusable test infrastructure.

**File**: `tests/specs/prelude.rs`

**Content**:
```rust
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

use std::path::PathBuf;
use std::process::Output;

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
        Command::cargo_bin("wk")
            .expect("wk binary not found")
            .current_dir(&path)
            .env("HOME", &path)
            .args(["init", "--prefix", prefix])
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
        // Extract ID from "Created test-abc123" or similar
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
        let cmd = Command::cargo_bin("wk").expect("wk binary not found");
        Self { cmd }
    }

    /// Create a wk command that runs in the specified directory.
    pub fn in_dir(path: &PathBuf) -> Self {
        let mut cmd = Command::cargo_bin("wk").expect("wk binary not found");
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
    pub fn env(mut self, key: &str, val: &str) -> Self {
        self.cmd.env(key, val);
        self
    }

    /// Write to stdin.
    pub fn stdin(mut self, input: impl Into<Vec<u8>>) -> Self {
        self.cmd.write_stdin(input);
        self
    }

    /// Execute and get assert handle.
    pub fn output(self) -> assert_cmd::assert::Assert {
        self.cmd.assert()
    }

    /// Execute and get raw output (for extracting values).
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

pub use assert_output;
```

**Verification**: `cargo test --test specs` compiles and both smoke tests pass.

---

### Phase 4: Update tests/specs/CLAUDE.md

**Goal**: Document Rust spec conventions alongside BATS.

**File**: `tests/specs/CLAUDE.md`

Append new section:

```markdown
## Rust Specs

For tests that benefit from Rust's type system or complex setup:

```bash
# Run Rust specs
cargo test --test specs

# Run specific test
cargo test --test specs smoke_test_wk_version
```

### Structure

- `tests/specs.rs` - Entry point, imports prelude
- `tests/specs/prelude.rs` - Helpers (Project, Wk, assertions)

### Core Helpers

```rust
// Isolated project with temp directory
let project = Project::new("test");

// Run commands in project context
project.wk().args(["new", "task", "My task"]).output().success();

// Create issue and get ID
let id = project.create_issue("task", "My task");

// Standalone command (no project context)
Wk::new().arg("--version").output().success();
```

### When to Use Rust vs BATS

Use **Rust specs** when:
- Complex setup/teardown logic
- Type-safe fixtures or builders
- Testing internal behavior (with `wkrs` lib)
- Parameterized tests with `yare`

Use **BATS specs** when:
- Simple command-line invocation checks
- Testing shell integration (pipes, redirects)
- Quick iteration on CLI behavior
- Documenting user-facing examples
```

**Verification**: Documentation is clear and complete.

---

### Phase 5: Final Integration Test

**Goal**: Verify complete integration with a real test case.

Add a more complete test to `tests/specs.rs`:

```rust
#[test]
fn project_lifecycle() {
    let project = Project::new("demo");

    // Create an issue
    let id = project.create_issue("task", "Integration test");
    assert!(id.starts_with("demo-"));

    // List shows the issue
    project
        .wk()
        .arg("list")
        .output()
        .success()
        .stdout(predicates::str::contains(&id));

    // Show displays details
    project
        .wk()
        .args(["show", &id])
        .output()
        .success()
        .stdout(predicates::str::contains("Integration test"));

    // Complete the issue
    project
        .wk()
        .args(["done", &id])
        .output()
        .success();
}
```

**Verification**: `cargo test --test specs` passes all tests.

---

### Phase 6: Verify Integration

**Goal**: Ensure everything works together.

**Commands**:
```bash
cargo check                    # Compiles
cargo test --test specs        # All specs pass
cargo test --test specs -- --list  # List available tests
```

## Key Implementation Details

### Binary Resolution

`assert_cmd::Command::cargo_bin("wk")` automatically:
1. Builds the binary if needed (in test mode)
2. Locates the binary in target/debug or target/release
3. Returns a configured Command

### Test Isolation

Each `Project` instance:
- Creates a unique temp directory
- Sets `HOME` to the temp dir (isolates from user config)
- Initializes a fresh `.wok/` project
- Cleans up automatically when dropped (via TempDir)

### ID Extraction

The `create_issue` helper parses output like:
```
Created demo-a3f2
```
And extracts `demo-a3f2` as the issue ID.

### Predicates

Use `predicates::str::contains()` for flexible matching:
```rust
.stdout(predicates::str::contains("expected text"))
.stderr(predicates::str::is_empty())
```

## Verification Plan

1. **Phase 1**: `cargo check` - dependency resolves
2. **Phase 2**: `cargo test --test specs --no-run` - compiles
3. **Phase 3**: `cargo test --test specs smoke_test_wk_version` - runs
4. **Phase 4**: Manual review of documentation
5. **Phase 5**: `cargo test --test specs project_lifecycle` - passes
6. **Phase 6**: `cargo test --test specs` - all tests pass

Final verification:
```bash
cargo test --test specs -- --nocapture
```

Should show 3+ passing tests with no compilation warnings.
