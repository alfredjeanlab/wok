# Filter Status Implementation Plan

**Branch:** `feature/filter-status`

## Overview

Enhance the `wk list` and `wk search` filter expressions to support:

1. **`now` as a filter value** - Accept `now` in expressions like `closed < now` meaning "all closed issues up to the current time"
2. **Bare status field names** - Accept status fields without operators: `closed`, `skipped`, `completed` as shorthand for "has this status"

These additions make filters more intuitive and reduce typing for common queries.

## Project Structure

Key files to modify:

```
crates/cli/src/
├── filter/
│   ├── expr.rs          # Add Now variant to FilterValue
│   ├── parser.rs        # Parse "now" keyword and bare status fields
│   ├── eval.rs          # Handle Now value (treat as current time)
│   ├── parser_tests.rs  # Add tests for new parsing
│   └── eval_tests.rs    # Add tests for Now evaluation
├── cli.rs               # Update help text

checks/specs/cli/
└── unit/list.bats       # Add specs for new filter syntax
```

## Dependencies

None - all changes are internal to existing crates.

## Implementation Phases

### Phase 1: Add `now` as a filter value

**Goal**: Parse and evaluate `now` as a special timestamp value.

**File: `crates/cli/src/filter/expr.rs`**

Add `Now` variant to `FilterValue`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// A duration like `3d`, `1w`, `24h`.
    Duration(Duration),
    /// An absolute date like `2024-01-01`.
    Date(NaiveDate),
    /// The current time (now).
    Now,
}
```

**File: `crates/cli/src/filter/parser.rs`**

Update `parse_value()` to check for "now" before trying date/duration:

```rust
fn parse_value(s: &str) -> Result<FilterValue> {
    // Check for "now" keyword first
    if s.eq_ignore_ascii_case("now") {
        return Ok(FilterValue::Now);
    }

    // Try parsing as a date first (YYYY-MM-DD format)
    if let Some(date) = try_parse_date(s) {
        return Ok(FilterValue::Date(date));
    }

    // Try parsing as a duration
    parse_duration(s).map(FilterValue::Duration)
}
```

**File: `crates/cli/src/filter/eval.rs`**

Update `FilterExpr::matches()` to handle `Now`:

```rust
match &self.value {
    FilterValue::Duration(threshold) => {
        let age = now.signed_duration_since(issue_time);
        self.op.compare_duration(age, *threshold)
    }
    FilterValue::Date(date) => {
        let threshold = date
            .and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default())
            .and_utc();
        self.op.compare_datetime(issue_time, threshold)
    }
    FilterValue::Now => {
        // "now" compares the issue timestamp directly to the current time
        self.op.compare_datetime(issue_time, now)
    }
}
```

**Verification**:
- `cargo check` passes
- Add unit tests for `now` parsing and evaluation
- `cargo test` passes

### Phase 2: Support bare status field names

**Goal**: Parse `closed`, `skipped`, `completed` without operators as shorthand for existence checks.

The semantics:
- `closed` alone means "any issue with closed status" (equivalent to `closed >= 0s`)
- `skipped` alone means "any skipped issue"
- `completed` alone means "any completed issue"

**File: `crates/cli/src/filter/parser.rs`**

Update `parse_filter()` to handle bare status fields:

```rust
pub fn parse_filter(input: &str) -> Result<FilterExpr> {
    let input = input.trim();

    if input.is_empty() {
        return Err(Error::InvalidInput("empty filter expression".to_string()));
    }

    // Extract field name (until whitespace or operator character)
    let (field_str, rest) = split_field(input)?;
    let field = parse_field(field_str)?;

    // Check if this is a bare status field (no operator/value)
    let rest = rest.trim_start();
    if rest.is_empty() {
        // Only allow bare syntax for status-aware fields
        if matches!(field, FilterField::Completed | FilterField::Skipped | FilterField::Closed) {
            // Bare status field: "closed" means "has closed status"
            // Equivalent to "closed >= 0s" (any time since closed)
            return Ok(FilterExpr {
                field,
                op: CompareOp::Ge,
                value: FilterValue::Duration(Duration::zero()),
            });
        } else {
            return Err(Error::InvalidInput(format!(
                "filter expression requires operator and value: \"{input}\""
            )));
        }
    }

    // Continue with normal parsing...
    let (op, rest) = parse_operator(rest)?;
    // ... rest unchanged
}
```

**Verification**:
- `cargo check` passes
- Add unit tests for bare status field parsing
- `cargo test` passes

### Phase 3: Update help text

**Goal**: Document the new filter syntax in CLI help.

**File: `crates/cli/src/cli.rs`**

Update list command help text (around line 213-219):

```rust
Filter Expressions (-q/--filter):\n  \
  Syntax: FIELD [OPERATOR VALUE]\n  \
  Fields: age, activity, completed, skipped, closed\n  \
  Status shortcuts: 'closed', 'skipped', 'completed' (no operator needed)\n  \
  Operators: < <= > >= = != (or: lt lte gt gte eq ne)\n  \
  Values: durations (30d, 1w, 24h, 5m, 10s), dates (2024-01-01), or 'now'\n  \
  Duration units: ms, s, m, h, d, w, M (30d), y (365d)"
