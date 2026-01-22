# Partial Bulk Updates for Lifecycle Commands

## Overview

Implement partial bulk update behavior for lifecycle commands (`start`, `done`, `close`, `reopen`). When multiple IDs are passed but some are unknown or have invalid transitions, the command should:
1. Perform updates on all valid IDs
2. Print individual success messages as operations complete
3. Print a summary line showing how many were transitioned
4. List unknown IDs separately from transition failures

## Project Structure

```
crates/cli/src/
├── commands/
│   └── lifecycle.rs          # Main implementation changes
├── error.rs                   # Add PartialBulkError variant
└── commands/lifecycle_tests.rs  # Unit tests

checks/specs/cli/unit/
└── lifecycle.bats            # Update/add spec tests
```

## Dependencies

No new external dependencies. Uses existing:
- `thiserror` for error handling
- Standard library collections for result accumulation

## Implementation Phases

### Phase 1: Add BulkResult Type and Error Variant

Create a `BulkResult` struct to track outcomes of bulk operations.

**File**: `crates/cli/src/commands/lifecycle.rs`

```rust
/// Result of a bulk lifecycle operation
#[derive(Default)]
pub(crate) struct BulkResult {
    /// Number of issues successfully transitioned
    pub success_count: usize,
    /// IDs that were not found in the database
    pub unknown_ids: Vec<String>,
    /// IDs that failed due to invalid transitions (with error message)
    pub transition_failures: Vec<(String, String)>,
}

impl BulkResult {
    /// Returns true if all operations succeeded
    pub fn is_success(&self) -> bool {
        self.unknown_ids.is_empty() && self.transition_failures.is_empty()
    }

    /// Returns true if any operations succeeded
    pub fn has_successes(&self) -> bool {
        self.success_count > 0
    }

    /// Returns the total number of failures
    pub fn failure_count(&self) -> usize {
        self.unknown_ids.len() + self.transition_failures.len()
    }
}
```

**File**: `crates/cli/src/error.rs`

Add a new error variant for partial bulk failures:

```rust
#[error("some operations failed: {succeeded} succeeded, {failed} failed")]
PartialBulkFailure {
    succeeded: usize,
    failed: usize,
    unknown_ids: Vec<String>,
    transition_failures: Vec<(String, String)>,
},
```

### Phase 2: Modify `_single` Functions to Return Operation-Specific Errors

Change the `_single` functions to return `Result<(), Error>` with specific error types that the `_impl` functions can categorize.

Currently the `_single` functions already return typed errors:
- `Error::IssueNotFound(id)` - when ID doesn't exist
- `Error::InvalidTransition { from, to, valid_targets }` - invalid state transition

No changes needed to single functions - they already return categorizable errors.

### Phase 3: Modify `_impl` Functions for Partial Updates

Update each `_impl` function to:
1. Iterate through all IDs, capturing results
2. Continue on errors instead of failing fast
3. Accumulate `BulkResult`
4. Print summary and return appropriate error

**Example for `start_impl`**:

```rust
pub(crate) fn start_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
) -> Result<()> {
    let mut result = BulkResult::default();

    for id in ids {
        match start_single(db, config, work_dir, id) {
            Ok(()) => result.success_count += 1,
            Err(Error::IssueNotFound(ref unknown_id)) => {
                result.unknown_ids.push(unknown_id.clone());
            }
            Err(Error::InvalidTransition { ref from, ref to, .. }) => {
                let msg = format!("cannot go from {} to {}", from, to);
                result.transition_failures.push((id.clone(), msg));
            }
            Err(e) => {
                // Unexpected errors should still fail fast
                return Err(e);
            }
        }
    }

    print_bulk_summary(&result, "started");

    if result.is_success() {
        Ok(())
    } else {
        Err(Error::PartialBulkFailure {
            succeeded: result.success_count,
            failed: result.failure_count(),
            unknown_ids: result.unknown_ids,
            transition_failures: result.transition_failures,
        })
    }
}
```

### Phase 4: Add Summary Output Function

Add a helper function to print the summary consistently:

```rust
/// Print summary for bulk operations
fn print_bulk_summary(result: &BulkResult, action_verb: &str) {
    // Only print summary if there were multiple items OR failures
    if result.success_count + result.failure_count() <= 1 && result.is_success() {
        return;
    }

    // Summary line
    let total = result.success_count + result.failure_count();
    println!("{} {} of {} issues", action_verb.to_title_case(), result.success_count, total);

    // List unknown IDs
    if !result.unknown_ids.is_empty() {
        println!("Unknown IDs: {}", result.unknown_ids.join(", "));
    }

    // List transition failures
    for (id, reason) in &result.transition_failures {
        eprintln!("  {}: {}", id, reason);
    }
}
```

### Phase 5: Apply Pattern to All Lifecycle Commands

Apply the same pattern to:
- `done_impl`
- `close_impl`
- `reopen_impl`

