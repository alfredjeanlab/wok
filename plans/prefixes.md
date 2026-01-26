# Implementation Plan: Multiple Prefix Support

## Overview

Enhance wok's prefix system to support multiple prefixes within a single database, enabling multi-project and fleet-mode collaboration. This includes:

1. Adding `--prefix` flag to `wk new` for creating issues with different prefixes
2. Adding `wk config prefixes` command to list all prefixes in the system
3. Creating a `prefixes` table that auto-tracks all prefixes
4. Fixing `wk config rename` help text for clarity

## Project Structure

```
crates/cli/src/
├── cli.rs                    # Add --prefix to New, add Prefixes subcommand to ConfigCommand
├── commands/
│   ├── new.rs                # Handle --prefix flag
│   └── config.rs             # Add prefixes listing, fix rename help text
├── db/
│   ├── schema.sql            # Add prefixes table
│   └── mod.rs                # Add prefix tracking methods
└── id.rs                     # (no changes needed)

tests/specs/cli/unit/
├── new.bats                  # Add tests for --prefix flag
├── config.bats               # Add tests for prefixes command
└── prefix-tracking.bats      # New: comprehensive prefix table tests

docs/specs/
└── 06-storage-config.md      # Document prefixes table
```

## Dependencies

No new external dependencies required. Uses existing:
- `rusqlite` for database operations
- `clap` for CLI parsing

## Implementation Phases

### Phase 1: Database Schema - Add Prefixes Table

Add a `prefixes` table to track all prefixes in the system.

**Files to modify:**
- `crates/cli/src/db/schema.sql`
- `crates/cli/src/db/mod.rs`

**Schema addition:**

```sql
-- Prefix registry (auto-populated)
CREATE TABLE IF NOT EXISTS prefixes (
    prefix TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    issue_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_prefixes_count ON prefixes(issue_count DESC);
```

**Database methods to add:**

```rust
// crates/cli/src/db/mod.rs

/// Ensure a prefix exists in the prefixes table
pub fn ensure_prefix(&self, prefix: &str) -> Result<()>;

/// Increment issue count for a prefix
pub fn increment_prefix_count(&self, prefix: &str) -> Result<()>;

/// Decrement issue count for a prefix (for delete operations)
pub fn decrement_prefix_count(&self, prefix: &str) -> Result<()>;

/// List all prefixes with their issue counts
pub fn list_prefixes(&self) -> Result<Vec<PrefixInfo>>;

/// Rename a prefix in the prefixes table
pub fn rename_prefix(&self, old: &str, new: &str) -> Result<()>;
```

**Migration strategy:**
- On database open, run migration that populates `prefixes` table from existing issues
- Extract prefix by splitting issue ID on first `-`

**Verification:**
- Unit tests in `db_tests.rs`
- Spec tests in `prefix-tracking.bats`

---

### Phase 2: Add `--prefix` Flag to `wk new`

Allow creating issues with a prefix different from the config default.

**Files to modify:**
- `crates/cli/src/cli.rs`
- `crates/cli/src/commands/new.rs`

**CLI changes:**

```rust
// crates/cli/src/cli.rs - Add to New command
/// Create issue with specific prefix (overrides config prefix)
#[arg(long, short = 'p')]
prefix: Option<String>,
```

**Implementation in `new.rs`:**

```rust
// Determine which prefix to use
let effective_prefix = match prefix {
    Some(p) => {
        // Validate the provided prefix
        if !validate_prefix(&p) {
            return Err(Error::InvalidPrefix);
        }
        p
    }
    None => {
        // Use config prefix (existing behavior)
        if config.prefix.is_empty() {
            return Err(Error::CannotCreateIssue { ... });
        }
        config.prefix.clone()
    }
};

// Use effective_prefix in generate_unique_id
let id = generate_unique_id(&effective_prefix, title, &created_at, |id| {
    db.issue_exists(id).unwrap_or(false)
});

// Track the prefix
db.ensure_prefix(&effective_prefix)?;
db.increment_prefix_count(&effective_prefix)?;
```

