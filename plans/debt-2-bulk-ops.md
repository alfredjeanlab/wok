# Plan: Extract Generic Bulk Operation Helper

**Root Feature:** `wok-8e48`

## Overview

Extract a reusable `bulk_operation()` helper function from `lifecycle.rs` to eliminate the duplicate error handling loops in `start_impl`, `done_impl`, `close_impl`, and `reopen_impl`. Each function currently repeats ~40 lines of identical pattern: iterate IDs, call single-item function, collect failures into `BulkResult`, handle single-ID backward compatibility, print summary, and return appropriate error.

## Current State

### The Duplicate Pattern (repeated 4 times)

Each `*_impl` function contains this structure:

```rust
pub(crate) fn start_impl(db: &Database, config: &Config, work_dir: &Path, ids: &[String]) -> Result<()> {
    let mut result = BulkResult::default();
    let mut last_error: Option<Error> = None;

    for id in ids {
        match start_single(db, config, work_dir, id) {
            Ok(()) => result.success_count += 1,
            Err(Error::IssueNotFound(ref unknown_id)) => {
                last_error = Some(Error::IssueNotFound(unknown_id.clone()));
                result.unknown_ids.push(unknown_id.clone());
            }
            Err(Error::InvalidTransition { ref from, ref to, ref valid_targets }) => {
                last_error = Some(Error::InvalidTransition { ... });
                let msg = format!("cannot go from {} to {}", from, to);
                result.transition_failures.push((id.clone(), msg));
            }
            // done_impl and reopen_impl also handle:
            // Err(Error::InvalidInput(ref msg)) if msg.contains("required for agent") => { ... }
            Err(e) => return Err(e),  // Fail fast for unexpected errors
        }
    }

    // For single ID, return original error for backward compatibility
    if ids.len() == 1 {
        if result.is_success() {
            return Ok(());
        }
        return Err(last_error.unwrap_or_else(|| ...));
    }

    print_bulk_summary(&result, "started");  // verb varies

    if result.is_success() {
        Ok(())
    } else {
        Err(Error::PartialBulkFailure { ... })
    }
}
```

### Differences Between Commands

| Command | Action Verb | Handles `InvalidInput` |
|---------|-------------|------------------------|
| `start_impl` | "started" | No |
| `done_impl` | "completed" | Yes (agent reason required) |
| `close_impl` | "closed" | No |
| `reopen_impl` | "reopened" | Yes (agent reason required) |

## Project Structure

```
crates/cli/src/commands/
├── lifecycle.rs        # Contains all 4 duplicate patterns (modify)
└── lifecycle_tests.rs  # Unit tests (may need additions)
```

## Dependencies

None. This is a pure refactoring that uses existing types and closures.

## Implementation Phases

### Phase 1: Define Error Classification Enum

**Goal**: Create an enum to classify how each error should be handled by the bulk operation helper.

**File**: `crates/cli/src/commands/lifecycle.rs`

**Changes**: Add near `BulkResult`:

```rust
/// How to handle an error in bulk operations
enum BulkErrorKind {
    /// ID was not found - add to unknown_ids
    NotFound(String),
    /// Invalid transition - add to transition_failures with message
    TransitionFailure(String),
    /// Unexpected error - fail fast
    Fatal(Error),
}

impl BulkErrorKind {
    /// Classify an error for bulk operation handling
    fn classify(error: &Error, id: &str) -> Self {
        match error {
            Error::IssueNotFound(unknown_id) => {
                BulkErrorKind::NotFound(unknown_id.clone())
            }
            Error::InvalidTransition { from, to, .. } => {
                let msg = format!("cannot go from {} to {}", from, to);
                BulkErrorKind::TransitionFailure(msg)
            }
            Error::InvalidInput(msg) if msg.contains("required for agent") => {
                BulkErrorKind::TransitionFailure(msg.clone())
            }
            _ => BulkErrorKind::Fatal(error.clone()),
        }
    }
}
```

**Note**: This requires `Error` to implement `Clone`. Check if it already does; if not, add `#[derive(Clone)]` in Phase 1.

**Verification**: `cargo check`

### Phase 2: Add `bulk_operation()` Helper Function

**Goal**: Create the generic helper that encapsulates the common pattern.

**File**: `crates/cli/src/commands/lifecycle.rs`

**Changes**: Add after `print_bulk_summary`:

