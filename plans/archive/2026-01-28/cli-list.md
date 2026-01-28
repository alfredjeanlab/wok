# Implementation Plan: Convert CLI List Tests to Rust

## Overview

Convert `tests/specs/cli/unit/list.bats` to Rust integration tests in `tests/specs/cli/list.rs`. This provides type-safe testing, better organization, and consistent patterns with existing Rust specs (`new.rs`, `init.rs`).

## Project Structure

```
tests/specs/cli/
├── mod.rs          # Add: mod list;
├── common.rs       # Shared helpers (wk(), init_temp())
├── init.rs         # Existing
├── new.rs          # Existing - reference for patterns
└── list.rs         # NEW - converted list tests
```

## Dependencies

Already available in workspace:
- `assert_cmd` - command assertions
- `predicates` - output predicates
- `tempfile` - temporary directories
- `serde_json` - JSON parsing for `--output json` tests
- `regex` - ID format validation (already used in new.rs)

## Implementation Phases

### Phase 1: Setup and Status Filtering Tests

**Objective:** Create list.rs with basic structure and status filtering tests.

**Files to create/modify:**
- `tests/specs/cli/list.rs` (create)
- `tests/specs/cli/mod.rs` (add `mod list;`)
- `tests/specs/cli/common.rs` (add `create_issue` helper)

**Tests to implement:**
1. `list_empty_database` - empty output on fresh init
2. `list_shows_created_issues` - basic issue display
3. `list_default_shows_todo_and_in_progress` - excludes done
4. `list_status_filter_todo` - `--status todo`
5. `list_status_filter_in_progress` - `--status in_progress`
6. `list_status_filter_done` - `--status done`

**Add helper to common.rs:**

