# Plan: Convert show.bats to Rust Specs

## Overview

Convert `tests/specs/cli/unit/show.bats` to Rust integration tests in `tests/specs/cli/show.rs`. The conversion covers display of issue details (status, type, title, labels), dependency relationships (blocks/blocked-by, tracks/tracked-by), notes display, and JSON output format.

## Project Structure

```
tests/specs/cli/
├── mod.rs          # Add: mod show;
├── common.rs       # Existing shared helpers
├── init.rs         # Existing init specs
├── new.rs          # Existing new specs (reference pattern)
└── show.rs         # NEW: show command specs
```

## Dependencies

Already available in the project:
- `assert_cmd` - Command execution and assertions
- `predicates` - Output matching
- `tempfile` - Isolated test directories
- `serde_json` - JSON parsing for output validation

## Implementation Phases

### Phase 1: Create show.rs Module Structure

1. Create `tests/specs/cli/show.rs` with standard header and imports
2. Add `mod show;` to `tests/specs/cli/mod.rs`
3. Add helper function for creating issues with options (similar to new.rs pattern)

```rust
// tests/specs/cli/show.rs
use crate::cli::common::*;

fn create_issue(temp: &TempDir, kind: &str, title: &str) -> String {
    let output = wk()
        .args(["new", kind, title, "-o", "id"])
        .current_dir(temp.path())
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_issue_with_opts(temp: &TempDir, kind: &str, title: &str, opts: &[&str]) -> String {
    let mut cmd = wk();
    cmd.arg("new").arg(kind).arg(title);
    for opt in opts {
        cmd.arg(opt);
    }
    cmd.arg("-o").arg("id");
    let output = cmd.current_dir(temp.path()).output().unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
```

**Verification:** `cargo test --test specs show` compiles without errors

### Phase 2: Basic Display Tests

Convert first test group: "show displays issue details and labels"

| BATS Test | Rust Test |
|-----------|-----------|
| Displays issue details | `show_displays_issue_details` |
| Shows status, type, title | (included in above) |
| Displays single label | `show_displays_single_label` |
| Displays multiple labels | `show_displays_multiple_labels` |

```rust
#[test]
fn show_displays_issue_details() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowBasic Test task");

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("[task] {}", id)))
        .stdout(predicate::str::contains("Title: ShowBasic Test task"))
        .stdout(predicate::str::contains("Status: todo"))
        .stdout(predicate::str::contains("Created:"))
        .stdout(predicate::str::contains("Updated:"));
}
```

**Verification:** `cargo test --test specs show_displays` passes

### Phase 3: Notes Display Tests

Convert: "show displays notes grouped by status"

| BATS Test | Rust Test |
|-----------|-----------|
| Displays notes | `show_displays_notes` |
| Groups notes by status | `show_groups_notes_by_status` |

```rust
#[test]
fn show_groups_notes_by_status() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "ShowNotes Grouped task");

    wk().args(["note", &id, "Todo note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["start", &id])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["note", &id, "In progress note"])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &id])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Todo note"))
        .stdout(predicate::str::contains("In progress note"));
}
```

**Verification:** `cargo test --test specs show_notes` passes

### Phase 4: Dependency Relationship Tests

Convert: "show displays dependency relationships"

| BATS Test | Rust Test |
|-----------|-----------|
| Displays blockers | `show_displays_blocked_by` |
| Displays blocking relationships | `show_displays_blocks` |
| Displays parent relationship | `show_displays_tracked_by` |
| Displays children | `show_displays_tracks` |
| Displays log | `show_displays_log` |

```rust
#[test]
fn show_displays_blocked_by() {
    let temp = init_temp();
    let a = create_issue(&temp, "task", "ShowDep Blocker");
    let b = create_issue(&temp, "task", "ShowDep Blocked");

    wk().args(["dep", &a, "blocks", &b])
        .current_dir(temp.path())
        .assert()
        .success();

    wk().args(["show", &b])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by"))
        .stdout(predicate::str::contains(&a));
}
```

