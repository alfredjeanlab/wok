# Plan: Add `-p`/`--prefix` Filter to `list`, `ready`, and `search` Commands

## Overview

Add a `-p`/`--prefix` flag to the `wk list`, `wk ready`, and `wk search` commands that filters issues by their ID prefix (the portion before the first hyphen). For example, `-p oj` shows only issues whose IDs start with `oj-`.

## Project Structure

Key files to modify:

```
crates/cli/src/
├── cli/
│   ├── mod.rs              # Command struct definitions (List, Ready, Search)
│   └── args.rs             # Shared argument structs (TypeLabelArgs, etc.)
├── commands/
│   ├── list.rs             # List command implementation
│   ├── ready.rs            # Ready command implementation
│   ├── search.rs           # Search command implementation
│   └── filtering.rs        # Shared filter utilities
├── lib.rs                  # Command dispatch (passes args to run functions)
tests/specs/cli/unit/
├── list.bats               # List command specs
├── ready.bats              # Ready command specs
└── search.bats             # Search command specs
```

## Dependencies

No new external dependencies. The prefix is extracted via simple string splitting (`id.split('-').next()`), which is already used in `crates/cli/src/id.rs`.

## Implementation Phases

### Phase 1: Add shared prefix filter argument and helper

**Files:** `crates/cli/src/cli/args.rs`, `crates/cli/src/commands/filtering.rs`, `crates/cli/src/commands/filtering_tests.rs`

1. Add a `prefix` field to `TypeLabelArgs` in `args.rs`:

```rust
/// Common filter arguments for type, label, and prefix filtering.
#[derive(Args, Clone, Debug, Default)]
pub struct TypeLabelArgs {
    /// Filter by type (comma-separated for OR, repeat for AND)
    #[arg(long, short = 't')]
    pub r#type: Vec<String>,

    /// Filter by label (comma-separated for OR, repeat for AND)
    #[arg(long, short)]
    pub label: Vec<String>,

    /// Filter by ID prefix (e.g., -p oj matches oj-*)
    #[arg(long, short)]
    pub prefix: Option<String>,
}
```

2. Add a `matches_prefix` helper to `filtering.rs`:

```rust
/// Check if an issue ID matches the given prefix filter.
/// The prefix is the portion of the ID before the first hyphen.
pub(crate) fn matches_prefix(prefix: &Option<String>, issue_id: &str) -> bool {
    match prefix {
        None => true,
        Some(p) => issue_id
            .split('-')
            .next()
            .is_some_and(|id_prefix| id_prefix == p),
    }
}
```

3. Add unit tests for `matches_prefix` in `filtering_tests.rs`.

**Milestone:** `cargo test` passes with new helper and tests.

### Phase 2: Wire prefix filter into `list` command

**Files:** `crates/cli/src/cli/mod.rs`, `crates/cli/src/lib.rs`, `crates/cli/src/commands/list.rs`

1. The `List` variant already flattens `TypeLabelArgs`, so the `-p` flag is automatically available.

2. In `lib.rs`, pass `type_label.prefix` through to `commands::list::run()`:
   - Add `prefix: Option<String>` parameter to `run()` and `run_impl()`.

3. In `list.rs`, add prefix filtering right after fetching issues (before other filters, since it's cheap):

```rust
// Filter by prefix
if prefix.is_some() {
    issues.retain(|issue| matches_prefix(&prefix, &issue.id));
}
```

**Milestone:** `wk list -p oj` filters issues by prefix. Verify with manual testing.

### Phase 3: Wire prefix filter into `ready` command

**Files:** `crates/cli/src/lib.rs`, `crates/cli/src/commands/ready.rs`

1. Add `prefix: Option<String>` parameter to `ready::run()` and `ready::run_impl()`.

2. In `lib.rs`, pass `type_label.prefix` through to `commands::ready::run()`.

3. In `ready.rs`, add prefix filtering early (before type/label filters):

```rust
if prefix.is_some() {
    issues.retain(|issue| matches_prefix(&prefix, &issue.id));
}
```

**Milestone:** `wk ready -p oj` filters ready issues by prefix.

### Phase 4: Wire prefix filter into `search` command

**Files:** `crates/cli/src/lib.rs`, `crates/cli/src/commands/search.rs`

1. Add `prefix: Option<String>` parameter to `search::run()` and `search::run_impl()`.

2. In `lib.rs`, pass `type_label.prefix` through to `commands::search::run()`.

3. In `search.rs`, add prefix filtering after search results are fetched:

```rust
if prefix.is_some() {
    issues.retain(|issue| matches_prefix(&prefix, &issue.id));
}
```

**Milestone:** `wk search "query" -p oj` filters search results by prefix.

### Phase 5: Add specs and validate

**Files:** `tests/specs/cli/unit/list.bats`, `tests/specs/cli/unit/ready.bats`, `tests/specs/cli/unit/search.bats`

1. Add a test to `list.bats` that creates issues with different prefixes and verifies `-p` filters correctly:

```bash
@test "list filters by prefix" {
    id1=$(create_issue task "PrefixFilter Alpha task")
    # Create an issue with a different prefix
    id2=$("$WK_BIN" new task "PrefixFilter Beta task" --prefix beta)

    prefix1="${id1%%-*}"  # extract prefix from first issue

    run "$WK_BIN" list -p "$prefix1"
    assert_success
    assert_output --partial "PrefixFilter Alpha task"
    refute_output --partial "PrefixFilter Beta task"

    run "$WK_BIN" list -p beta
    assert_success
    assert_output --partial "PrefixFilter Beta task"
    refute_output --partial "PrefixFilter Alpha task"

    run "$WK_BIN" list --prefix "$prefix1"
    assert_success
    assert_output --partial "PrefixFilter Alpha task"
}
```

2. Add equivalent tests to `ready.bats` and `search.bats`.

3. Run full validation:
   - `make check-fast`
   - `make spec-cli`

**Milestone:** All specs pass, including new prefix filter tests.

## Key Implementation Details

- **Prefix extraction:** Use `issue_id.split('-').next()` for exact match against the prefix string. This is consistent with how `crates/cli/src/id.rs` defines the prefix (everything before the first hyphen).

- **Single-value filter:** Unlike `--type` and `--label` which support comma-separated OR groups, `--prefix` accepts a single string. A workspace typically has a small number of prefixes, and filtering to one at a time is the expected use case.

- **Placement in `TypeLabelArgs`:** The prefix filter is structurally similar to type/label filters (it narrows which issues are shown), so embedding it in `TypeLabelArgs` keeps the arg groups cohesive and avoids adding another `#[command(flatten)]` struct. The struct could be renamed to something more general if desired, but that's optional.

- **Filter ordering:** Apply prefix filtering early in the retain chain since it's a simple string comparison with no DB access, reducing the set before more expensive filters (labels, blocked).

- **No core library changes:** The filter is applied in-memory in the CLI layer after fetching issues, consistent with how all other filters work. No changes to `crates/core/` are needed.

## Verification Plan

1. **Unit tests:** `matches_prefix` helper tested in `filtering_tests.rs`
2. **Spec tests:** BATS specs in `list.bats`, `ready.bats`, `search.bats` covering:
   - Prefix filter shows only matching issues
   - Prefix filter excludes non-matching issues
   - Both `-p` short flag and `--prefix` long flag work
   - Prefix filter combines with other filters (type, label, status)
3. **Validation:** `make check-fast` (fmt, clippy, check, build, test) and `make spec-cli`
