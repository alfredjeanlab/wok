# Implementation Plan: Add Dependency Flags to `wok new`

## Overview

Add `--blocks`, `--blocked-by`, `--tracks`, and `--tracked-by` flags to the `wok new` command, allowing users to create issues with dependencies in a single transaction. This reduces friction when creating related issues and improves workflow efficiency. Also update the prime output with practical examples.

## Project Structure

Key files to modify:

```
crates/cli/src/
├── cli.rs                      # Add new CLI arguments to New command
├── commands/
│   ├── new.rs                  # Process dependency flags after issue creation
│   ├── new_tests.rs            # Add tests for new functionality
│   ├── dep.rs                  # Reference for dependency logic (no changes)
│   └── prime.md                # Add examples using new flags
└── models/
    └── dependency.rs           # Reference for UserRelation enum (no changes)
```

## Dependencies

No new external dependencies required. Uses existing:
- `clap` for CLI argument parsing
- `wk_core` for `Relation` enum
- Existing `db::add_dependency()` function

## Implementation Phases

### Phase 1: Add CLI Arguments

**File**: `crates/cli/src/cli.rs`

Add four new `Vec<String>` arguments to the `New` command variant:

```rust
New {
    // ... existing fields ...

    /// Issues this new issue blocks (comma-separated or repeated)
    #[arg(long, value_name = "IDS")]
    blocks: Vec<String>,

    /// Issues that block this new issue (comma-separated or repeated)
    #[arg(long, value_name = "IDS")]
    blocked_by: Vec<String>,

    /// Issues this new issue tracks/contains (comma-separated or repeated)
    #[arg(long, value_name = "IDS")]
    tracks: Vec<String>,

    /// Issues that track this new issue (comma-separated or repeated)
    #[arg(long, value_name = "IDS")]
    tracked_by: Vec<String>,
}
```

Update the match arm for `Commands::New` to pass these to `new::run()`.

**Verification**: `cargo check` passes, `wok new --help` shows new flags.

---

### Phase 2: Expand IDs Helper Function

**File**: `crates/cli/src/commands/new.rs`

Add a helper function to expand comma-separated IDs (similar to `expand_labels()`):

```rust
fn expand_ids(ids: &[String]) -> Vec<String> {
    ids.iter()
        .flat_map(|id| id.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

**Verification**: Unit test that `expand_ids(&["a,b", "c"])` returns `["a", "b", "c"]`.

---

### Phase 3: Update `new::run()` Signature and Implementation

**File**: `crates/cli/src/commands/new.rs`

1. Update function signature to accept dependency arguments:

```rust
pub fn run(
    type_or_title: String,
    title: Option<String>,
    labels: Vec<String>,
    note: Option<String>,
    links: Vec<String>,
    assignee: Option<String>,
    priority: Option<u8>,
    description: Option<String>,
    blocks: Vec<String>,
    blocked_by: Vec<String>,
    tracks: Vec<String>,
    tracked_by: Vec<String>,
) -> Result<()>
```

2. After issue creation, process dependencies using the existing `dep` module logic:

```rust
// After successful issue creation, add dependencies
let new_id = &issue.id;

// Process each relation type
for target_id in expand_ids(&blocks) {
    dep::add_impl(&db, &work_dir, &config, new_id, UserRelation::Blocks, &target_id)?;
}

for target_id in expand_ids(&blocked_by) {
    dep::add_impl(&db, &work_dir, &config, new_id, UserRelation::BlockedBy, &target_id)?;
}

for target_id in expand_ids(&tracks) {
    dep::add_impl(&db, &work_dir, &config, new_id, UserRelation::Tracks, &target_id)?;
}