```rust
/// Create an issue and return its ID
pub fn create_issue(temp: &TempDir, type_: &str, title: &str) -> String {
    create_issue_with_opts(temp, type_, title, &[])
}

pub fn create_issue_with_opts(temp: &TempDir, type_: &str, title: &str, opts: &[&str]) -> String {
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

**Verification:**
```bash
cargo test --test specs list_status
```

---

### Phase 2: Type, Label, and Blocked Filter Tests

**Objective:** Add filtering tests for type, label, and blocked status.

**Tests to implement:**
1. `list_type_filter_feature` - `--type feature`
2. `list_type_filter_bug` - `--type bug`
3. `list_type_filter_chore` - `--type chore`
4. `list_type_filter_idea` - `--type idea`
5. `list_type_short_flag` - `-t task`
6. `list_label_filter` - `--label "project:auth"`
7. `list_blocked_filter` - `--blocked`
8. `list_default_shows_blocked` - both blocker and blocked visible
9. `list_combined_filters` - `--type feature --label "team:alpha"`

**Verification:**
```bash
cargo test --test specs list_type
cargo test --test specs list_label
cargo test --test specs list_blocked
```

---

### Phase 3: JSON Output Tests

**Objective:** Test `--output json` format and structure.

**Tests to implement:**
1. `list_output_json_valid` - valid JSON structure
2. `list_output_json_fields` - id, issue_type, status, title, labels fields
3. `list_output_json_labels` - labels array populated correctly
4. `list_output_json_short_flag` - `-o json`
5. `list_output_json_type_filter` - respects `--type` filter
6. `list_output_json_no_blocked_count` - no blocked_count field

**Example test pattern:**

```rust
#[test]
fn list_output_json_valid() {
    let temp = init_temp();
    let id = create_issue(&temp, "task", "JSONList Task");
    wk().arg("label").arg(&id).arg("priority:high")
        .current_dir(temp.path()).assert().success();

    let output = wk()
        .arg("list").arg("--output").arg("json")
        .current_dir(temp.path())
        .output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    assert!(json.get("issues").and_then(|v| v.as_array()).is_some());
    let issues = json["issues"].as_array().unwrap();
    let issue = issues.iter().find(|i| i["title"] == "JSONList Task").unwrap();
    assert!(issue.get("id").is_some());
    assert_eq!(issue["labels"].as_array().unwrap()[0], "priority:high");
}
```

**Verification:**
```bash
cargo test --test specs list_output_json
```

---

### Phase 4: Priority Sorting Tests

**Objective:** Verify sort order by priority and creation time.

**Tests to implement:**
1. `list_sorts_by_priority_asc` - P1 before P3
2. `list_same_priority_newer_first` - created_at DESC within priority
3. `list_missing_priority_as_2` - default priority behavior
4. `list_prefers_priority_over_p` - `priority:4` > `p:0`

**Example pattern with timing:**

```rust
#[test]
fn list_same_priority_newer_first() {
    let temp = init_temp();
    create_issue(&temp, "task", "SortList Older");
    std::thread::sleep(std::time::Duration::from_millis(100));
    create_issue(&temp, "task", "SortList Newer");

    let output = wk()
        .arg("list")
        .current_dir(temp.path())
        .output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let newer_pos = stdout.find("SortList Newer").unwrap();
    let older_pos = stdout.find("SortList Older").unwrap();
    assert!(newer_pos < older_pos, "Newer should appear before older");
}
```

**Verification:**
```bash
cargo test --test specs list_sort
```

---

### Phase 5: Filter Expressions and Duration Parsing

**Objective:** Test `--filter` with age, updated, closed fields and duration parsing.

**Tests to implement:**
1. `list_filter_age_less_than` - `--filter "age < 400ms"`
2. `list_filter_age_gte` - `--filter "age >= 400ms"`
3. `list_filter_short_flag` - `-q "age < 1h"`
4. `list_filter_invalid_field` - error on unknown field
5. `list_filter_invalid_operator` - error on `<<`
6. `list_filter_invalid_duration` - error on `3x`
7. `list_filter_multiple` - multiple `--filter` args
8. `list_filter_combined_with_flags` - `--filter` + `--type` + `--label`
9. `list_filter_closed` - `--filter "closed < 1d"` shows done/closed
10. `list_filter_completed` - only Status::Done
11. `list_filter_skipped` - only Status::Closed
12. `list_filter_bare_closed` - bare `--filter "closed"` syntax
13. `list_filter_bare_completed` - bare `--filter "completed"` syntax
14. `list_filter_bare_age_fails` - bare `--filter "age"` requires operator
15. `list_filter_accepts_now` - `--filter "closed < now"`

**Word operators tests:**
1. `list_filter_word_lt` - `age lt 400ms`
2. `list_filter_word_gte` - `age gte 400ms`
3. `list_filter_word_gt` - `age gt 300ms`
4. `list_filter_word_lte` - `age lte 1d`
5. `list_filter_word_case_insensitive` - `LT`, `GT`

**Verification:**
```bash
cargo test --test specs list_filter
```

---

### Phase 6: Limit and IDs Output Tests

**Objective:** Test `--limit`, default 100 results, and `--output ids` format.

**Tests to implement:**

**Limit tests:**
1. `list_defaults_to_100_results` - max 100 without explicit limit
2. `list_limit_truncates` - `--limit 2` returns 2
3. `list_limit_short_flag` - `-n 1`
4. `list_limit_0_unlimited` - `--limit 0` shows all
5. `list_explicit_limit_overrides` - `--limit 20`
6. `list_json_metadata_filters_applied` - `filters_applied` field when filtered
7. `list_json_metadata_limit` - `limit` field in JSON

**IDs output tests:**
1. `list_output_ids_space_separated` - single line, space-separated
2. `list_output_ids_no_metadata` - no type, status, title
3. `list_output_ids_with_filters` - respects `--type` filter
4. `list_output_ids_respects_limit` - `--limit 10`
5. `list_output_ids_short_flag` - `-o ids`
6. `list_output_ids_clean_format` - alphanumeric with hyphens only

**Example pattern:**

```rust
#[test]
fn list_output_ids_space_separated() {
    let temp = init_temp();
    let id1 = create_issue(&temp, "task", "IDFormat Issue 1");
    let id2 = create_issue(&temp, "task", "IDFormat Issue 2");

    let output = wk()
        .arg("list").arg("--output").arg("ids")
        .current_dir(temp.path())
        .output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&id1));
    assert!(stdout.contains(&id2));
    assert!(!stdout.contains("task"));
    assert!(!stdout.contains("todo"));
    assert!(!stdout.contains("IDFormat"));

    // Single line
    let line_count = stdout.lines().count();
    assert_eq!(line_count, 1);
}
```

**Verification:**
```bash
cargo test --test specs list_limit
cargo test --test specs list_output_ids
```

## Key Implementation Details

### Test Isolation

Each test uses `init_temp()` for isolated project directory. Use unique prefixes in issue titles (e.g., `StatusFilter`, `TypeFilter`) to avoid collisions when asserting output.

### Timing-Sensitive Tests

For age/timing tests, use generous margins:
```rust
std::thread::sleep(std::time::Duration::from_millis(500));
```
And filter with corresponding margin: `--filter "age < 400ms"` vs `--filter "age >= 400ms"`

### JSON Assertions

Use `serde_json::Value` for flexible JSON structure checks:
```rust
let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
assert!(json["issues"].is_array());
```

### Output Assertions

Use predicates for contains/not-contains:
```rust
.assert()
.success()
.stdout(predicate::str::contains("Expected"))
.stdout(predicate::str::contains("Unexpected").not());
```

### Helper Functions

Add to `common.rs`:
- `create_issue(temp, type_, title)` - create issue, return ID
- `create_issue_with_opts(temp, type_, title, opts)` - with additional args

## Verification Plan

### Per-Phase Verification

```bash
# After each phase
cargo test --test specs list
cargo fmt --check
cargo clippy -- -D warnings
```

### Full Test Suite

```bash
# Run Rust specs
cargo test --test specs

# Run remaining bats specs (can be removed after conversion)
make spec ARGS='--file cli/unit/list.bats'

# Full validation
make check
```

### Checklist

- [ ] `mod list;` added to `tests/specs/cli/mod.rs`
- [ ] `create_issue` helper added to `common.rs`
- [ ] Phase 1: Status filtering tests pass
- [ ] Phase 2: Type/label/blocked filter tests pass
- [ ] Phase 3: JSON output tests pass
- [ ] Phase 4: Priority sorting tests pass
- [ ] Phase 5: Filter expressions tests pass
- [ ] Phase 6: Limit and IDs output tests pass
- [ ] `cargo test --test specs list` passes (all 40+ tests)
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Original bats tests remain passing (verify parity)
