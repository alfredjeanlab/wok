# Plan: Add --description as Hidden Alias for --note

## Overview

Add `--description` as an undocumented (hidden) alias for `--note` in the `wok new` command. This provides backward compatibility or alternative naming for users/agents who prefer "description" semantics while keeping the documented interface clean with only `--note`.

## Project Structure

Key files involved:

```
crates/cli/
├── src/
│   ├── cli.rs              # Command definitions (clap)
│   ├── lib.rs              # Command dispatch
│   └── commands/
│       ├── new.rs          # Implementation logic
│       └── new_tests.rs    # Unit tests
checks/specs/cli/unit/
├── new.bats                # Integration tests for new command
└── help.bats               # Help output tests (hidden flag verification)
```

## Dependencies

No new dependencies required. Uses existing:
- `clap v4` with derive macros - provides `hide = true` attribute for hidden flags

## Implementation Phases

### Phase 1: Add Hidden Flag to CLI Definition
**Status: Complete**

Add `--description` flag with `hide = true` to the `New` command variant in `cli.rs`:

```rust
/// Add initial description note (hidden, use --note instead)
#[arg(long, hide = true)]
description: Option<String>,
```

Location: `crates/cli/src/cli.rs:134-136`

### Phase 2: Implement Merge Logic
**Status: Complete**

Update `run_impl` in `new.rs` to accept and merge the description parameter:

```rust
// Merge description into note - description is a hidden alias for note.
// If both are provided, note (documented flag) takes precedence.
let effective_note = note.or(description);
```

Location: `crates/cli/src/commands/new.rs:177-179`

Key behaviors:
- `--description "text"` alone: adds note with "text"
- `--note "text"` alone: adds note with "text"
- Both provided: `--note` takes precedence (documented flag wins)

### Phase 3: Update Command Dispatch
**Status: Complete**

Wire the new `description` field through `lib.rs` command dispatch to `run_impl`.

Location: `crates/cli/src/lib.rs:92-118`

### Phase 4: Add Unit Tests
**Status: Complete**

Add tests in `new_tests.rs`:

1. **Positive test - description works**: `test_run_impl_with_description`
2. **Precedence test - note wins**: `test_run_impl_note_takes_precedence_over_description`
3. **Baseline - neither provided**: `test_run_impl_without_description_or_note`
4. **Combination test**: `test_run_impl_description_with_labels`

Location: `crates/cli/src/commands/new_tests.rs:678-800`

### Phase 5: Add Integration Tests
**Status: Complete**

Add BATS tests in `new.bats`:

```bash
@test "new with --description adds note as Description" {
    # --description adds note
    id=$(create_issue task "Described task" --description "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"
    ...
}
```

Location: `checks/specs/cli/unit/new.bats:115-142`

### Phase 6: Add Negative Documentation Test
**Status: Complete**

Verify `--description` is NOT shown in help output:

```bash
@test "hidden flags not shown in help" {
    run "$WK_BIN" new --help
    assert_success
    refute_output --partial "--description"
    # but --note is shown
    assert_output --partial "--note"
    ...
}
```

Location: `checks/specs/cli/unit/help.bats:107-128`

## Key Implementation Details

### Clap Hidden Flags

Clap v4 provides `hide = true` attribute to exclude flags from help text while keeping them functional:

```rust
#[arg(long, hide = true)]
description: Option<String>,
```

This differs from aliases (`alias = "desc"`) which would be visible. Using a separate hidden field gives full control over behavior when both are provided.

### Precedence Rule

When both `--note` and `--description` are provided, `--note` takes precedence. Rationale:
- `--note` is the documented, canonical flag
- Users explicitly using `--note` have clear intent
- Hidden aliases should not override explicit documented flags

Implementation: `note.or(description)` - if `note` is `Some`, use it; otherwise fall back to `description`.

## Verification Plan

All verification steps are complete:

- [x] `cargo check` - compiles without errors
- [x] `cargo test` - unit tests pass
- [x] `make spec-cli ARGS='--filter "description"'` - integration tests pass
- [x] `make spec-cli ARGS='--filter "hidden"'` - negative doc test passes
- [x] `wok new task "Test" --description "works"` - manual verification
- [x] `wok new --help` does NOT show `--description`
- [x] `wok new task "Test" --note "A" --description "B"` - note "A" is used
