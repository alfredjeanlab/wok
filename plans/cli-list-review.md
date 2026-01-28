# Implementation Plan: CLI List Command Review

## Overview

Review and ensure comprehensive test coverage for the `wk list` command by:
1. Auditing all 609 lines in `list.bats` (16 test cases)
2. Verifying corresponding Rust unit tests exist
3. Converting filter expression tests to use parameterization with `yare`
4. Ensuring output format tests are parameterized

## Project Structure

```
crates/cli/
├── src/
│   ├── cli_tests/
│   │   └── list_tests.rs          # CLI parsing tests (232 lines)
│   ├── commands/
│   │   ├── list.rs                # Implementation
│   │   ├── list_tests.rs          # Command logic tests (709 lines)
│   │   └── filtering_tests.rs     # Filter group tests (105 lines)
│   ├── filter/
│   │   ├── parser.rs              # Filter expression parser
│   │   ├── parser_tests.rs        # Parser tests (527 lines) - uses yare
│   │   ├── eval_tests.rs          # Evaluation tests (705 lines)
│   │   └── expr_tests.rs          # Expression tests
│   └── schema/
│       └── list.rs                # JSON output schema
├── tests/
│   └── list.rs                    # Integration tests (307 lines)

tests/specs/cli/unit/
└── list.bats                      # BATS specs (609 lines, 16 test cases)
```

## Dependencies

Already available:
- `yare = "3"` in `[dev-dependencies]`

## Implementation Phases

### Phase 1: Audit Coverage Gap Analysis

**Objective:** Map all 16 `list.bats` tests to Rust unit tests.

| BATS Test | Lines | Rust Coverage | Gap |
|-----------|-------|---------------|-----|
| `list shows issues with status filtering` | 4-52 | `list_tests.rs`: `test_run_impl_default`, `test_run_impl_with_status_filter`, `list.rs`: `list_status_or_filter` | **Missing: comma-separated status OR** |
| `list filters by type, label, and blocked` | 54-121 | `cli_tests/list_tests.rs`, `commands/list_tests.rs` | Covered |
| `list --output json outputs valid data` | 123-162 | `test_run_impl_json_format_*` | **Missing: JSON field validation tests** |
| `list sorts by priority` | 164-209 | `test_list_sorts_by_priority_asc`, `test_list_same_priority_sorts_by_created_at_desc`, `test_list_priority_tag_precedence` | Covered |
| `list --filter expressions` | 211-257 | `filter/parser_tests.rs`, `filter/eval_tests.rs` | **Missing: integration-level filter combo tests** |
| `list --limit and --filter closed` | 259-316 | `test_run_impl_*` with limit | **Missing: JSON metadata tests** |
| `list --filter completed only shows done status` | 318-339 | `eval_tests.rs`: `completed_filter_*` | Covered |
| `list --filter skipped only shows closed status` | 341-362 | `eval_tests.rs`: `skipped_filter_*` | Covered |
| `list --filter closed shows both done and closed status` | 364-379 | `eval_tests.rs`: `closed_filter_matches_*` | Covered |
| `list --filter with word operators` | 381-417 | `parser_tests.rs`: `parse_operator_word` (parameterized) | Covered |
| `list defaults to 100 results` | 419-430 | **None** | **Missing: default limit test** |
| `list --limit 0 shows all results` | 432-443 | **None** | **Missing: unlimited results test** |
| `list explicit limit overrides default` | 445-456 | **None** | **Missing: explicit limit test** |
| `list --output ids outputs space-separated IDs` | 458-472 | `test_run_impl_ids_format_outputs_space_separated` | Covered |
| `list --output ids works with filters` | 474-481 | `test_run_impl_ids_format_with_filters` | Covered |
| `list --output ids respects limit` | 483-492 | **None** | **Missing: ids format with limit** |
| `list -o ids works as short flag` | 494-499 | Covered by CLI parsing | Covered |
| `list --output ids can be piped` | 501-510 | Integration test appropriate | N/A |
| `list --output ids composes with batch` | 512-529 | Integration test appropriate | N/A |
| `list --filter accepts 'now' as value` | 531-545 | `parser_tests.rs`: `parses_now_*`, `eval_tests.rs`: `now_value_*` | Covered |
| `list --filter accepts bare status fields` | 547-576 | `parser_tests.rs`: `parses_bare_*` | Covered |
| `list --filter bare fields work with aliases` | 578-596 | `parser_tests.rs`: `parses_bare_done_alias`, `parses_bare_cancelled_alias` | Covered |
| `list --filter rejects bare non-status fields` | 598-608 | `parser_tests.rs`: `rejects_bare_*` | Covered |

**Files to review:**
- `crates/cli/src/commands/list_tests.rs`
- `crates/cli/tests/list.rs`

