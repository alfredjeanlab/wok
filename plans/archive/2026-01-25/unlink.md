# Plan: Unlink Command Implementation

## Overview

Add an `unlink` command to remove external links from issues. This follows the existing pattern of paired add/remove commands (`label`/`unlabel`, `dep`/`undep`) and uses the existing `Action::Unlinked` event type which is already defined in the codebase.

## Project Structure

```
crates/cli/src/cli.rs             # Add Unlink command variant
crates/cli/src/lib.rs             # Add command dispatch
crates/cli/src/commands/link.rs   # Add remove() and remove_impl() functions
crates/cli/src/commands/link_tests.rs  # Add unit tests for remove
checks/specs/cli/unit/link.bats   # Add spec tests for unlink
docs/specs/04-cli-interface.md    # Update documentation
```

## Dependencies

No new dependencies. Uses existing infrastructure:
- `db.get_links()` - retrieve links for an issue
- `db.remove_link()` - remove link by database ID
- `Action::Unlinked` - already defined in `models/event.rs`
- `apply_mutation()` - existing helper for event logging

## Implementation Phases

### Phase 1: Add CLI Command Definition

**File:** `crates/cli/src/cli.rs` (after line 396, before `Dep`)

Add the `Unlink` command variant:

```rust
/// Remove an external link from an issue
#[command(
    arg_required_else_help = true,
    after_help = colors::examples("\
Examples:
  wok unlink prj-a3f2 https://github.com/org/repo/issues/123
  wok unlink prj-a3f2 jira://PE-5555")
)]
Unlink {
    /// Issue ID
    id: String,
    /// External URL to remove (must match exactly)
    url: String,
},
```

**Verification:**
```bash
cargo check
cargo build --release
./target/release/wk unlink --help
```

### Phase 2: Add Command Dispatch

**File:** `crates/cli/src/lib.rs` (after line 144, the `Link` dispatch)

Add dispatch for the new command:

```rust
Command::Unlink { id, url } => commands::link::remove(&id, &url),
```

**Verification:**
```bash
cargo check
```

### Phase 3: Implement Remove Functions

**File:** `crates/cli/src/commands/link.rs`

Add the `remove()` and `remove_impl()` functions (after `add_link_impl`):

```rust
/// Remove an external link from an issue.
pub fn remove(id: &str, url: &str) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    remove_impl(&db, &work_dir, &config, id, url)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn remove_impl(
    db: &Database,
    work_dir: &Path,
    config: &Config,
    id: &str,
    url: &str,
) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    // Find the link by URL
    let links = db.get_links(id)?;
    let link = links.iter().find(|l| l.url.as_deref() == Some(url));

    match link {
        Some(link) => {
            db.remove_link(link.id)?;

            // Log event (links don't sync currently)
            apply_mutation(
                db,
                work_dir,
                config,
                Event::new(id.to_string(), Action::Unlinked)
                    .with_values(Some(url.to_string()), None),
                None,
            )?;

            println!("Removed link from {}", id);
            Ok(())
        }
        None => {
            println!("Link {} not found on {}", url, id);
            Ok(())
        }
    }
}
```