```rust
/// Execute a bulk operation on multiple IDs with consistent error handling.
///
/// - `ids`: The issue IDs to process
/// - `action_verb`: Past tense verb for summary (e.g., "started", "completed")
/// - `operation`: Closure that performs the single-item operation
fn bulk_operation<F>(ids: &[String], action_verb: &str, operation: F) -> Result<()>
where
    F: Fn(&str) -> Result<()>,
{
    let mut result = BulkResult::default();
    let mut last_error: Option<Error> = None;

    for id in ids {
        match operation(id) {
            Ok(()) => result.success_count += 1,
            Err(ref e) => {
                match BulkErrorKind::classify(e, id) {
                    BulkErrorKind::NotFound(unknown_id) => {
                        last_error = Some(Error::IssueNotFound(unknown_id.clone()));
                        result.unknown_ids.push(unknown_id);
                    }
                    BulkErrorKind::TransitionFailure(msg) => {
                        last_error = Some(e.clone());
                        result.transition_failures.push((id.clone(), msg));
                    }
                    BulkErrorKind::Fatal(fatal_error) => {
                        return Err(fatal_error);
                    }
                }
            }
        }
    }

    // For single ID, return original error for backward compatibility
    if ids.len() == 1 {
        if result.is_success() {
            return Ok(());
        }
        return Err(last_error
            .unwrap_or_else(|| Error::InvalidInput("internal error: expected error".to_string())));
    }

    print_bulk_summary(&result, action_verb);

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

**Verification**: `cargo check` (function defined but not yet used)

### Phase 3: Refactor `start_impl`

**Goal**: Replace the loop in `start_impl` with `bulk_operation()`.

**Before** (~50 lines):
```rust
pub(crate) fn start_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
) -> Result<()> {
    let mut result = BulkResult::default();
    let mut last_error: Option<Error> = None;

    for id in ids {
        match start_single(db, config, work_dir, id) {
            // ... 30+ lines of error handling
        }
    }
    // ... 15+ lines of summary/return logic
}
```

**After** (~5 lines):
```rust
pub(crate) fn start_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
) -> Result<()> {
    bulk_operation(ids, "started", |id| start_single(db, config, work_dir, id))
}
```

**Verification**: `cargo test` - run lifecycle tests

### Phase 4: Refactor `done_impl`, `close_impl`, `reopen_impl`

**Goal**: Apply the same refactoring to the remaining three functions.

**Changes**:

```rust
pub(crate) fn done_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: Option<&str>,
) -> Result<()> {
    bulk_operation(ids, "completed", |id| done_single(db, config, work_dir, id, reason))
}

pub(crate) fn close_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: &str,
) -> Result<()> {
    bulk_operation(ids, "closed", |id| close_single(db, config, work_dir, id, reason))
}

pub(crate) fn reopen_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    ids: &[String],
    reason: Option<&str>,
) -> Result<()> {
    bulk_operation(ids, "reopened", |id| reopen_single(db, config, work_dir, id, reason))
}
```

**Verification**: `cargo test`, `make spec-cli`

### Phase 5: Clean Up and Final Verification

**Goal**: Remove any dead code, ensure tests pass, run full validation.

**Tasks**:
1. Remove unused imports if any
2. Run `cargo clippy` to check for warnings
3. Run `cargo fmt` to ensure formatting
4. Run `make spec-cli` to verify CLI behavior unchanged
5. Run `make validate` for full validation

**Verification**: All checks pass

## Key Implementation Details

### Error Clone Requirement

The `bulk_operation()` helper needs to clone errors in some cases (to store in `last_error` while also using the classified version). Check if `Error` already derives `Clone`:

```rust
// In crates/cli/src/error.rs
#[derive(Debug, Clone)]  // Clone may need to be added
pub enum Error {
    // ...
}
```

If `Error` contains non-cloneable types, an alternative is to restructure to avoid cloning:

```rust
// Alternative: store the ID and recreate the error
let mut last_error_id: Option<String> = None;
// ...
if let BulkErrorKind::NotFound(unknown_id) = kind {
    last_error_id = Some(id.clone());
    // ...
}
// When returning for single ID:
return Err(Error::IssueNotFound(last_error_id.unwrap()));
```

### Closure Captures

The closures capture references to `db`, `config`, `work_dir`, and optionally `reason`. This works because:
- The closure borrows these for the duration of `bulk_operation()`
- Rust infers `Fn(&str) -> Result<()>` (not `FnMut` or `FnOnce`)
- No ownership issues since we only borrow

### Why Not Generic Over Error Types

The current design classifies errors inside `BulkErrorKind::classify()` using pattern matching. This keeps the `bulk_operation()` signature simple and avoids needing the caller to specify error handling behavior.

## Verification Plan

1. **After Phase 1**: `cargo check` passes
2. **After Phase 2**: `cargo check` passes (helper compiles)
3. **After Phase 3**: `cargo test` - lifecycle tests pass
4. **After Phase 4**: `cargo test`, `make spec-cli` pass
5. **After Phase 5**: `make validate` passes

## Summary Statistics

- **Lines removed**: ~160 (4 functions × ~40 lines of duplicate code each)
- **Lines added**: ~50 (helper function + enum)
- **Net reduction**: ~110 lines
- **Files modified**: 2 (`lifecycle.rs`, possibly `error.rs` for Clone)
