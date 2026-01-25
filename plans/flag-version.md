# Plan: Version Flag Implementation

## Overview

Add `-v`/`--version` flags to the `wok` CLI to output the version number. Also accept `-V` silently (undocumented) for compatibility. The output should be just the version string (e.g., `0.4.0`).

## Project Structure

```
crates/cli/src/cli.rs        # Add version flag configuration
checks/specs/cli/unit/version.bats  # New spec file for version tests
```

## Dependencies

- No new dependencies needed
- Uses `clap` (already v4) built-in version support
- Uses `env!("CARGO_PKG_VERSION")` macro (workspace version: `0.4.0`)

## Implementation Phases

### Phase 1: Update CLI to Support Version Flags

**File:** `crates/cli/src/cli.rs`

Add version support to the `Cli` struct with custom flag configuration:

```rust
#[derive(Parser)]
#[command(name = "wok")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "A collaborative, offline-first, AI-friendly issue tracker with dependency tracking"
)]
// ... existing attributes ...
pub struct Cli {
    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: (),

    /// Print version (undocumented alias)
    #[arg(short = 'V', action = clap::ArgAction::Version, hide = true)]
    version_upper: (),

    #[command(subcommand)]
    pub command: Command,
}
```

**Key Implementation Notes:**
- `#[command(version = env!("CARGO_PKG_VERSION"))]` sets the version string
- `#[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]` creates documented `-v`/`--version`
- `#[arg(short = 'V', action = clap::ArgAction::Version, hide = true)]` creates hidden `-V`
- Unit type `()` is used since the action handles the output directly

**Verification:**
```bash
cargo build --release
./target/release/wk --version  # Should output: wk 0.4.0
./target/release/wk -v         # Should output: wk 0.4.0
./target/release/wk -V         # Should output: wk 0.4.0
```

### Phase 2: Create Version Spec Tests

**File:** `checks/specs/cli/unit/version.bats`

Create comprehensive spec tests:

```bash
#!/usr/bin/env bats
load '../../helpers/common'

# Version flag tests - verifies -v, --version, and -V behavior.
# NOTE: These tests only check version output and don't need wk init.

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

# Positive tests - flags work correctly

@test "--version outputs version" {
    run "$WK_BIN" --version
    assert_success
    assert_output --partial "wk"
    # Version should be semver-like
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-v outputs version" {
    run "$WK_BIN" -v
    assert_success
    assert_output --partial "wk"
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-V outputs version (silent alias)" {
    run "$WK_BIN" -V
    assert_success
    assert_output --partial "wk"
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-v and --version produce identical output" {
    run "$WK_BIN" -v
    local v_output="$output"
    run "$WK_BIN" --version
    [ "$v_output" = "$output" ]
}

@test "-V produces same output as -v" {
    run "$WK_BIN" -v
    local v_output="$output"
    run "$WK_BIN" -V
    [ "$v_output" = "$output" ]
}

# Negative tests - help output

@test "-v is documented in help" {
    run "$WK_BIN" --help
    assert_success
    assert_output --partial "-v"
    assert_output --partial "--version"
}

@test "-V is NOT documented in help" {
    run "$WK_BIN" --help
    assert_success
    # -V should be hidden
    refute_output --regexp "\s-V[,\s]"
    refute_output --partial "[-V"
}

@test "version subcommand does not exist" {
    # This test exists in no_aliases.bats but we reinforce it here
    run "$WK_BIN" version
    assert_failure
}
```

**Verification:**
```bash
make spec ARGS='--file cli/unit/version.bats'
```

### Phase 3: Update Existing No-Aliases Tests

**File:** `checks/specs/cli/edge_cases/no_aliases.bats`

The existing tests at lines 53, 74, 83 already verify:
- `wk version` subcommand does NOT exist
- "version" is not mentioned in help as a command
- `help version` fails

These tests remain valid and should still pass. No changes needed.

**Verification:**
```bash
make spec ARGS='--file cli/edge_cases/no_aliases.bats'
```

### Phase 4: Run Full Validation

Run the complete validation suite:

```bash
make check
make spec-cli
```

## Key Implementation Details

### Clap Version Action

Using `clap::ArgAction::Version` causes clap to:
1. Print the version string (set via `#[command(version = ...)]`)
2. Exit with status code 0

The output format is: `{binary_name} {version}` (e.g., `wk 0.4.0`)

### Flag Priority

Both `-v` and `-V` use the same action (`ArgAction::Version`). Either flag triggers version output. The `hide = true` attribute on the `-V` variant keeps it out of help text.

### Existing Test Compatibility

The `no_aliases.bats` tests verify `version` is NOT a subcommand. This remains true - we're adding a **flag**, not a command. The tests should continue to pass.

## Verification Plan

1. **Build check:** `cargo check` and `cargo build`
2. **Unit tests:** `cargo test`
3. **Linting:** `cargo clippy`
4. **Version flag behavior:**
   - `wk -v` outputs version
   - `wk --version` outputs version
   - `wk -V` outputs version
5. **Help output:**
   - `wk --help` shows `-v, --version`
   - `wk --help` does NOT show `-V`
6. **Spec tests:**
   - `make spec ARGS='--file cli/unit/version.bats'`
   - `make spec ARGS='--file cli/edge_cases/no_aliases.bats'`
7. **Full validation:** `make check`