**Verification:**
- Complete audit table with all gaps identified

---

### Phase 2: Add Missing Limit Tests

**Objective:** Add unit tests for limit behavior.

**Files to modify:**
- `crates/cli/src/commands/list_tests.rs`

**Add tests:**

```rust
use yare::parameterized;

#[parameterized(
    default_limit = { None, 100 },      // No explicit limit = 100
    explicit_50 = { Some(50), 50 },     // --limit 50
    unlimited = { Some(0), usize::MAX }, // --limit 0 = unlimited
    explicit_200 = { Some(200), 200 },  // --limit 200 (> default)
)]
fn test_effective_limit(input: Option<usize>, expected_max: usize) {
    let db = setup_db();
    // Create enough issues to test limits
    for i in 0..150 {
        create_issue(&db, &format!("limit-{}", i), Status::Todo, IssueType::Task);
    }

    // Verify limit is applied correctly
    // ...
}

#[test]
fn test_default_limit_is_100() {
    let db = setup_db();
    for i in 0..110 {
        create_issue(&db, &format!("default-{}", i), Status::Todo, IssueType::Task);
    }

    let result = run_impl(
        &db,
        vec![], vec![], vec![], vec![], false, vec![],
        None,  // No explicit limit
        false, false, OutputFormat::Text,
    );
    assert!(result.is_ok());
    // Output would have at most 100 issues
}

#[test]
fn test_limit_zero_is_unlimited() {
    let db = setup_db();
    for i in 0..110 {
        create_issue(&db, &format!("unlimited-{}", i), Status::Todo, IssueType::Task);
    }

    let result = run_impl(
        &db,
        vec![], vec![], vec![], vec![], false, vec![],
        Some(0),  // Unlimited
        false, false, OutputFormat::Text,
    );
    assert!(result.is_ok());
    // Output would have all 110 issues
}
```

**Verification:**
```bash
cargo test -p wk test_effective_limit
cargo test -p wk limit
```

---

### Phase 3: Add JSON Output Validation Tests

**Objective:** Test JSON output structure and metadata fields.

**Files to modify:**
- `crates/cli/src/commands/list_tests.rs`

**Add tests:**

```rust
#[test]
fn test_json_output_includes_filters_applied() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    // With filter, should include filters_applied
    let result = run_impl(
        &db,
        vec![], vec![], vec![], vec![], false,
        vec!["age < 1d".to_string()],  // Filter specified
        None, false, false, OutputFormat::Json,
    );
    assert!(result.is_ok());
    // Verify filters_applied is not None
}

#[test]
fn test_json_output_includes_limit_when_specified() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec![], vec![], vec![], vec![], false, vec![],
        Some(10),  // Explicit limit
        false, false, OutputFormat::Json,
    );
    assert!(result.is_ok());
    // Verify limit field is 10
}

#[test]
fn test_json_output_excludes_null_metadata() {
    let db = setup_db();
    create_issue(&db, "test-1", Status::Todo, IssueType::Task);

    let result = run_impl(
        &db,
        vec![], vec![], vec![], vec![], false, vec![],
        None,  // No limit
        false, false, OutputFormat::Json,
    );
    assert!(result.is_ok());
    // Verify limit and filters_applied are not in output (skip_serializing_if)
}
```

**Verification:**
```bash
cargo test -p wk json_output
```

---

### Phase 4: Parameterize Output Format Tests

**Objective:** Use yare for output format CLI parsing tests.

**Files to modify:**
- `crates/cli/src/cli_tests/list_tests.rs`

**Add parameterized tests:**

```rust
use yare::parameterized;

#[parameterized(
    text_default = { &["wk", "list"], OutputFormat::Text },
    json_long = { &["wk", "list", "--output", "json"], OutputFormat::Json },
    json_short = { &["wk", "list", "-o", "json"], OutputFormat::Json },
    ids_long = { &["wk", "list", "--output", "ids"], OutputFormat::Id },
    ids_short = { &["wk", "list", "-o", "ids"], OutputFormat::Id },
    id_alias = { &["wk", "list", "-o", "id"], OutputFormat::Id },
)]
fn test_list_output_format(args: &[&str], expected: OutputFormat) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::List { output, .. } => {
            assert!(matches!(output, expected));
        }
        _ => panic!("Expected List command"),
    }
}
```

**Verification:**
```bash
cargo test -p wk test_list_output_format
```

---

### Phase 5: Parameterize Filter Operator CLI Tests

**Objective:** Ensure all filter operators are tested at CLI level.

**Files to modify:**
- `crates/cli/src/cli_tests/list_tests.rs`

**Note:** The `filter/parser_tests.rs` already has comprehensive parameterized tests for operators. Verify these cover all cases:

