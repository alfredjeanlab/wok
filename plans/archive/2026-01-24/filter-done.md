# Filter Done Implementation Plan

**Root Feature:** `wok-9252`

## Overview

Fix the filter behavior in `wk list` and `wk search` so that filter fields correctly distinguish between successfully completed issues and cancelled issues:

- `completed` (or `done`) → only issues completed with `wk done` (Status::Done)
- `skipped` (or `cancelled`) → only issues closed with `wk close --reason` (Status::Closed)
- `closed` → any terminal state (both Status::Done and Status::Closed)

Also simplify help text to show canonical field names only.

## Project Structure

Key files to modify:

```
crates/cli/src/
├── filter/
│   ├── expr.rs          # Add Done and Cancelled FilterField variants
│   ├── parser.rs        # Route aliases to new fields
│   ├── eval.rs          # Check issue.status for Done/Cancelled
│   ├── expr_tests.rs    # Update tests
│   ├── parser_tests.rs  # Update tests
│   └── eval_tests.rs    # Update tests
├── cli.rs               # Update help text
└── models/issue.rs      # (reference only - Status enum)

checks/specs/cli/
├── unit/list.bats       # Add specs for new filter behavior
└── integration/filtering.bats  # Add integration specs
```

## Dependencies

None - all changes are internal to existing crates.

## Implementation Phases

### Phase 1: Update FilterField enum and parser

**Goal**: Add new filter field variants and route aliases correctly.

**File: `crates/cli/src/filter/expr.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Age,
    Updated,
    /// Successfully completed issues only (Status::Done)
    Completed,
    /// Cancelled/skipped issues only (Status::Closed)
    Skipped,
    /// Any terminal state (Status::Done or Status::Closed)
    Closed,
}

impl FilterField {
    pub fn valid_names() -> &'static str {
        "age, created, activity, updated, completed, done, skipped, cancelled, closed"
    }
}
```

**File: `crates/cli/src/filter/parser.rs`**

Update `parse_field()`:

```rust
fn parse_field(s: &str) -> Result<FilterField> {
    match s.to_lowercase().as_str() {
        "age" | "created" => Ok(FilterField::Age),
        "updated" | "activity" => Ok(FilterField::Updated),
        "completed" | "done" => Ok(FilterField::Completed),
        "skipped" | "cancelled" => Ok(FilterField::Skipped),
        "closed" => Ok(FilterField::Closed),
        _ => Err(Error::InvalidInput(...))
    }
}
```

**Verification**: `cargo check` passes, parser tests updated.

### Phase 2: Update filter evaluation

**Goal**: Make `Completed` and `Skipped` filter fields check issue status.

**File: `crates/cli/src/filter/eval.rs`**

Update `FilterExpr::matches()` to check status for new fields:

```rust
pub fn matches(&self, issue: &Issue, now: DateTime<Utc>) -> bool {
    // For Completed: must be Status::Done
    // For Skipped: must be Status::Closed
    // For Closed: either Status::Done or Status::Closed

    let status_matches = match self.field {
        FilterField::Completed => issue.status == Status::Done,
        FilterField::Skipped => issue.status == Status::Closed,
        FilterField::Closed => {
            issue.status == Status::Done || issue.status == Status::Closed
        }
        _ => true, // Age and Updated don't filter by status
    };

    if !status_matches {
        return false;
    }

    let issue_time = match self.field {
        FilterField::Age => Some(issue.created_at),
        FilterField::Updated => Some(issue.updated_at),
        FilterField::Completed | FilterField::Skipped | FilterField::Closed => issue.closed_at,
    };

    // Rest of existing logic...
}
```

**Verification**: Unit tests pass for new filter behavior.

### Phase 3: Update help text

**Goal**: Simplify help text to show canonical field names only.

**File: `crates/cli/src/cli.rs`**

Update `List` command help (around line 213-219):

```rust
Filter Expressions (-q/--filter):\n  \
  Syntax: FIELD OPERATOR VALUE\n  \
  Fields: age, activity, completed, skipped, closed\n  \
  Operators: < <= > >= = != (or: lt lte gt gte eq ne)\n  \
  Values: durations (30d, 1w, 24h, 5m, 10s) or dates (2024-01-01)\n  \
  Duration units: ms, s, m, h, d, w, M (30d), y (365d)"
```

