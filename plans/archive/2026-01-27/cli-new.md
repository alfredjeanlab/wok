# Implementation Plan: Convert `new.bats` to Rust Specs

## Overview

Convert the BATS specs in `tests/specs/cli/unit/new.bats` to Rust integration tests in `tests/specs/cli/new.rs`. This establishes a new pattern for Rust-based specs alongside existing BATS specs, enabling type-safe testing with `assert_cmd`.

## Project Structure

```
tests/specs/cli/
├── unit/
│   └── new.bats          # Existing BATS specs (keep for reference)
├── new.rs                 # New: Rust spec file
└── mod.rs                 # New: Module declarations

crates/cli/
├── Cargo.toml            # Add tests/specs as test path
└── tests/
    └── common.rs         # Existing test helpers (reuse)
```

## Dependencies

The Rust specs will use existing dependencies from `crates/cli/Cargo.toml`:
- `assert_cmd` - Command execution and assertions
- `predicates` - Output matching predicates
- `tempfile` - Temp directory management
- `serde_json` - JSON validation for `-o json` tests

No new dependencies required.

## Implementation Phases

### Phase 1: Set Up Rust Spec Infrastructure

**Goal:** Establish the directory structure and cargo configuration.

**Files to create:**
- `tests/specs/cli/mod.rs` - Module declarations
- `tests/specs/cli/new.rs` - Main spec file

**Files to modify:**
- `crates/cli/Cargo.toml` - Add test path for specs

**Cargo.toml changes:**

```toml
[[test]]
name = "spec_new"
path = "../../tests/specs/cli/new.rs"
```

**mod.rs structure:**

```rust
// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

// Rust specs for CLI commands
// Run with: cargo test --test spec_new
```

**Verification:**
- `cargo test --test spec_new` runs without errors (empty test file)

---

### Phase 2: Issue Type Tests

**Goal:** Cover creation of issues with all types (task, bug, feature, chore, idea).

**Test cases from BATS:**
```
"new creates issues with correct type (default task, feature, bug, chore, idea)"
```

**Rust implementation:**

```rust
#[test]
fn new_default_creates_task() {
    let temp = init_temp();
    wk().arg("new")
        .arg("My task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[task]"));
}

#[test]
fn new_explicit_task() {
    let temp = init_temp();
    wk().arg("new")
        .arg("task")
        .arg("My explicit task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[task]"));
}

#[test]
fn new_feature() {
    let temp = init_temp();
    wk().arg("new")
        .arg("feature")
        .arg("My feature")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[feature]"));
}

#[test]
fn new_bug() {
    let temp = init_temp();
    wk().arg("new")
        .arg("bug")
        .arg("My bug")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[bug]"));
}

#[test]
fn new_chore() {
    let temp = init_temp();
    wk().arg("new")
        .arg("chore")
        .arg("My chore")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[chore]"));
}

#[test]
fn new_idea() {
    let temp = init_temp();
    wk().arg("new")
        .arg("idea")
        .arg("My idea")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[idea]"));
}

#[test]
fn new_starts_in_todo_status() {
    let temp = init_temp();
    wk().arg("new")
        .arg("Status task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("(todo)"));
}
```

**Verification:**
- `cargo test --test spec_new new_` shows all type tests passing

---

### Phase 3: Label Handling Tests

**Goal:** Cover `--label` flag variations including comma-separated labels.

**Test cases from BATS:**
```
"new with --label and --note adds metadata"
"new with comma-separated labels adds all labels"
```

**Rust implementation:**

```rust
#[test]
fn new_with_single_label() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Labeled task", &["--label", "project:auth"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"));
}

#[test]
fn new_with_multiple_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Multi-labeled",
        &["--label", "project:auth", "--label", "priority:high"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project:auth"))
        .stdout(predicate::str::contains("priority:high"));
}

#[test]
fn new_with_comma_separated_labels() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Comma labels", &["--label", "a,b,c"]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels: a, b, c"));
}

#[test]
fn new_comma_labels_trim_whitespace() {
    let temp = init_temp();
    let id = create_issue_with_opts(&temp, "task", "Whitespace labels", &["--label", "  x  ,  y  "]);

    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Labels: x, y"));
}
```

**Verification:**
- `cargo test --test spec_new label` shows all label tests passing

---

### Phase 4: Title and Validation Tests

**Goal:** Cover title handling including empty/whitespace rejection and invalid type rejection.

**Test cases from BATS:**
```
"new generates unique ID with prefix and validates inputs"
```

**Rust implementation:**

```rust
#[test]
fn new_requires_title() {
    let temp = init_temp();
    wk().arg("new")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_empty_title_fails() {
    let temp = init_temp();
    wk().arg("new")
        .arg("")
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn new_invalid_type_fails() {
    let temp = init_temp();
    wk().arg("new")
        .arg("epic")
        .arg("My epic")
        .current_dir(temp.path())
        .assert()
        .failure();
}
```

**Verification:**
- `cargo test --test spec_new title` and `cargo test --test spec_new invalid` pass

---

### Phase 5: ID Format and Prefix Tests

**Goal:** Verify ID generation with prefixes and `--prefix` flag.

**Test cases from BATS:**
```
"new generates unique ID with prefix and validates inputs"
"new --prefix creates issue with different prefix"
```