```

Update search command help text similarly.

**Verification**: `wk list --help` shows updated syntax

### Phase 4: Update specs

**Goal**: Add acceptance tests for new filter syntax.

**File: `checks/specs/cli/unit/list.bats`**

Add new test cases:

```bash
@test "list --filter accepts 'now' as value" {
    # Create and close an issue
    id=$(create_issue task "NowFilter Issue")
    "$WK_BIN" close "$id" --reason "test"

    # closed < now should match (closed before current time)
    run "$WK_BIN" list --filter "closed < now"
    assert_success
    assert_output --partial "NowFilter Issue"

    # closed > now should not match (nothing closed in the future)
    run "$WK_BIN" list --filter "closed > now"
    assert_success
    refute_output --partial "NowFilter Issue"
}

@test "list --filter accepts bare status fields" {
    # Create issues with different states
    open_id=$(create_issue task "BareFilter Open")
    done_id=$(create_issue task "BareFilter Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    skipped_id=$(create_issue task "BareFilter Skipped")
    "$WK_BIN" close "$skipped_id" --reason "wontfix"

    # Bare "closed" matches any terminal state
    run "$WK_BIN" list --filter "closed"
    assert_success
    assert_output --partial "BareFilter Done"
    assert_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Open"

    # Bare "completed" matches only Status::Done
    run "$WK_BIN" list --filter "completed"
    assert_success
    assert_output --partial "BareFilter Done"
    refute_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Open"

    # Bare "skipped" matches only Status::Closed
    run "$WK_BIN" list --filter "skipped"
    assert_success
    assert_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Done"
    refute_output --partial "BareFilter Open"
}

@test "list --filter bare fields work with aliases" {
    id=$(create_issue task "AliasFilter Done")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"

    # "done" is alias for "completed"
    run "$WK_BIN" list --filter "done"
    assert_success
    assert_output --partial "AliasFilter Done"

    # "cancelled" is alias for "skipped"
    skipped_id=$(create_issue task "AliasFilter Skipped")
    "$WK_BIN" close "$skipped_id" --reason "test"

    run "$WK_BIN" list --filter "cancelled"
    assert_success
    assert_output --partial "AliasFilter Skipped"
    refute_output --partial "AliasFilter Done"
}

@test "list --filter rejects bare non-status fields" {
    # Bare "age" without operator should fail
    run "$WK_BIN" list --filter "age"
    assert_failure
    assert_output --partial "requires operator"

    # Bare "updated" without operator should fail
    run "$WK_BIN" list --filter "updated"
    assert_failure
    assert_output --partial "requires operator"
}
```

**Verification**: `make spec-cli` passes

### Phase 5: Unit tests

**Goal**: Add comprehensive unit tests for new functionality.

**File: `crates/cli/src/filter/parser_tests.rs`**

```rust
#[test]
fn parses_now_value() {
    let expr = parse_filter("closed < now").unwrap();
    assert_eq!(expr.field, FilterField::Closed);
    assert_eq!(expr.op, CompareOp::Lt);
    assert_eq!(expr.value, FilterValue::Now);
}

#[test]
fn parses_now_case_insensitive() {
    let expr = parse_filter("closed < NOW").unwrap();
    assert_eq!(expr.value, FilterValue::Now);

    let expr = parse_filter("closed < Now").unwrap();
    assert_eq!(expr.value, FilterValue::Now);
}

#[test]
fn parses_bare_closed() {
    let expr = parse_filter("closed").unwrap();
    assert_eq!(expr.field, FilterField::Closed);
    assert_eq!(expr.op, CompareOp::Ge);
    assert!(matches!(expr.value, FilterValue::Duration(d) if d.is_zero()));
}

#[test]
fn parses_bare_completed() {
    let expr = parse_filter("completed").unwrap();
    assert_eq!(expr.field, FilterField::Completed);
}

#[test]
fn parses_bare_skipped() {
    let expr = parse_filter("skipped").unwrap();
    assert_eq!(expr.field, FilterField::Skipped);
}

#[test]
fn parses_bare_done_alias() {
    let expr = parse_filter("done").unwrap();
    assert_eq!(expr.field, FilterField::Completed);
}

#[test]
fn parses_bare_cancelled_alias() {
    let expr = parse_filter("cancelled").unwrap();
    assert_eq!(expr.field, FilterField::Skipped);
}

#[test]
fn rejects_bare_age() {
    let result = parse_filter("age");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires operator"));
}

#[test]
fn rejects_bare_updated() {
    let result = parse_filter("updated");
    assert!(result.is_err());
}
```

**File: `crates/cli/src/filter/eval_tests.rs`**

```rust
#[test]
fn now_value_matches_past_timestamps() {
    let now = Utc::now();
    let issue = Issue {
        closed_at: Some(now - Duration::hours(1)),
        status: Status::Done,
        // ... other fields
    };

    // closed < now should match (closed 1 hour ago)
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Lt,
        value: FilterValue::Now,
    };
    assert!(expr.matches(&issue, now));

    // closed > now should not match
    let expr = FilterExpr {
        field: FilterField::Closed,
        op: CompareOp::Gt,
        value: FilterValue::Now,
    };
    assert!(!expr.matches(&issue, now));
}
```

**Verification**: `cargo test` passes

### Phase 6: Final validation

**Goal**: Run full validation suite.

1. `cargo check` - no errors
2. `cargo clippy` - no warnings
3. `cargo test` - all unit tests pass
4. `cargo fmt` - code formatted
5. `make spec-cli` - CLI specs pass
6. Manual testing:
   ```bash
   # Test 'now' value
   wk new task "Test now"
   wk close TEST-xxxx --reason "test"
   wk list -q "closed < now"      # Should show the issue
   wk list -q "closed > now"      # Should not show anything

   # Test bare status fields
   wk list -q "closed"            # Should show all closed issues
   wk list -q "completed"         # Should show only done issues
   wk list -q "skipped"           # Should show only skipped issues
   ```

## Key Implementation Details

### Semantics of `now`

The `now` value represents the current timestamp when the filter is evaluated. It's useful for:

- `closed < now` - All issues closed before the current time (effectively all closed issues)
- `created < now` - All issues created before now (all issues)
- `updated > now` - Issues updated after now (none - useful for edge case testing)

The primary use case is `closed < now` as a more readable alternative to `closed >= 0s`.

### Semantics of bare status fields

Bare status fields are syntactic sugar:

| Input | Equivalent to | Meaning |
|-------|--------------|---------|
| `closed` | `closed >= 0s` | Any issue in a terminal state |
| `completed` | `completed >= 0s` | Any successfully completed issue |
| `skipped` | `skipped >= 0s` | Any cancelled/skipped issue |
| `done` | `completed >= 0s` | Alias for completed |
| `cancelled` | `skipped >= 0s` | Alias for skipped |

Non-status fields (`age`, `updated`, `activity`, `created`) still require explicit operators because their meaning without an operator is ambiguous.

### Error messages

For bare non-status fields, provide helpful error:

```
error: filter expression requires operator and value: "age"
```

## Verification Plan

1. **Unit tests**: Run `cargo test` to verify parsing and evaluation
2. **Spec tests**: Run `make spec-cli` to verify end-to-end behavior
3. **Manual testing**: Test all new filter syntaxes interactively
4. **Landing checklist**: Follow `crates/cli/CLAUDE.md` checklist
