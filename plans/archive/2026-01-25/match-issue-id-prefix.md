# Plan: Match Issue ID Prefix

## Overview

Add prefix matching support for issue IDs in wok commands. Users will be able to specify a partial ID (minimum 3 characters) instead of the full ID. For example, `wok edit wok-b18` will match `wok-b18dddb6`. When a prefix matches multiple issues, the command will fail with a clear error listing the ambiguous matches.

**Goals:**
- Enable prefix matching for all commands that accept issue IDs
- Require minimum 3-character prefix for matching
- Handle ambiguous matches gracefully with informative errors
- Preserve exact match behavior (full IDs always work)
- Maintain existing partial failure semantics for bulk operations

## Project Structure

```
crates/cli/src/
├── db/
│   └── issues.rs          # Primary change: prefix matching in get_issue()
├── error.rs               # New error variant: AmbiguousId
├── commands/
│   ├── lifecycle.rs       # Update BulkErrorKind for ambiguous IDs
│   ├── edit.rs            # No changes needed (uses db.get_issue)
│   ├── show.rs            # No changes needed
│   ├── dep.rs             # No changes needed
│   └── ...                # Other commands unchanged
└── id.rs                  # Add prefix validation helper

checks/specs/cli/unit/
├── prefix-matching.bats   # New: prefix matching tests
├── lifecycle.bats         # Update for ambiguous ID tests
├── edit.bats              # Update for prefix matching tests
└── ...
```

## Dependencies

No new external dependencies required. Implementation uses existing:
- `rusqlite` for database queries (already in use)
- Standard library string operations

## Implementation Phases

### Phase 1: Core Prefix Resolution

Add prefix matching logic to the database layer with ambiguity detection.

**Files to modify:**
- `crates/cli/src/db/issues.rs` - Add `resolve_id()` and update `get_issue()`
- `crates/cli/src/error.rs` - Add `AmbiguousId` error variant

**Implementation:**

```rust
// In crates/cli/src/error.rs
pub enum Error {
    // ... existing variants
    AmbiguousId {
        prefix: String,
        matches: Vec<String>,
    },
    // ...
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ...
            Error::AmbiguousId { prefix, matches } => {
                write!(f, "ambiguous issue ID '{}' matches: {}", prefix, matches.join(", "))
            }
        }
    }
}
```

```rust
// In crates/cli/src/db/issues.rs

/// Minimum prefix length for matching
const MIN_PREFIX_LENGTH: usize = 3;

/// Resolve a potentially partial issue ID to a full ID.
/// Returns the full ID if exactly one match is found.
/// Returns an error if no match or multiple matches.
pub fn resolve_id(&self, partial_id: &str) -> Result<String> {
    // First try exact match (fast path)
    if self.issue_exists(partial_id)? {
        return Ok(partial_id.to_string());
    }

    // Check minimum length for prefix matching
    if partial_id.len() < MIN_PREFIX_LENGTH {
        return Err(Error::IssueNotFound(partial_id.to_string()));
    }

    // Find all IDs that start with the prefix
    let pattern = format!("{}%", partial_id);
    let mut stmt = self.conn.prepare(
        "SELECT id FROM issues WHERE id LIKE ?1"
    )?;

    let matches: Vec<String> = stmt
        .query_map([&pattern], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    match matches.len() {
        0 => Err(Error::IssueNotFound(partial_id.to_string())),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(Error::AmbiguousId {
            prefix: partial_id.to_string(),
            matches,
        }),
    }
}

/// Check if an issue exists with exact ID match
fn issue_exists(&self, id: &str) -> Result<bool> {
    let count: i64 = self.conn.query_row(
        "SELECT COUNT(*) FROM issues WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}
```

**Verification:**
- Unit tests for `resolve_id()` with exact, prefix, ambiguous, and not-found cases
- `cargo test` passes

### Phase 2: Integration with Single-ID Commands

Update single-ID commands to use prefix resolution.

**Files to modify:**
- `crates/cli/src/commands/edit.rs`
- `crates/cli/src/commands/show.rs`
- `crates/cli/src/commands/tree.rs`
- `crates/cli/src/commands/link.rs`
- `crates/cli/src/commands/unlink.rs`
- `crates/cli/src/commands/note.rs`
- `crates/cli/src/commands/log.rs`

**Pattern for each command:**

```rust
// Before:
let issue = db.get_issue(&id)?;

// After:
let resolved_id = db.resolve_id(&id)?;
let issue = db.get_issue(&resolved_id)?;
```

**Alternative approach:** Modify `get_issue()` itself to call `resolve_id()` internally. This centralizes the change but requires careful consideration of performance (adds an extra query for exact matches).

**Verification:**
- `wok edit prj-abc value` works when `prj-abcdef12` exists
- `wok show prj-abc` shows the correct issue
- Error message shows "ambiguous issue ID" for ambiguous prefixes