**Key Implementation Notes:**
- Follow `unlabel` pattern: gracefully handle non-existent link (print message, don't error)
- Use `Action::Unlinked` with `old_value` set to the URL (opposite of `add` which sets `new_value`)
- No `OpPayload` needed since links don't sync (see comment on line 65 of existing code)

**Verification:**
```bash
cargo check
cargo test -p wk-cli link
```

### Phase 4: Add Unit Tests

**File:** `crates/cli/src/commands/link_tests.rs` (append after existing tests)

```rust
#[test]
fn test_remove_link() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Add a link first
    add_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    // Verify link exists
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);

    // Remove the link
    let result = remove_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/123",
    );
    assert!(result.is_ok());

    // Verify link is gone
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 0);
}

#[test]
fn test_remove_link_nonexistent_url() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Try to remove a link that doesn't exist (should succeed with message)
    let result = remove_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://example.com/not-linked",
    );
    assert!(result.is_ok());
}

#[test]
fn test_remove_link_nonexistent_issue() {
    let ctx = TestContext::new();

    let result = remove_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "nonexistent",
        "https://github.com/org/repo/issues/123",
    );
    assert!(result.is_err());
}

#[test]
fn test_remove_link_logs_event() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    add_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    remove_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/123",
    )
    .unwrap();

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Unlinked));
}

#[test]
fn test_remove_link_multiple_links() {
    let ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Add multiple links
    add_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/1",
        None,
    )
    .unwrap();
    add_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/2",
        None,
    )
    .unwrap();

    // Remove only one
    remove_impl(
        &ctx.db,
        &ctx.work_dir,
        &ctx.config,
        "test-1",
        "https://github.com/org/repo/issues/1",
    )
    .unwrap();

    // Verify only one remains
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].url,
        Some("https://github.com/org/repo/issues/2".to_string())
    );
}
```

**Verification:**
```bash
cargo test -p wk-cli link
```

### Phase 5: Add Spec Tests

**File:** `checks/specs/cli/unit/link.bats` (append after existing tests)

```bash
@test "unlink removes a link from an issue" {
    id=$(create_issue task "Unlink Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"

    run "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/123"
    assert_success
    assert_output --partial "Removed link"

    run "$WK_BIN" show "$id"
    refute_output --partial "Links:"
}

@test "unlink with nonexistent URL succeeds with message" {
    id=$(create_issue task "Unlink Nonexistent Test")

    run "$WK_BIN" unlink "$id" "https://example.com/not-linked"
    assert_success
    assert_output --partial "not found"
}

@test "unlink with nonexistent issue fails" {
    run "$WK_BIN" unlink "test-nonexistent" "https://github.com/org/repo/issues/123"
    assert_failure
}

@test "unlink removes only the specified link" {
    id=$(create_issue task "Unlink Multiple Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/1"
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/2"

    run "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/1"
    assert_success

    run "$WK_BIN" show "$id"
    refute_output --partial "issues/1"
    assert_output --partial "issues/2"
}

@test "log shows unlinked event" {
    id=$(create_issue task "Unlink Log Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" log "$id"
    assert_output --partial "unlinked"
}
```

**Verification:**
```bash
make spec ARGS='--file cli/unit/link.bats'
```

### Phase 6: Update Documentation

**File:** `docs/specs/04-cli-interface.md` (in the External Links section, after `wk link` examples)

Add unlink documentation:

```markdown
# Remove external link from an issue
wk unlink <id> <url>

# Examples:
wk unlink prj-a3f2 https://github.com/org/repo/issues/123
wk unlink prj-a3f2 jira://PE-5555
```

**Verification:**
```bash
# Visual inspection of docs
```

## Key Implementation Details

### URL Matching

Links are matched by exact URL comparison. The URL provided to `unlink` must match the URL that was used with `link` exactly:
- `link prj-1 https://github.com/org/repo/issues/123` requires `unlink prj-1 https://github.com/org/repo/issues/123`
- Case-sensitive comparison
- No normalization (trailing slashes matter)

### Event Logging

The `Unlinked` action uses `with_values(Some(url), None)`:
- `old_value`: the removed URL
- `new_value`: None

This is the inverse of `Linked` which uses `with_values(None, Some(url))`.

### No Sync Operations

Links are local-only data (see comment in existing `link.rs` line 65). The `apply_mutation` call passes `None` for the `OpPayload` parameter, matching the existing `add` behavior.

### Idempotent Behavior

Following the `unlabel` pattern, removing a non-existent link prints a message but does not fail. This makes the command idempotent - running `unlink` twice has the same effect as running it once.

## Verification Plan

1. **Build:** `cargo check && cargo build`
2. **Linting:** `cargo clippy`
3. **Unit tests:** `cargo test -p wk-cli link`
4. **Manual verification:**
   ```bash
   wk init --prefix test
   wk new task "Test issue"
   wk link test-XXX https://github.com/org/repo/issues/1
   wk show test-XXX  # Should show Links section
   wk unlink test-XXX https://github.com/org/repo/issues/1
   wk show test-XXX  # Should NOT show Links section
   wk log test-XXX   # Should show "unlinked" event
   ```
5. **Spec tests:** `make spec ARGS='--file cli/unit/link.bats'`
6. **Full validation:** `make check && make spec-cli`