Update `Search` command help (around line 306-312) with same text.

Changes:
1. Remove aliases from Fields line (show only: age, activity, completed, skipped, closed)
2. Remove "Word operators are shell-friendly (no quoting needed)" line

**Verification**: `wk list --help` and `wk search --help` show updated text.

### Phase 4: Update unit tests

**Goal**: Ensure all filter tests reflect new behavior.

**Files to update**:
- `crates/cli/src/filter/parser_tests.rs` - test new field parsing
- `crates/cli/src/filter/eval_tests.rs` - test status-aware evaluation
- `crates/cli/src/filter/expr_tests.rs` - update valid_names test if any

**Key test cases**:
1. `completed` field only matches Status::Done issues
2. `skipped` field only matches Status::Closed issues
3. `closed` field matches both Status::Done and Status::Closed
4. Aliases work: `done` → Completed, `cancelled` → Skipped
5. Open issues (Status::Todo, Status::InProgress) don't match any of the three

**Verification**: `cargo test` passes.

### Phase 5: Update specs

**Goal**: Add acceptance tests for new filter behavior.

**File: `checks/specs/cli/unit/list.bats`** or new file

Add specs:

```bash
@test "completed filter only shows done issues" {
    d=$(create_issue task "Done task")
    c=$(create_issue task "Closed task")
    "$WK_BIN" done "$d"
    "$WK_BIN" close "$c" --reason "wontfix"

    run "$WK_BIN" list -q "completed < 1d"
    assert_success
    assert_output --partial "Done task"
    refute_output --partial "Closed task"
}

@test "skipped filter only shows closed issues" {
    d=$(create_issue task "Done task")
    c=$(create_issue task "Closed task")
    "$WK_BIN" done "$d"
    "$WK_BIN" close "$c" --reason "wontfix"

    run "$WK_BIN" list -q "skipped < 1d"
    assert_success
    assert_output --partial "Closed task"
    refute_output --partial "Done task"
}

@test "closed filter shows both done and closed issues" {
    d=$(create_issue task "Done task")
    c=$(create_issue task "Closed task")
    "$WK_BIN" done "$d"
    "$WK_BIN" close "$c" --reason "wontfix"

    run "$WK_BIN" list -q "closed < 1d"
    assert_success
    assert_output --partial "Done task"
    assert_output --partial "Closed task"
}
```

**Verification**: `make spec-cli` passes.

### Phase 6: Final validation

**Goal**: Run full validation suite.

1. `cargo check` - no errors
2. `cargo clippy` - no warnings
3. `cargo test` - all unit tests pass
4. `cargo fmt` - code formatted
5. `make spec-cli` - CLI specs pass
6. Manual testing of filter commands

## Key Implementation Details

### Status vs closed_at

The Issue model has both:
- `status: Status` - enum with Todo, InProgress, Done, Closed
- `closed_at: Option<DateTime<Utc>>` - computed from event log

Both `wk done` and `wk close` set `closed_at`, but they set different statuses:
- `wk done` → Status::Done
- `wk close --reason` → Status::Closed

The new filter fields use both:
- Time comparison uses `closed_at`
- Status check uses `issue.status`

### Backward compatibility

The `closed` field retains its current behavior (matching any terminal state). Users who want the old behavior of `done`/`completed` can use `closed` instead.

## Verification Plan

1. **Unit tests**: Run `cargo test` to verify filter parsing and evaluation
2. **Spec tests**: Run `make spec-cli` to verify end-to-end behavior
3. **Manual testing**:
   ```bash
   # Create test issues
   wk new task "Test done"
   wk new task "Test closed"
   wk done TEST-xxxx
   wk close TEST-yyyy --reason "wontfix"

   # Verify filters
   wk list -q "completed < 1d"   # Should show only done issue
   wk list -q "skipped < 1d"     # Should show only closed issue
   wk list -q "closed < 1d"      # Should show both

   # Verify help text
   wk list --help
   wk search --help
   ```
4. **Landing checklist**: Follow `crates/cli/CLAUDE.md` checklist