```rust
// Already exists in parser_tests.rs:
#[parameterized(
    lt = { "age lt 3d", CompareOp::Lt },
    lte = { "age lte 3d", CompareOp::Le },
    gt = { "age gt 3d", CompareOp::Gt },
    gte = { "age gte 3d", CompareOp::Ge },
    eq = { "age eq 3d", CompareOp::Eq },
    ne = { "age ne 3d", CompareOp::Ne },
    lt_upper = { "age LT 3d", CompareOp::Lt },
    gte_upper = { "age GTE 3d", CompareOp::Ge },
    lt_mixed = { "age Lt 3d", CompareOp::Lt },
)]
fn parse_operator_word(input: &str, expected: CompareOp) { ... }
```

**Add CLI-level filter flag tests:**

```rust
#[parameterized(
    single_filter = { &["wk", "list", "-q", "age < 1d"], vec!["age < 1d"] },
    multiple_filters = { &["wk", "list", "-q", "age < 1d", "-q", "updated < 1h"],
                         vec!["age < 1d", "updated < 1h"] },
    long_flag = { &["wk", "list", "--filter", "closed < 1w"], vec!["closed < 1w"] },
)]
fn test_list_filter_parsing(args: &[&str], expected: Vec<&str>) {
    let cli = parse(args).unwrap();
    match cli.command {
        Command::List { filter, .. } => {
            let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
            assert_eq!(filter, expected);
        }
        _ => panic!("Expected List command"),
    }
}
```

**Verification:**
```bash
cargo test -p wk test_list_filter
```

---

### Phase 6: Integration Verification

**Objective:** Ensure all tests pass and coverage is complete.

**Run full test suite:**

```bash
# Unit tests
cargo test -p wk

# Spec tests for list
make spec ARGS='--file cli/unit/list.bats'

# Full validation
make check
```

**Coverage check:**

```bash
make coverage
```

## Key Implementation Details

### Test File Organization

Following project convention from `CLAUDE.md`:
- CLI parsing tests: `crates/cli/src/cli_tests/list_tests.rs`
- Command implementation tests: `crates/cli/src/commands/list_tests.rs`
- Filter parsing tests: `crates/cli/src/filter/parser_tests.rs`
- Filter evaluation tests: `crates/cli/src/filter/eval_tests.rs`

### Yare Parameterized Test Pattern

The project uses yare extensively. Pattern:

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

### Filter Expression Test Coverage Summary

The filter module is **well-tested** with parameterization:

| Test Area | File | Tests | Parameterized |
|-----------|------|-------|---------------|
| Field parsing | `parser_tests.rs` | 12 | No (could add) |
| Operators (symbols) | `parser_tests.rs` | 7 | No |
| Operators (words) | `parser_tests.rs` | 1 | **Yes (9 cases)** |
| Duration parsing | `parser_tests.rs` | 15 | No (could add) |
| Date parsing | `parser_tests.rs` | 5 | No |
| Now value | `parser_tests.rs` | 4 | No |
| Bare fields | `parser_tests.rs` | 11 | No |
| Age evaluation | `eval_tests.rs` | 11 | No (could add) |
| Updated evaluation | `eval_tests.rs` | 3 | No |
| Closed evaluation | `eval_tests.rs` | 14 | No |
| Now evaluation | `eval_tests.rs` | 7 | No |

### Existing Parameterization

Found in `filter/parser_tests.rs`:
```rust
#[parameterized(
    lt = { "age lt 3d", CompareOp::Lt },
    ...
)]
fn parse_operator_word(input: &str, expected: CompareOp) { ... }
```

### Output Format Values

From schema:
- `Text` - Default markdown-style output
- `Json` - JSON with `issues`, `filters_applied`, `limit`
- `Id` - Space-separated IDs (accepts both `id` and `ids`)

## Verification Plan

### Unit Tests

```bash
# Run all list command tests
cargo test -p wk list

# Run with verbose output
cargo test -p wk list -- --nocapture

# Run specific test
cargo test -p wk test_effective_limit
```

### Spec Tests

```bash
# Run list.bats
make spec ARGS='--file cli/unit/list.bats'

# Run with filter
make spec-cli ARGS='--filter "list"'
```

### Coverage

```bash
# Generate coverage report
make coverage
```

### Checklist

- [ ] All 16 BATS test cases have corresponding unit tests
- [ ] Limit behavior tests added (default, explicit, unlimited)
- [ ] JSON metadata tests added (filters_applied, limit)
- [ ] Output format parsing is parameterized
- [ ] Filter flag parsing tests added
- [ ] `cargo test -p wk` passes
- [ ] `make spec ARGS='--file cli/unit/list.bats'` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Coverage meets 85% threshold
