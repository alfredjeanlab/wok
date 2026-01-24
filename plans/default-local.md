# Default Local Mode for `wk init`

**Root Feature:** `wok-f5ed`

## Overview

Change the default behavior of `wk init` from automatically setting up git-based remote sync (`git:.`) to local-only mode with no remote configuration. Users who want remote sync will explicitly specify `--remote .` or `--remote <url>`.

**Current behavior:** `wk init` → sets up `remote.url = "git:."` (git orphan branch sync)
**New behavior:** `wk init` → local mode, no remote; `wk init --remote .` → git sync

## Project Structure

Key files to modify:

```
crates/cli/
├── src/
│   ├── commands/
│   │   └── init.rs          # Main init logic (primary change)
│   └── cli.rs               # CLI argument definitions (help text)
└── tests/
    └── init.rs              # Unit tests

checks/specs/cli/unit/
└── init.bats                # Spec tests
```

## Dependencies

No new dependencies required. This is a behavioral change to existing code.

## Implementation Phases

### Phase 1: Update Init Command Logic

**File:** `crates/cli/src/commands/init.rs`

Change the condition for calling `setup_remote()` from "unless local flag" to "only if remote explicitly provided":

```rust
// Current (lines 74-78):
if !local {
    let remote_url = remote.as_deref().unwrap_or(".");
    setup_remote(&work_dir, &target_path, remote_url)?;
}

// New:
if let Some(remote_url) = remote.as_deref() {
    setup_remote(&work_dir, &target_path, remote_url)?;
}
```

Update `.gitignore` generation to use local mode by default:

```rust
// Current (line 69):
let local = local || workspace.is_some();

// New:
let local = remote.is_none() || workspace.is_some();
```

**Verification:** `cargo check -p wk-cli`

### Phase 2: Update CLI Help Text

**File:** `crates/cli/src/cli.rs`

Update the `--local` and `--remote` argument descriptions to reflect new defaults:

```rust
/// Remote URL for sync (git:., path, ssh URL, or ws://host:port)
/// If not specified, initializes in local-only mode
#[arg(long, value_name = "URL")]
remote: Option<String>,

/// Initialize without remote (no default sync) [default behavior]
#[arg(long, hide = true)]  // Hide since it's now the default
local: bool,
```

Consider whether to deprecate `--local` flag or keep it for explicitness.

**Verification:** `cargo build -p wk-cli && ./target/debug/wk init --help`

### Phase 3: Update Spec Tests

**File:** `checks/specs/cli/unit/init.bats`

Update tests that assume remote mode is the default:

1. **Update `.gitignore` test** (lines 211-235): Default should now include `config.toml` in gitignore

```bash
@test "init creates .gitignore with correct entries" {
    run timeout 3 "$WK_BIN" init --prefix prj
    assert_success

    # Default is now local mode - config.toml should be ignored
    run cat .wok/.gitignore
    assert_success
    assert_line "config.toml"
    assert_line "issues.db"
    assert_line "current/"
}

@test "init with remote excludes config.toml from .gitignore" {
    run timeout 3 git init
    run timeout 3 "$WK_BIN" init --prefix prj --remote .
    assert_success

    # Remote mode - config.toml should NOT be ignored (shared via git)
    run cat .wok/.gitignore
    assert_success
    refute_line "config.toml"
    assert_line "issues.db"
}
```

2. **Update git remote test** (lines 237-259): Ensure it uses explicit `--remote .`

```bash
@test "init with git remote creates worktree and supports sync" {
    run timeout 3 git init
    assert_success
    run timeout 3 "$WK_BIN" init --prefix prj --remote .  # Explicit remote
    assert_success
    [ -d ".git/wk/oplog" ]
    ...
}
```

3. **Add new test for default local behavior:**

```bash
@test "init defaults to local mode without remote" {
    run timeout 3 "$WK_BIN" init --prefix prj
    assert_success

    # Should not have remote config
    run cat .wok/config.toml
    assert_success
    refute_output --partial "[remote]"
    refute_output --partial "url ="

    # Should not create git worktree
    [ ! -d ".git/wk/oplog" ] || [ ! -d ".git" ]
}
```

**Verification:** `make spec ARGS='--file cli/unit/init.bats'`

### Phase 4: Update Unit Tests

**File:** `crates/cli/tests/init.rs`

Update any tests that assume remote mode is default:

1. Test that basic init creates local-only config (no `[remote]` section)
2. Test that `--remote .` creates config with remote section
3. Test that `--local` flag still works (for backwards compatibility)

**Verification:** `cargo test -p wk-cli`

### Phase 5: Update Documentation

**Files:**
- `docs/specs/init.md` (if exists)
- Any quickstart or getting started docs

Document the new default behavior and how to enable remote sync.

**Verification:** `quench check` (checks for broken links/references)

## Key Implementation Details

### Backwards Compatibility

The `--local` flag should remain functional for explicit local mode, even though it's now the default. This ensures existing scripts that use `--local` continue to work.

### Git Repository Detection

The current implementation only sets up git worktree when:
1. Remote is explicitly configured AND
2. The URL indicates git sync (`git:.` or `git:<path>`)

This behavior remains unchanged; we're only changing when remote gets configured.

### Config File Behavior

| Mode | `config.toml` in `.gitignore`? | `[remote]` section? |
|------|-------------------------------|---------------------|
| Local (default) | Yes | No |
| Remote (`--remote .`) | No | Yes |

## Verification Plan

### Phase 1 Verification
```bash
cargo check -p wk-cli
```

### Phase 2 Verification
```bash
cargo build -p wk-cli
./target/debug/wk init --help  # Check help text
```

### Phase 3-4 Verification
```bash
make spec ARGS='--file cli/unit/init.bats'
cargo test -p wk-cli
```

### Full Verification
```bash
make check        # Full lint/build/test cycle
make spec-cli     # All CLI specs
```

### Manual Verification
```bash
# Test 1: Default local mode
mkdir /tmp/test-local && cd /tmp/test-local
wk init --prefix test
cat .wok/config.toml   # Should NOT have [remote] section
cat .wok/.gitignore    # Should include config.toml

# Test 2: Explicit remote mode
mkdir /tmp/test-remote && cd /tmp/test-remote
git init
wk init --prefix test --remote .
cat .wok/config.toml   # Should have [remote] section with url = "git:."
cat .wok/.gitignore    # Should NOT include config.toml
ls .git/wk/oplog/      # Should exist with oplog.jsonl
```
