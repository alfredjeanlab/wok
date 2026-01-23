# Plan: debt-5-filter-module

**Root Feature:** `wok-7d1b`

Move filter parsing/matching logic from `commands/list.rs` to a dedicated `commands/filtering.rs` module.

## Overview

The `commands/list.rs` file currently contains filter group parsing and matching functions that are used by multiple commands (`list`, `search`, `ready`). This refactoring extracts these shared functions into a dedicated `commands/filtering.rs` module for better organization and maintainability.

## Project Structure

```
crates/cli/src/commands/
├── mod.rs             # Add: pub mod filtering
├── filtering.rs       # NEW: Filter group parsing/matching
├── filtering_tests.rs # NEW: Tests for filtering module
├── list.rs            # MODIFY: Remove functions, update imports
├── search.rs          # MODIFY: Update imports
└── ready.rs           # MODIFY: Update imports
```

## Dependencies

No new external dependencies required. Uses existing crate types:
- `crate::error::Result`
- `crate::models::Status`, `IssueType`

## Implementation Phases

### Phase 1: Create `filtering.rs` Module

Create new file `crates/cli/src/commands/filtering.rs` with the following functions moved from `list.rs`:

```rust
// crates/cli/src/commands/filtering.rs

/// Parse filter values: comma-separated values within each Vec entry are OR'd,
/// multiple Vec entries are AND'd together.
pub fn parse_filter_groups<T, F>(values: &[String], parse_fn: F) -> Result<Option<Vec<Vec<T>>>>
where
    F: Fn(&str) -> Result<T>;

/// Check if an issue matches the filter groups.
/// Each group is OR'd internally, all groups must match (AND).
pub fn matches_filter_groups<T, F>(groups: &Option<Vec<Vec<T>>>, get_value: F) -> bool
where
    T: PartialEq,
    F: Fn() -> T;

/// Check if an issue matches label filter groups.
/// Each group is OR'd internally (issue has at least one label from group),
/// all groups must match (AND).
pub fn matches_label_groups(groups: &Option<Vec<Vec<String>>>, issue_labels: &[String]) -> bool;
```

**Milestone:** New file compiles with `cargo check`.

### Phase 2: Create `filtering_tests.rs`

Move filter group tests from `list_tests.rs` to new `filtering_tests.rs`:

Tests to move:
- `test_parse_filter_groups_empty`
- `test_parse_filter_groups_single_value`
- `test_parse_filter_groups_comma_separated`
- `test_parse_filter_groups_multiple_entries`
- `test_matches_filter_groups_none`
- `test_matches_filter_groups_single_match`
- `test_matches_filter_groups_and_logic`
- `test_matches_label_groups_none`
- `test_matches_label_groups_single_group`
- `test_matches_label_groups_and_logic`

**Milestone:** `cargo test --lib filtering` passes.

### Phase 3: Update `commands/mod.rs`

Add the new module to `commands/mod.rs`:

```rust
pub mod filtering;
```

**Milestone:** Module is exported and accessible.

### Phase 4: Update Consumer Imports

Update imports in each consumer file:

**`list.rs`:**
```rust
// Remove function definitions, add import:
use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
```

**`search.rs`:**
```rust
// Change from:
use super::list::{matches_filter_groups, matches_label_groups, parse_filter_groups};
// To:
use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
```

**`ready.rs`:**
```rust
// Change from:
use super::list::{matches_filter_groups, matches_label_groups, parse_filter_groups};
// To:
use super::filtering::{matches_filter_groups, matches_label_groups, parse_filter_groups};
```

**Milestone:** All imports updated, `cargo check` passes.

### Phase 5: Cleanup and Verification

1. Remove moved tests from `list_tests.rs`
2. Run full test suite: `cargo test`
3. Run clippy: `cargo clippy`
4. Run formatter: `cargo fmt`

**Milestone:** All checks pass, no dead code warnings.

## Key Implementation Details

### Filter Group Semantics

The filter group functions implement a specific AND/OR logic pattern used across CLI arguments:

- **Comma-separated values** within a single `--status` or `--label` argument are **OR'd**
  - `--status todo,in_progress` → matches Todo OR InProgress

- **Multiple flag occurrences** are **AND'd**
  - `--label a --label b` → must have label "a" AND label "b"

### Label vs Value Matching

Two different matching functions exist because:
- `matches_filter_groups`: For single-valued fields (status, type) - issue has exactly one value
- `matches_label_groups`: For multi-valued fields (labels) - issue can have multiple labels

### Visibility

Functions should be `pub(crate)` since they're internal to the CLI crate but shared across command modules.

## Verification Plan

1. **Unit Tests:**
   - `cargo test filtering` - Run new filtering module tests
   - `cargo test list` - Verify list command still works
   - `cargo test search` - Verify search command still works
   - `cargo test ready` - Verify ready command still works

2. **Integration Tests:**
   - `make spec-cli ARGS='--filter "list"'` - CLI list specs
   - `make spec-cli ARGS='--filter "search"'` - CLI search specs
   - `make spec-cli ARGS='--filter "ready"'` - CLI ready specs

3. **Quality Checks:**
   - `cargo clippy` - No new warnings
   - `cargo fmt --check` - Code formatted
   - `cargo check` - No compile errors