for target_id in expand_ids(&tracked_by) {
    dep::add_impl(&db, &work_dir, &config, new_id, UserRelation::TrackedBy, &target_id)?;
}
```

**Note**: The `add_impl` function in `dep.rs` is currently private. Either:
- Extract the core logic into a shared helper, or
- Refactor to expose a public API for adding dependencies programmatically

**Verification**: `cargo check` passes, manual test of `wok new task "Test" --blocks abc-1`.

---

### Phase 4: Refactor `dep::add_impl` for Reuse

**File**: `crates/cli/src/commands/dep.rs`

Currently `add_impl` requires `to_ids: &[String]`. Create a public helper that can be called from `new.rs`:

```rust
/// Add a single dependency relationship between two issues.
/// Called by both `wok dep` and `wok new --blocks/--tracks/etc`.
pub fn add_dependency_relation(
    db: &Database,
    work_dir: &Path,
    config: &Config,
    from_id: &str,
    relation: UserRelation,
    to_id: &str,
) -> Result<()> {
    // Existing logic from add_impl, but for single target
}
```

This avoids duplicating the complex bidirectional handling logic (tracks creates two deps).

**Verification**: Existing `wok dep` tests still pass.

---

### Phase 5: Update Prime Output

**File**: `crates/cli/src/commands/prime.md`

Update the "Creating & Updating" section to show the new flags:

```markdown
## Creating & Updating
- `wok new [type] "title" [--note "description"] [--label label,...]` - New issue
  - Types: task (default), bug, feature
  - Priority: `--label priority:0` through `--label priority:4` (0=critical, 2=medium, 4=backlog)
  - Multiple labels: `--label a,b,c` or `--label a --label b`
  - Dependencies: `--blocks`, `--blocked-by`, `--tracks`, `--tracked-by`
```

Update the "Common Workflows" section with examples:

```markdown
**Creating a bug that blocks another issue:**
```bash
wok new bug "Fix auth token expiry" --blocks prj-42
```

**Creating a feature with subtasks inline:**
```bash
wok new feature "User authentication" --tracks prj-task-1,prj-task-2
```

**Creating a task tracked by a feature:**
```bash
wok new "Implement login endpoint" --tracked-by prj-feat-1
```
```

**Verification**: `wok prime` output includes new examples.

---

### Phase 6: Add Tests

**File**: `crates/cli/src/commands/new_tests.rs`

Add tests for the new dependency functionality:

```rust
#[test]
fn new_with_blocks_creates_dependency() {
    let ctx = TestContext::new();
    // Create target issue first
    new::run("task".into(), Some("Target".into()), ..., vec![], vec![], vec![], vec![]).unwrap();
    let target = ctx.db.list_issues(...).first().unwrap();

    // Create new issue that blocks it
    new::run("bug".into(), Some("Blocker".into()), ...,
             vec![target.id.clone()], vec![], vec![], vec![]).unwrap();

    // Verify dependency exists
    let deps = ctx.db.get_deps_from(&blocker_id);
    assert!(deps.iter().any(|d| d.to_id == target.id && d.relation == Relation::Blocks));
}

#[test]
fn new_with_comma_separated_ids() {
    // Test that "a,b,c" expands correctly
}

#[test]
fn new_with_tracks_creates_bidirectional() {
    // Test that --tracks creates both directions
}

#[test]
fn new_with_invalid_target_fails() {
    // Test error handling for non-existent target
}
```

**Verification**: `cargo test` passes, all new tests green.

## Key Implementation Details

### Comma-Separated ID Expansion

Follow the same pattern as `expand_labels()`:
- Split on commas
- Trim whitespace
- Filter empty strings
- Combine with repeated flags

Example: `--blocks a,b --blocks c` → `["a", "b", "c"]`

### Bidirectional Tracks Relationship

The `tracks` relationship creates two database entries:
1. `new_issue tracks target`
2. `target tracked_by new_issue`

This is already handled in `dep.rs` lines 76-107. Reuse that logic.

### Error Handling

Validate target issues exist before creating dependencies. If any target doesn't exist, fail early with a clear error message. This matches the existing `wok dep` behavior.

### Transaction Semantics

All operations (issue creation + dependencies) happen in sequence. If dependency creation fails:
- The issue has already been created (this is acceptable)
- User sees error message with which dependency failed
- User can manually add remaining dependencies with `wok dep`

## Verification Plan

1. **Unit tests**: `cargo test` - all new tests pass
2. **Manual testing**:
   ```bash
   # Setup
   wok new task "Target 1"
   wok new task "Target 2"

   # Test single --blocks
   wok new bug "Fix bug" --blocks prj-1
   wok show prj-3  # Should show "blocks: prj-1"

   # Test comma-separated
   wok new feature "Epic" --tracks prj-1,prj-2
   wok tree prj-4  # Should show both tracked issues

   # Test repeated flags
   wok new task "Task" --blocked-by prj-1 --blocked-by prj-2
   wok show prj-5  # Should show both blockers

   # Test error case
   wok new task "Bad" --blocks nonexistent  # Should error
   ```

3. **Integration specs**: Add bats tests to `checks/specs/cli/unit/new.bats` if not already covered
4. **Lint and format**: `cargo clippy && cargo fmt --check`
5. **Full validation**: `make check`
