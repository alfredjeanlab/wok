# Implementation Plan: CLI New Command Review

## Overview

Review and enhance test coverage for the `wk new` command to ensure:
1. All `new.bats` integration tests have corresponding unit tests
2. Type validation tests are comprehensive and parameterized
3. Use `yare` for parameterized tests where appropriate

## Project Structure

```
crates/cli/src/
├── cli_tests/
│   └── new_tests.rs              # CLI parsing tests (update with yare)
├── commands/
│   ├── new.rs                    # Implementation
│   └── new_tests.rs              # Command unit tests (add missing)
├── models/
│   └── issue_tests.rs            # IssueType tests (already uses yare)
└── validate_tests.rs             # Validation tests

tests/specs/cli/unit/
└── new.bats                      # Integration tests (11 tests, all implemented)
```

## Dependencies

Already available:
- `yare = "3"` in `[dev-dependencies]` of both `crates/cli` and `crates/core`

## Implementation Phases

### Phase 1: Audit Test Coverage Gap Analysis

**Objective:** Map `new.bats` tests to unit tests and identify gaps.

| new.bats Test | Unit Test Coverage | Gap |
|---------------|-------------------|-----|
| Creates issues with correct type | `cli_tests/new_tests.rs`: `test_new_with_type_and_title` | Missing: all type variants |
| --label and --note adds metadata | `test_new_with_labels`, `test_new_with_note` | Covered |
| Unique ID with prefix, validates inputs | `test_new_empty_title_rejected` | Missing: invalid type test at impl level |
| --priority adds priority label | `test_new_priority_*` | Covered (bounds, non-numeric) |
| --description adds note | `test_new_with_description` | Covered |
| Comma-separated labels | None | **Missing** |
| -o id outputs only ID | None | **Missing** (output format unit test) |
| -o ids backward compatibility | None | **Missing** |
| -o json outputs valid JSON | None | **Missing** |
| -o id enables scripting | Integration only | N/A (integration test appropriate) |
| --prefix creates different prefix | None in new_tests.rs | **Missing** |

**Files to review:**
- `crates/cli/src/cli_tests/new_tests.rs`
- `crates/cli/src/commands/new_tests.rs` (if exists)

**Verification:**
- Complete audit document with gaps identified

---

### Phase 2: Add Parameterized IssueType Tests with Yare

**Objective:** Ensure all IssueType variants are tested at CLI parsing level using yare.

**Files to modify:**
- `crates/cli/src/cli_tests/new_tests.rs`

**Add parameterized test for valid types:**

```rust
use yare::parameterized;

#[parameterized(
    task_lower = { "task", "Task title", "task" },
    task_upper = { "TASK", "Task title", "task" },
    task_mixed = { "Task", "Task title", "task" },
    feature_lower = { "feature", "Feature title", "feature" },
    feature_upper = { "FEATURE", "Feature title", "feature" },
    bug_lower = { "bug", "Bug title", "bug" },
    bug_upper = { "BUG", "Bug title", "bug" },
    chore_lower = { "chore", "Chore title", "chore" },
    chore_upper = { "CHORE", "Chore title", "chore" },
    idea_lower = { "idea", "Idea title", "idea" },
    idea_upper = { "IDEA", "Idea title", "idea" },
)]
fn test_new_type_parsing(type_str: &str, title: &str, expected: &str) {
    let cli = parse(&["wk", "new", type_str, title]).unwrap();
    match cli.command {
        Command::New { type_or_title, title: Some(_), .. } => {
            assert_eq!(type_or_title, type_str);
            // Type parsing validation happens in run_impl
        }
        _ => panic!("Expected New command"),
    }
}
```

**Add parameterized test for invalid types:**

```rust
#[parameterized(
    epic = { "epic" },
    story = { "story" },
    invalid = { "invalid" },
    empty = { "" },
)]
fn test_new_invalid_type_fails_at_impl(type_str: &str) {
    // Type validation happens in run_impl, not at clap level
    // This tests that the type is passed through
    let cli = parse(&["wk", "new", type_str, "Title"]);
    // For empty type, clap rejects due to non_empty_string validator
    if type_str.is_empty() {
        assert!(cli.is_err());
    } else {
        assert!(cli.is_ok()); // Parsing succeeds, validation in run_impl
    }
}
```

**Verification:**
```bash
cargo test -p wk new_type
```

---

### Phase 3: Add Missing Output Format Tests

**Objective:** Add unit tests for output format handling.

**Files to modify:**
- `crates/cli/src/cli_tests/new_tests.rs`

**Add tests:**

```rust
#[parameterized(
    text_default = { &["wk", "new", "task", "Test"], OutputFormat::Text },
    id_long = { &["wk", "new", "task", "Test", "-o", "id"], OutputFormat::Id },
    id_short = { &["wk", "new", "task", "Test", "--output", "id"], OutputFormat::Id },
    ids_alias = { &["wk", "new", "task", "Test", "-o", "ids"], OutputFormat::Id },
    json_format = { &["wk", "new", "task", "Test", "-o", "json"], OutputFormat::Json },
)]
fn test_new_output_format(args: &[&str], expected: OutputFormat) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::New { output, .. } => {
            assert!(matches!(output, expected));
        }
        _ => panic!("Expected New command"),
    }
}
```