**Rust implementation:**

```rust
#[test]
fn new_id_uses_configured_prefix() {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("myproj")
        .current_dir(temp.path())
        .assert()
        .success();

    wk().arg("new")
        .arg("Test task")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("myproj-"));
}

#[test]
fn new_prefix_flag_overrides_config() {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("main")
        .current_dir(temp.path())
        .assert()
        .success();

    let output = wk()
        .arg("new")
        .arg("Task")
        .arg("--prefix")
        .arg("other")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(id.starts_with("other-"), "Expected other- prefix, got: {}", id);
}

#[test]
fn new_id_format_prefix_hex() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Test")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // ID format: prefix-xxxx where xxxx is hex
    let re = regex::Regex::new(r"^[a-z]+-[a-f0-9]+$").unwrap();
    assert!(re.is_match(&id), "ID format should be prefix-hex, got: {}", id);
}
```

**Verification:**
- `cargo test --test spec_new prefix` passes

---

### Phase 6: Output Format Tests

**Goal:** Cover `-o id`, `-o ids`, and `-o json` output formats.

**Test cases from BATS:**
```
"new -o id outputs only the issue ID"
"new -o ids alias works for backward compatibility"
"new -o json outputs valid JSON with expected fields"
"new -o id enables scripting workflows"
```

**Rust implementation:**

```rust
#[test]
fn new_output_id_only() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("ID output task")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-"), "Should output ID");
    assert!(!stdout.contains("Created"), "Should NOT contain verbose message");
    assert!(!stdout.contains("[task]"), "Should NOT contain type tag");
}

#[test]
fn new_output_ids_alias() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("IDs alias task")
        .arg("-o")
        .arg("ids")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-"), "Should output ID");
    assert!(!stdout.contains("Created"), "Should NOT contain verbose message");
}

#[test]
fn new_output_json_valid() {
    let temp = init_temp();
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("JSON task")
        .arg("--label")
        .arg("test:json")
        .arg("-o")
        .arg("json")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    assert!(json.get("id").is_some(), "Should have id field");
    assert_eq!(json.get("type").and_then(|v| v.as_str()), Some("task"));
    assert_eq!(json.get("title").and_then(|v| v.as_str()), Some("JSON task"));
    assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("todo"));
    assert!(json.get("labels").and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|l| l.as_str() == Some("test:json")))
        .unwrap_or(false));
}

#[test]
fn new_output_id_scripting_workflow() {
    let temp = init_temp();

    // Create issue and capture ID
    let output = wk()
        .arg("new")
        .arg("task")
        .arg("Scripted task")
        .arg("-o")
        .arg("id")
        .current_dir(temp.path())
        .output()
        .unwrap();

    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!id.is_empty(), "ID should not be empty");

    // Use ID in subsequent command
    wk().arg("label")
        .arg(&id)
        .arg("scripted")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify label was added
    wk().arg("show")
        .arg(&id)
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("scripted"));
}
```

**Verification:**
- `cargo test --test spec_new output` passes

## Key Implementation Details

### Test Helpers

Reuse helpers from `crates/cli/tests/common.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn wk() -> Command {
    Command::cargo_bin("wk").unwrap()
}

fn init_temp() -> TempDir {
    let temp = TempDir::new().unwrap();
    wk().arg("init")
        .arg("--prefix")
        .arg("test")
        .current_dir(temp.path())
        .assert()
        .success();
    temp
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
```

### Regex Dependency

Add `regex` to dev-dependencies if not present:

```toml
[dev-dependencies]
regex = "1"
```

### Test Organization

Group tests by functionality with descriptive names matching BATS test names for traceability:

```rust
// Type tests
mod type_tests { ... }

// Label tests
mod label_tests { ... }

// Validation tests
mod validation_tests { ... }

// Prefix tests
mod prefix_tests { ... }

// Output format tests
mod output_tests { ... }
```

## Verification Plan

### Run Rust Specs

```bash
# Build first
cargo build

# Run all new specs
cargo test --test spec_new

# Run with output
cargo test --test spec_new -- --nocapture
```

### Compare with BATS

```bash
# Run BATS specs for comparison
make spec ARGS='--file cli/unit/new.bats'
```

### Checklist

- [ ] `tests/specs/cli/new.rs` exists with all tests
- [ ] `crates/cli/Cargo.toml` has test path configured
- [ ] `cargo test --test spec_new` passes (all tests green)
- [ ] All BATS test cases have corresponding Rust tests:
  - [ ] Issue type creation (task, bug, feature, chore, idea)
  - [ ] Default type is task
  - [ ] Status starts as todo
  - [ ] Single label with `--label`
  - [ ] Multiple labels with `--label`
  - [ ] Comma-separated labels
  - [ ] Whitespace trimming in labels
  - [ ] Empty title rejection
  - [ ] Missing title rejection
  - [ ] Invalid type rejection
  - [ ] ID format (prefix-hex)
  - [ ] `--prefix` flag override
  - [ ] `-o id` output format
  - [ ] `-o ids` alias
  - [ ] `-o json` output format
  - [ ] Scripting workflow with captured ID
- [ ] `cargo clippy` passes
- [ ] `cargo fmt --check` passes