### Phase 3: Integration with Bulk Operations

Update bulk operations to handle ambiguous IDs alongside existing error types.

**Files to modify:**
- `crates/cli/src/commands/lifecycle.rs` - Add `ambiguous_ids` to `BulkResult`
- `crates/cli/src/error.rs` - Update `PartialBulkFailure` if needed

**Implementation:**

```rust
// In crates/cli/src/commands/lifecycle.rs

pub struct BulkResult {
    pub success_count: usize,
    pub unknown_ids: Vec<String>,
    pub ambiguous_ids: Vec<(String, Vec<String>)>, // (prefix, matches)
    pub transition_failures: Vec<(String, String)>,
}

pub enum BulkErrorKind {
    NotFound(String),
    Ambiguous(String, Vec<String>),
    TransitionFailure,
    RequiredFor,
    Fatal,
}

fn run_bulk<F>(db: &Database, ids: &[String], operation: F) -> Result<()>
where
    F: Fn(&Database, &Issue) -> Result<()>,
{
    let mut result = BulkResult::default();

    for id in ids {
        // Resolve prefix first
        match db.resolve_id(id) {
            Ok(resolved_id) => {
                match db.get_issue(&resolved_id) {
                    Ok(issue) => {
                        match operation(db, &issue) {
                            Ok(()) => result.success_count += 1,
                            Err(e) => classify_error(e, &mut result),
                        }
                    }
                    Err(e) => classify_error(e, &mut result),
                }
            }
            Err(Error::AmbiguousId { prefix, matches }) => {
                result.ambiguous_ids.push((prefix, matches));
            }
            Err(Error::IssueNotFound(id)) => {
                result.unknown_ids.push(id);
            }
            Err(e) => return Err(e), // Fatal error
        }
    }

    // Generate summary/error as before, including ambiguous IDs
    // ...
}
```

**Output format for ambiguous IDs:**
```
Started 2 of 4 issues
Unknown IDs: prj-9999
Ambiguous IDs: prj-a (matches: prj-a1b2c3d4, prj-a5b6c7d8)
```

**Verification:**
- Bulk operations succeed for non-ambiguous prefixes
- Ambiguous prefixes are reported in the summary, not as fatal errors
- Non-ambiguous IDs in the same batch still succeed

### Phase 4: Dependency Commands

Update dependency commands which take multiple ID arguments.

**Files to modify:**
- `crates/cli/src/commands/dep.rs`
- `crates/cli/src/commands/undep.rs`

**Note:** These commands have a `from_id` and multiple `to_ids`. The `from_id` should fail immediately on ambiguity (single-ID semantics), while `to_ids` should use bulk semantics.

```rust
// In dep.rs
pub fn add(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let db = Database::open()?;

    // Resolve source ID (fail fast on ambiguity)
    let resolved_from = db.resolve_id(from_id)?;
    let from_issue = db.get_issue(&resolved_from)?;

    // Resolve target IDs with bulk semantics
    let mut result = BulkResult::default();
    for to_id in to_ids {
        match db.resolve_id(to_id) {
            Ok(resolved) => {
                // Add dependency...
            }
            Err(Error::AmbiguousId { prefix, matches }) => {
                result.ambiguous_ids.push((prefix, matches));
            }
            // ...
        }
    }
    // ...
}
```

**Verification:**
- `wok dep prj-abc blocks prj-def` works with prefixes
- Ambiguous `from_id` fails immediately with clear error
- Ambiguous `to_id` reported in bulk summary

### Phase 5: Label Commands

Update label/unlabel commands which operate on multiple IDs.

**Files to modify:**
- `crates/cli/src/commands/label.rs`
- `crates/cli/src/commands/unlabel.rs`

These follow the same bulk pattern as lifecycle commands.

**Verification:**
- `wok label prj-abc prj-def bug` works with prefixes
- Ambiguous IDs reported in summary

### Phase 6: Specification Tests

Add comprehensive specs for prefix matching behavior.

**New file:** `checks/specs/cli/unit/prefix-matching.bats`

