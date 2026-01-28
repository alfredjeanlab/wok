# CLI Init: Convert BATS to Rust Specs

## Overview

Convert `tests/specs/cli/unit/init.bats` to Rust integration tests in `tests/specs/cli/init.rs`, providing one-to-one test coverage. The existing `crates/cli/tests/init.rs` covers some tests but has gaps. This plan creates a new Rust spec location and ensures complete coverage of all BATS test cases.

## Project Structure

```
tests/
├── specs/
│   ├── cli/
│   │   ├── mod.rs            # Module declarations
│   │   ├── common.rs         # Shared test helpers
│   │   └── init.rs           # Init command specs (NEW)
│   └── lib.rs                # Test crate entry point
└── Cargo.toml                # Integration test manifest
```

Key files:
- `tests/specs/cli/unit/init.bats` - Source BATS tests (9 test functions)
- `crates/cli/tests/init.rs` - Existing Rust tests (12 tests, partial coverage)
- `crates/cli/tests/common.rs` - Helper patterns to reuse

## Dependencies

Dependencies for `tests/specs/Cargo.toml`:
```toml
[package]
name = "specs"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.10"
```

## Implementation Phases

### Phase 1: Create Test Infrastructure

**Goal**: Set up the Rust test crate in `tests/specs/`.

1. Create `tests/specs/Cargo.toml` with dev-dependencies
2. Create `tests/specs/cli/mod.rs` module file
3. Create `tests/specs/cli/common.rs` with helper functions:
   - `wk()` - Command builder
   - `init_temp()` - Create initialized temp directory
   - `init_temp_remote()` - Create with git + remote mode

**Verification**: `cargo test -p specs --no-run` compiles without errors.

### Phase 2: Basic Init Tests

**Goal**: Cover BATS tests 1-2 (directory creation, --path option).

Tests to implement:
```rust
// From: "init creates .wok directory and fails if already initialized"
#[test] fn creates_wok_directory()
#[test] fn fails_if_already_initialized()
#[test] fn succeeds_if_wok_exists_without_config()

// From: "init with --path creates at specified location"
#[test] fn path_option_creates_at_specified_location()
#[test] fn path_option_creates_parent_directories()
#[test] fn path_option_fails_if_already_initialized()
```

**Verification**: `cargo test -p specs -- creates_wok` passes.

### Phase 3: Prefix Handling Tests

**Goal**: Cover BATS test 3 (prefix validation).

Tests to implement:
```rust
// From: "init prefix handling and validation"
#[test] fn uses_directory_name_as_default_prefix()
#[test] fn lowercases_and_filters_alphanumeric()
#[test] fn explicit_prefix_overrides_directory()
#[test] fn fails_with_invalid_directory_name_for_prefix()
#[test] fn valid_prefixes_accepted()  // abc, ab, abc123, mylongprefix
#[test] fn invalid_prefixes_rejected()  // ABC, 123, my-prefix, my_prefix, a
```

**Verification**: `cargo test -p specs -- prefix` passes.

### Phase 4: Database/Config Tests

**Goal**: Cover BATS test 4 (database tables, config format, issue creation).

Tests to implement:
```rust
// From: "init creates valid database, config, and allows issue creation"
#[test] fn creates_valid_sqlite_database()
#[test] fn database_has_required_tables()  // issues, deps, labels, notes, events
#[test] fn empty_database_shows_no_issues()
#[test] fn config_is_valid_toml()
#[test] fn allows_immediate_issue_creation_with_prefix()
```

**Verification**: `cargo test -p specs -- database` and `cargo test -p specs -- config` pass.

### Phase 5: Workspace Mode Tests

**Goal**: Cover BATS test 5 (--workspace option).

Tests to implement:
```rust
// From: "init with --workspace"
#[test] fn workspace_creates_config_without_database()
#[test] fn workspace_with_prefix()
#[test] fn workspace_validates_prefix()
#[test] fn workspace_accepts_relative_path()
#[test] fn workspace_with_path_option()
#[test] fn workspace_fails_if_not_exist()
```