**Verification:**
```bash
cargo test -p wk new_output
```

---

### Phase 4: Add Prefix Flag Tests

**Objective:** Ensure `--prefix` flag is tested at unit level.

**Files to modify:**
- `crates/cli/src/cli_tests/new_tests.rs`

**Add tests:**

```rust
#[test]
fn test_new_with_prefix() {
    let cli = parse(&["wk", "new", "task", "Test", "--prefix", "custom"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert_eq!(prefix, Some("custom".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_with_prefix_short() {
    let cli = parse(&["wk", "new", "task", "Test", "-p", "short"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert_eq!(prefix, Some("short".to_string()));
        }
        _ => panic!("Expected New command"),
    }
}

#[test]
fn test_new_prefix_default_none() {
    let cli = parse(&["wk", "new", "task", "Test"]).unwrap();
    match cli.command {
        Command::New { prefix, .. } => {
            assert!(prefix.is_none());
        }
        _ => panic!("Expected New command"),
    }
}
```

**Verification:**
```bash
cargo test -p wk new_prefix
```

---

### Phase 5: Add Label Expansion Tests

**Objective:** Test comma-separated label expansion.

**Files to modify:**
- `crates/cli/src/commands/new_tests.rs` (create if needed, or add to new.rs inline)

**Add tests for `expand_labels` function:**

```rust
#[cfg(test)]
mod expand_tests {
    use super::*;
    use yare::parameterized;

    #[parameterized(
        single = { vec!["urgent".into()], vec!["urgent"] },
        multiple = { vec!["a".into(), "b".into()], vec!["a", "b"] },
        comma_separated = { vec!["a,b,c".into()], vec!["a", "b", "c"] },
        mixed = { vec!["a,b".into(), "c".into()], vec!["a", "b", "c"] },
        whitespace = { vec!["  a  ,  b  ".into()], vec!["a", "b"] },
        empty_filter = { vec!["a,,b".into()], vec!["a", "b"] },
    )]
    fn test_expand_labels(input: Vec<String>, expected: Vec<&str>) {
        let result = expand_labels(&input);
        assert_eq!(result, expected);
    }
}
```

**Verification:**
```bash
cargo test -p wk expand_labels
```

---

### Phase 6: Integration Verification

**Objective:** Ensure all tests pass and coverage is improved.

**Run full test suite:**

```bash
# Unit tests
cargo test -p wk

# Spec tests
make spec-cli ARGS='--filter "new"'

# Full validation
make check
```

**Coverage check:**

```bash
make coverage
```

**Verify no regressions in new.bats:**

```bash
make spec ARGS='--file cli/unit/new.bats'
```

## Key Implementation Details

### Yare Parameterized Test Pattern

The project already uses yare extensively. The pattern is:

```rust
use yare::parameterized;

#[parameterized(
    test_case_name = { input_value, expected_value },
    another_case = { input, expected },
)]
fn test_function(input: InputType, expected: ExpectedType) {
    assert_eq!(actual_result(input), expected);
}
```

### Test File Organization

Following project convention:
- CLI parsing tests: `crates/cli/src/cli_tests/new_tests.rs`
- Command implementation tests: `crates/cli/src/commands/new_tests.rs`
- Model tests: `crates/cli/src/models/issue_tests.rs`

### Type Validation Location

Type validation happens at two levels:
1. **CLI level**: Empty string rejected by `non_empty_string` validator
2. **Implementation level**: Invalid type string rejected in `run_impl()` via `type_or_title.parse::<IssueType>()`

Unit tests should cover both levels.

### Existing Parameterized Tests to Reference

- `crates/core/src/issue_tests.rs`: IssueType parsing with yare
- `crates/cli/src/models/issue_tests.rs`: IssueType and Status tests
- `crates/cli/src/help_tests.rs`: Extensive yare usage for colorization tests

## Verification Plan

### Unit Tests

```bash
# Run all new command tests
cargo test -p wk new

# Run with verbose output
cargo test -p wk new -- --nocapture

# Run specific test
cargo test -p wk test_new_type_parsing
```

### Spec Tests

```bash
# Run new.bats
make spec ARGS='--file cli/unit/new.bats'

# Run with filter
make spec-cli ARGS='--filter "new creates"'
```

### Coverage

```bash
# Generate coverage report
make coverage

# Check new.rs coverage specifically
```

### Checklist

- [ ] All IssueType variants tested with yare parameterization
- [ ] Output format tests added (-o text/id/ids/json)
- [ ] Prefix flag tests added (--prefix, -p)
- [ ] Label expansion tests added (comma-separated)
- [ ] `cargo test -p wk` passes
- [ ] `make spec ARGS='--file cli/unit/new.bats'` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No duplicate test coverage between unit and integration tests