```bash
#!/usr/bin/env bats

load ../test_helper

setup() {
    setup_test_repo
    "$WK_BIN" init --prefix test
    "$WK_BIN" new "First issue"   # test-a1b2c3d4
    "$WK_BIN" new "Second issue"  # test-a1b2d5e6 (similar prefix)
    "$WK_BIN" new "Third issue"   # test-b9c8d7e6 (different prefix)
}

teardown() {
    teardown_test_repo
}

@test "exact ID match still works" {
    # Get actual ID from list
    id=$("$WK_BIN" list --format="{id}" | head -1)
    run "$WK_BIN" show "$id"
    assert_success
}

@test "prefix match with minimum 3 characters" {
    run "$WK_BIN" show "test-b9c"
    assert_success
    assert_output --partial "Third issue"
}

@test "prefix shorter than 3 characters fails" {
    run "$WK_BIN" show "te"
    assert_failure
    assert_output --partial "issue not found"
}

@test "ambiguous prefix shows all matches" {
    run "$WK_BIN" show "test-a1b"
    assert_failure
    assert_output --partial "ambiguous issue ID"
    assert_output --partial "test-a1b2c3d4"
    assert_output --partial "test-a1b2d5e6"
}

@test "bulk operation succeeds for unambiguous prefixes" {
    run "$WK_BIN" start "test-b9c"
    assert_success
}

@test "bulk operation reports ambiguous prefix in summary" {
    run "$WK_BIN" start "test-a1b" "test-b9c"
    assert_failure
    assert_output --partial "Started 1 of 2"
    assert_output --partial "Ambiguous IDs"
}

@test "single ID command shows good error for ambiguous prefix" {
    run "$WK_BIN" edit "test-a1b" title "New title"
    assert_failure
    assert_output --partial "ambiguous issue ID 'test-a1b'"
}
```

**Update existing specs:**
- `checks/specs/cli/unit/lifecycle.bats` - Add prefix matching cases
- `checks/specs/cli/unit/edit.bats` - Add prefix matching cases

**Verification:**
- `make spec-cli` passes
- New tests cover all specified behaviors

## Key Implementation Details

### 1. Resolution Strategy

The resolution follows this priority:
1. **Exact match** - Check if the ID exists exactly (fast path, no ambiguity possible)
2. **Prefix match** - If no exact match and length >= 3, search for prefix matches
3. **Ambiguity check** - If multiple prefix matches, return error with all matches

This ensures backward compatibility: existing workflows with full IDs continue working unchanged.

### 2. Minimum Prefix Length

The 3-character minimum serves two purposes:
- **Performance:** Prevents overly broad prefix searches
- **Usability:** Very short prefixes are likely to be ambiguous anyway

With the format `prefix-hash` where prefix is 2+ chars and hash is 8 hex chars, a 3-character minimum allows:
- Matching by project prefix alone (e.g., `wok` matches `wok-*`)
- Matching by partial hash (e.g., `wok-a1b` matches `wok-a1b*`)

### 3. Error Message Format

For single-ID commands:
```
error: ambiguous issue ID 'prj-a1b' matches: prj-a1b2c3d4, prj-a1b5e6f7
```

For bulk commands:
```
Started 3 of 5 issues
Unknown IDs: prj-nonexistent
Ambiguous IDs: prj-a1b (matches: prj-a1b2c3d4, prj-a1b5e6f7)
```

### 4. Database Query Performance

The prefix query uses SQLite's `LIKE` with a trailing wildcard:
```sql
SELECT id FROM issues WHERE id LIKE 'prj-a1b%'
```

This is efficient because:
- SQLite can use an index on `id` for prefix matching with `LIKE 'prefix%'`
- The query returns only IDs, not full issue data
- Typical repositories have hundreds to low thousands of issues

### 5. Case Sensitivity

Issue IDs are case-sensitive (following the existing behavior):
- `WOK-abc` does not match `wok-abcdef12`
- This matches the ID generation which uses lowercase hex

## Verification Plan

### Unit Tests (Rust)

Add to `crates/cli/src/db/issues_tests.rs`:
- `test_resolve_id_exact_match`
- `test_resolve_id_prefix_match`
- `test_resolve_id_ambiguous`
- `test_resolve_id_not_found`
- `test_resolve_id_too_short`

### Integration Tests (Bats)

1. **New spec file:** `checks/specs/cli/unit/prefix-matching.bats`
   - Exact match still works
   - Prefix match with minimum 3 characters
   - Prefix shorter than 3 characters fails
   - Ambiguous prefix shows all matches
   - Bulk operation succeeds for unambiguous prefixes
   - Bulk operation reports ambiguous prefix in summary

2. **Update existing specs:**
   - `lifecycle.bats` - Add cases for prefix matching in start/done/close/reopen
   - `edit.bats` - Add cases for prefix matching
   - `dep.bats` - Add cases for prefix matching in dependencies

### Manual Testing Checklist

- [ ] `wok show prj-abc` matches `prj-abcdef12`
- [ ] `wok edit prj-abc title "New"` updates correct issue
- [ ] `wok start prj-abc prj-def` starts multiple issues by prefix
- [ ] Ambiguous prefix returns clear error message
- [ ] 2-character prefix fails with "not found" (not ambiguous)
- [ ] Full exact IDs continue working unchanged
- [ ] Bulk operations with mixed valid/ambiguous prefixes report partial success

### CI Verification

```bash
make check       # Rust tests, formatting, linting
make spec-cli    # All CLI specs including new prefix tests
```