**Verification:**
- `wk new "Test" --prefix other` creates `other-XXXX`
- `wk new "Test" -p other` short form works
- Invalid prefix rejected
- Works with all issue types and flags

---

### Phase 3: Add `wk config prefixes` Command

Add command to list all prefixes in the system.

**Files to modify:**
- `crates/cli/src/cli.rs`
- `crates/cli/src/commands/config.rs`

**CLI addition:**

```rust
// crates/cli/src/cli.rs - Add to ConfigCommand enum
/// List all prefixes in the issue tracker
#[command(after_help = colors::examples("\
Examples:
  wok config prefixes              List all prefixes with issue counts
  wok config prefixes -o json      Output as JSON"))]
Prefixes {
    /// Output format
    #[arg(long, short, default_value = "text")]
    output: OutputFormat,
},
```

**Implementation:**

```rust
// crates/cli/src/commands/config.rs

fn run_list_prefixes(db: &Database, config: &Config, output: OutputFormat) -> Result<()> {
    let prefixes = db.list_prefixes()?;

    match output {
        OutputFormat::Text => {
            if prefixes.is_empty() {
                println!("No prefixes found.");
                return Ok(());
            }

            // Show current/default prefix with marker
            for p in &prefixes {
                let marker = if p.prefix == config.prefix { " (default)" } else { "" };
                println!("{}: {} issues{}", p.prefix, p.issue_count, marker);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "default": config.prefix,
                "prefixes": prefixes.iter().map(|p| {
                    serde_json::json!({
                        "prefix": p.prefix,
                        "issue_count": p.issue_count,
                        "is_default": p.prefix == config.prefix
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Id => {
            // Just list prefix names
            for p in &prefixes {
                println!("{}", p.prefix);
            }
        }
    }
    Ok(())
}
```

**Verification:**
- `wk config prefixes` lists all prefixes
- Shows issue count per prefix
- Marks default prefix
- JSON output works

---

### Phase 4: Update `config rename` for Prefix Table

Ensure `config rename` updates the prefixes table.

**Files to modify:**
- `crates/cli/src/commands/config.rs`

**Changes to `rename_all_issue_ids`:**

```rust
fn rename_all_issue_ids(db: &Database, old_prefix: &str, new_prefix: &str) -> Result<()> {
    // ... existing code ...

    // After updating issues, update prefixes table
    db.rename_prefix(old_prefix, new_prefix)?;

    // ... rest of function ...
}
```

**Verification:**
- After rename, `config prefixes` shows new prefix, not old
- Issue counts preserved

---

### Phase 5: Fix `config rename` Help Text

Clarify that rename operates on a specific prefix, not "the" prefix.

**Files to modify:**
- `crates/cli/src/cli.rs`

**Current (unclear):**
```rust
/// Rename the issue ID prefix (updates config and all existing issues)
```

**Updated (clear):**
```rust
/// Rename a prefix, updating all issues with that prefix
///
/// Renames issues from `old-XXXX` to `new-XXXX`. Updates the config file
/// only if `old` is the current default prefix.
```

**Example updates:**
```rust
#[command(
    arg_required_else_help = true,
    after_help = colors::examples("\
Examples:
  wok config rename old new        Rename 'old-*' issues to 'new-*'
  wok config rename proj app       Rename 'proj' prefix to 'app'")
)]
Rename {
    /// The prefix to rename from (e.g., 'old' renames 'old-XXXX' issues)
    old_prefix: String,

    /// The prefix to rename to (e.g., 'new' creates 'new-XXXX' IDs)
    new_prefix: String,
},
```

**Verification:**
- `wk config rename --help` shows clear description
- Examples demonstrate multi-prefix scenarios

---

### Phase 6: Backfill Migration & Integration Tests

Ensure existing databases get prefix tracking.

**Files to modify/create:**
- `crates/cli/src/db/mod.rs` - Add migration
- `tests/specs/cli/unit/prefix-tracking.bats` - New test file

**Migration on open:**