**Verification:** `cargo test --test specs show_displays_block` passes

### Phase 5: Error Handling and Multiple Issues

Convert remaining tests:

| BATS Test | Rust Test |
|-----------|-----------|
| Nonexistent issue fails | `show_nonexistent_fails` |
| Requires issue ID | `show_requires_issue_id` |
| Multiple issues separated by --- | `show_multiple_issues_separator` |
| Fails if any ID invalid | `show_fails_if_any_id_invalid` |

```rust
#[test]
fn show_nonexistent_fails() {
    let temp = init_temp();
    wk().args(["show", "test-nonexistent"])
        .current_dir(temp.path())
        .assert()
        .failure();
}

#[test]
fn show_multiple_issues_separator() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "First issue");
    let id2 = create_issue(&temp, "task", "Second issue");

    wk().args(["show", &id1, &id2])
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("First issue"))
        .stdout(predicate::str::contains("---"))
        .stdout(predicate::str::contains("Second issue"));
}
```

**Verification:** `cargo test --test specs show_` passes

### Phase 6: JSON Output Format Tests

Convert JSON-related tests:

| BATS Test | Rust Test |
|-----------|-----------|
| Single issue JSON is compact (JSONL) | `show_json_single_issue_compact` |
| Multiple issues JSON outputs JSONL | `show_json_multiple_issues_jsonl` |

```rust
#[test]
fn show_json_single_issue_compact() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "Test issue");

    let output = wk()
        .args(["show", &id, "-o", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Single line (compact JSONL format)
    assert_eq!(stdout.trim().lines().count(), 1);

    // Valid JSON
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("Output should be valid JSON");
    assert!(json.get("id").is_some());
}

#[test]
fn show_json_multiple_issues_jsonl() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "First issue");
    let id2 = create_issue(&temp, "task", "Second issue");

    let output = wk()
        .args(["show", &id1, &id2, "-o", "json"])
        .current_dir(temp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    // Two lines (one per issue)
    assert_eq!(lines.len(), 2);

    // Each line is valid JSON
    for line in lines {
        let _: serde_json::Value = serde_json::from_str(line)
            .expect("Each line should be valid JSON");
    }
}
```

**Verification:** `cargo test --test specs show_json` passes

## Key Implementation Details

### Test Organization

Group tests by functionality using comment headers (following new.rs pattern):
```rust
// =============================================================================
// Basic Display Tests
// =============================================================================

// =============================================================================
// Notes Tests
// =============================================================================

// =============================================================================
// Dependency Tests
// =============================================================================

// =============================================================================
// Error Handling Tests
// =============================================================================

// =============================================================================
// JSON Output Tests
// =============================================================================
```

### Helper Reuse

Use existing helpers from `common.rs`:
- `wk()` - Command builder
- `init_temp()` - Initialize temp directory with project
- `predicate::str::contains()` - Output assertions

### Test Naming Convention

Follow pattern: `show_<what_it_tests>`
- `show_displays_issue_details`
- `show_displays_single_label`
- `show_nonexistent_fails`
- `show_json_single_issue_compact`

## Verification Plan

1. **Compile check:** `cargo check --test specs`
2. **Run all show tests:** `cargo test --test specs show`
3. **Run full spec suite:** `make spec-cli` to ensure no regressions
4. **Compare coverage:** Verify all BATS test cases are covered

### Test Mapping Checklist

- [ ] `show displays issue details and labels` → 3 Rust tests
- [ ] `show displays notes grouped by status` → 2 Rust tests
- [ ] `show displays dependency relationships` → 5 Rust tests
- [ ] `show error handling` → 2 Rust tests
- [ ] `show: multiple issues in text mode` → 1 Rust test
- [ ] `show: multiple issues in json mode` → 1 Rust test
- [ ] `show: single issue json format` → 1 Rust test
- [ ] `show: fails if any ID is invalid` → 1 Rust test

**Total: 16 Rust tests covering all BATS functionality**