**Verification**: `cargo test -p specs -- workspace` passes.

### Phase 6: Gitignore and Remote Mode Tests

**Goal**: Cover BATS tests 6-9 (.gitignore entries, --remote mode, git worktree).

Tests to implement:
```rust
// From: "init creates .gitignore with correct entries"
#[test] fn gitignore_contains_current_and_database()
#[test] fn default_mode_ignores_config_toml()
#[test] fn local_flag_ignores_config_toml()
#[test] fn workspace_mode_ignores_config_toml()

// From: "init with --remote excludes config.toml from .gitignore"
#[test] fn remote_mode_does_not_ignore_config_toml()

// From: "init defaults to local mode without remote"
#[test] fn defaults_to_local_mode_no_remote_config()

// From: "init with git remote creates worktree and supports sync"
#[test] fn remote_creates_git_worktree()
#[test] fn remote_creates_orphan_branch()
#[test] fn remote_worktree_protects_branch()
#[test] fn remote_sync_works_with_worktree()
```

**Verification**: `cargo test -p specs -- gitignore` and `cargo test -p specs -- remote` pass.

## Key Implementation Details

### Helper Functions

```rust
// tests/specs/cli/common.rs

use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
pub use predicates::prelude::*;
pub use tempfile::TempDir;

pub fn wk() -> Command {
    cargo_bin_cmd!("wk")
}

pub fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix").arg("test")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}

pub fn init_temp_remote() -> TempDir {
    let temp = TempDir::new().unwrap();
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp.path())
        .status()
        .expect("git init failed");
    wk().arg("init")
        .arg("--prefix").arg("test")
        .arg("--remote").arg(".")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
}
```

### Test Patterns

**File existence checks**:
```rust
assert!(temp.path().join(".wok").exists());
assert!(temp.path().join(".wok/config.toml").exists());
assert!(!temp.path().join(".wok/issues.db").exists()); // for workspace mode
```

**Config content verification**:
```rust
let config = std::fs::read_to_string(temp.path().join(".wok/config.toml")).unwrap();
assert!(config.contains("prefix = \"myapp\""));
assert!(!config.contains("[remote]"));
```

**Git operations** (for remote tests):
```rust
// Check orphan branch exists
let output = std::process::Command::new("git")
    .args(["rev-parse", "--verify", "refs/heads/wok/oplog"])
    .current_dir(temp.path())
    .output()
    .unwrap();
assert!(output.status.success());
```

### BATS to Rust Test Mapping

| BATS Test | Rust Test Functions |
|-----------|---------------------|
| init creates .wok directory... | `creates_wok_directory`, `fails_if_already_initialized`, `succeeds_if_wok_exists_without_config` |
| init with --path... | `path_option_creates_at_specified_location`, `path_option_creates_parent_directories`, `path_option_fails_if_already_initialized` |
| init prefix handling... | 6 tests for prefix validation |
| init creates valid database... | 5 tests for database/config |
| init with --workspace | 6 tests for workspace mode |
| init creates .gitignore... | 4 tests for gitignore entries |
| init with --remote... | 1 test for remote gitignore |
| init defaults to local... | 1 test for default mode |
| init with git remote... | 4 tests for git worktree |

**Total**: 9 BATS tests → ~30 Rust tests (one-to-one assertion coverage)

## Verification Plan

1. **Phase-by-phase validation**:
   - After each phase, run `cargo test -p specs` to ensure tests pass
   - Check test names match BATS test descriptions

2. **Coverage comparison**:
   ```bash
   # List BATS test names
   grep '@test' tests/specs/cli/unit/init.bats

   # List Rust test names
   cargo test -p specs -- --list 2>&1 | grep 'test '
   ```

3. **Final validation**:
   ```bash
   # Run all specs
   make spec-cli ARGS='--file cli/unit/init.bats'  # BATS
   cargo test -p specs                              # Rust

   # Both should have equivalent coverage
   ```

4. **Integration check**:
   - Add `specs` to workspace in root `Cargo.toml`
   - Ensure `make validate` still passes
   - Run `cargo test --workspace` includes new specs