Each will need minor adjustments for their specific action verbs:
- start → "Started"
- done → "Completed"
- close → "Closed"
- reopen → "Reopened"

### Phase 6: Update and Add Specs

**File**: `checks/specs/cli/unit/lifecycle.bats`

Update existing test and add new tests:

```bash
@test "batch start with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialBatch Task 1")
    run "$WK_BIN" start "$id1" "unknown-123" "unknown-456"
    assert_failure
    assert_output --partial "Started 1 of 3"
    assert_output --partial "Unknown IDs: unknown-123, unknown-456"

    # Verify the valid issue was still transitioned
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"
}

@test "batch start with mixed unknown and invalid shows both" {
    id1=$(create_issue task "PartialMixed Task 1")
    id2=$(create_issue task "PartialMixed Task 2")
    "$WK_BIN" start "$id1"  # Now in_progress, can't start again

    run "$WK_BIN" start "$id1" "$id2" "unknown-789"
    assert_failure
    assert_output --partial "Started 1 of 3"
    assert_output --partial "Unknown IDs: unknown-789"
    assert_output --partial "$id1: cannot go from in_progress to in_progress"
}

@test "batch done with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialDone Task 1")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" done "$id1" "unknown-123"
    assert_failure
    assert_output --partial "Completed 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch close with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialClose Task 1")
    run "$WK_BIN" close "$id1" "unknown-123" --reason "duplicate"
    assert_failure
    assert_output --partial "Closed 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch reopen with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialReopen Task 1")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" reopen "$id1" "unknown-123"
    assert_failure
    assert_output --partial "Reopened 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch operation all unknown IDs shows all as unknown" {
    run "$WK_BIN" start "unknown-1" "unknown-2" "unknown-3"
    assert_failure
    assert_output --partial "Started 0 of 3"
    assert_output --partial "Unknown IDs: unknown-1, unknown-2, unknown-3"
}

@test "batch operation with single ID preserves current behavior" {
    # Single unknown ID should show simple error, no summary
    run "$WK_BIN" start "unknown-single"
    assert_failure
    assert_output --partial "issue not found: unknown-single"
    refute_output --partial "Started 0 of 1"
}
```

Update existing test to expect new behavior:

```bash
# Change from:
@test "batch start fails if any issue has invalid status" {
    ...
    assert_failure
}

# To:
@test "batch start with invalid status performs partial update" {
    id1=$(create_issue task "LifeBatchFail Task 1")
    id2=$(create_issue task "LifeBatchFail Task 2")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" start "$id1" "$id2"
    assert_failure
    assert_output --partial "Started 1 of 2"
    # id2 should still be transitioned
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: in_progress"
}
```

## Key Implementation Details

### Error Classification

Errors are classified into three categories:
1. **Unknown IDs** - `Error::IssueNotFound` - listed separately
2. **Transition Failures** - `Error::InvalidTransition` - listed with reason
3. **Unexpected Errors** - all other errors - fail fast (database errors, IO, etc.)

### Output Format

```
Started abc-1
Started abc-2
Started 2 of 4 issues
Unknown IDs: xyz-99, xyz-100
  abc-3: cannot go from in_progress to in_progress
```

The format is:
1. Individual operation messages (success only, from `_single` functions)
2. Summary line: `{Action} {N} of {M} issues`
3. Unknown IDs list (if any)
4. Transition failure details (if any, to stderr)

### Exit Code Behavior

- Exit 0: All operations succeeded
- Exit 1: Any operations failed (partial success still returns failure)

This ensures scripts can detect incomplete operations.

### Single ID Behavior

When only a single ID is passed and it fails, preserve the existing simple error message without a summary line. This maintains backward compatibility for common single-issue operations.

## Verification Plan

### Unit Tests

1. `cargo test` - Run existing + new unit tests
2. Verify `BulkResult` methods work correctly
3. Test error classification logic

### Spec Tests

1. `make spec-cli ARGS='--filter "partial"'` - Run new partial bulk specs
2. `make spec-cli ARGS='--file cli/unit/lifecycle.bats'` - Run all lifecycle specs

### Manual Testing

```bash
# Create test issues
wk init
id1=$(wk new task "Test 1" | grep -o '[a-z]*-[0-9]*')
id2=$(wk new task "Test 2" | grep -o '[a-z]*-[0-9]*')

# Test partial start with unknown
wk start "$id1" "unknown-123"
# Expected: Started $id1, summary showing 1 of 2, unknown-123 listed

# Test mixed failures
wk start "$id1" "$id2"  # $id1 is already started
# Expected: Started $id2, summary showing 1 of 2, $id1 listed as invalid transition

# Verify states
wk show "$id1"  # in_progress
wk show "$id2"  # in_progress
```

### Quality Checks

1. `cargo check` - Verify compilation
2. `cargo clippy` - No new warnings
3. `cargo fmt` - Formatting passes
4. `make coverage` - Maintain ≥85% coverage