```rust
// In Database::open or ensure_schema
fn migrate_prefixes_table(conn: &Connection) -> Result<()> {
    // Check if migration needed (table empty but issues exist)
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM prefixes", [], |row| row.get(0)
    ).unwrap_or(0);

    if count == 0 {
        // Backfill from existing issues
        conn.execute(
            "INSERT OR IGNORE INTO prefixes (prefix, created_at, issue_count)
             SELECT
                 substr(id, 1, instr(id, '-') - 1) as prefix,
                 MIN(created_at) as created_at,
                 COUNT(*) as issue_count
             FROM issues
             WHERE id LIKE '%-%'
             GROUP BY prefix",
            [],
        )?;
    }
    Ok(())
}
```

**New spec file `prefix-tracking.bats`:**

```bash
@test "prefixes table auto-populated on init" {
    mkdir -p prefix_init && cd prefix_init
    run "$WK_BIN" init --prefix test
    assert_success

    create_issue task "First issue"

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "test: 1 issue"
}

@test "new --prefix creates issue with different prefix" {
    mkdir -p prefix_new && cd prefix_new
    run "$WK_BIN" init --prefix main
    assert_success

    id=$(run "$WK_BIN" new "Other project task" --prefix other -o id)
    [[ "$id" == other-* ]]

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "other: 1 issue"
}

@test "config prefixes shows all prefixes with counts" {
    mkdir -p prefix_list && cd prefix_list
    run "$WK_BIN" init --prefix proj
    assert_success

    create_issue task "Proj task 1"
    create_issue task "Proj task 2"
    "$WK_BIN" new "Other task" --prefix other

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "proj: 2 issues"
    assert_output --partial "other: 1 issue"
    assert_output --partial "(default)"
}

@test "config rename updates prefixes table" {
    mkdir -p prefix_rename && cd prefix_rename
    run "$WK_BIN" init --prefix old
    assert_success

    create_issue task "Test"

    run "$WK_BIN" config rename old new
    assert_success

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "new: 1 issue"
    refute_output --partial "old:"
}
```

**Verification:**
- Fresh init creates prefix entry
- Existing databases get backfilled
- All prefix operations maintain consistency

## Key Implementation Details

### Prefix Extraction

Extract prefix from issue ID using first `-` as delimiter:

```rust
fn extract_prefix(issue_id: &str) -> Option<&str> {
    issue_id.split('-').next()
}
```

### Thread Safety

The prefix table uses atomic operations:
- `INSERT OR IGNORE` for ensuring prefix exists
- `UPDATE ... SET issue_count = issue_count + 1` for incrementing

### Operation Sync

When adding remote sync support for prefixes:
- `OpPayload::ConfigRename` already syncs prefix renames
- Consider adding `OpPayload::CreateIssue` to include prefix for fleet-wide prefix tracking

### Multi-Project Considerations

- Each project can use `--prefix` to create issues with project-specific prefixes
- Workspace links can override the default prefix in their config
- `config prefixes` shows all prefixes across the shared database

## Verification Plan

### Unit Tests (Rust)

```bash
cargo test -p wk prefix
cargo test -p wk config
```

### Spec Tests (BATS)

```bash
make spec-cli ARGS='--filter "prefix"'
make spec-cli ARGS='--filter "config"'
```

### Manual Testing

```bash
# Initialize fresh tracker
wk init --prefix main

# Create issues with different prefixes
wk new "Main project task"
wk new "Backend task" --prefix api
wk new "Frontend task" --prefix web

# List all prefixes
wk config prefixes
# Output:
# main: 1 issue (default)
# api: 1 issue
# web: 1 issue

# JSON output
wk config prefixes -o json

# Rename a prefix
wk config rename api backend
wk config prefixes
# Output:
# main: 1 issue (default)
# backend: 1 issue
# web: 1 issue
```

### Checklist

- [ ] `cargo check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] `make spec-cli` passes
- [ ] `cargo fmt --check` passes
- [ ] Help text is clear and accurate
- [ ] JSON output includes all fields
- [ ] Existing databases migrate correctly
