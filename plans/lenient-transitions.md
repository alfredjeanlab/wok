# Lenient Transitions

## Overview

Remove strict state machine enforcement from wok lifecycle commands (`start`, `done`, `close`, `reopen`) so that every command succeeds from any state by applying the most sensible intermediate transitions. If the issue is already in the target state, succeed silently (idempotent). No new CLI flags — just make existing commands forgiving.

## Project Structure

Key files to modify:

```
crates/core/src/issue.rs            # Status::can_transition_to, valid_targets
crates/cli/src/models/issue.rs      # Duplicate Status::can_transition_to, valid_targets
crates/cli/src/commands/lifecycle.rs # start_single, done_single, close_single, reopen_single
crates/core/src/issue_tests.rs      # Unit tests for core Status
crates/cli/src/models/issue_tests.rs # Unit tests for CLI Status
crates/cli/src/commands/lifecycle_tests.rs # Unit tests for lifecycle commands
tests/specs/cli/unit/lifecycle.bats  # BATS integration specs
```

## Dependencies

No new external libraries needed. All changes are internal logic changes.

## Implementation Phases

### Phase 1: Update specs to reflect lenient behavior

Update `tests/specs/cli/unit/lifecycle.bats` to encode the new expected behavior:

1. **Remove "invalid transitions fail" test** (lines 66-91) — replace with a new test proving lenient transitions work:
   - `wk start` on a done issue → reopens then starts (status: in_progress)
   - `wk start` on a closed issue → reopens then starts (status: in_progress)
   - `wk start` on an in_progress issue → succeeds silently (idempotent)
   - `wk done` on a closed issue → moves to done
   - `wk done` on a done issue → succeeds silently (idempotent)
   - `wk close` on a done issue → moves to closed
   - `wk close` on a closed issue → succeeds silently (idempotent)
   - `wk reopen` on a todo issue → succeeds silently (idempotent)
   - `wk reopen` on an in_progress issue → moves to todo (already works)

2. **Update "batch start with invalid status" test** (lines 195-205) — `start` on an already-in_progress issue should now succeed (idempotent), so the batch succeeds fully (2 of 2).

3. **Update "batch start with mixed unknown and invalid" test** (lines 219-229) — the previously-started issue should now be idempotent, so only the unknown ID fails.

4. **Keep existing passing tests** — the happy-path tests for basic transitions, reasons, and batch operations should still pass unchanged.

### Phase 2: Allow all transitions in `can_transition_to`

In both `crates/core/src/issue.rs` and `crates/cli/src/models/issue.rs`, update `can_transition_to` to allow every non-self transition:

```rust
pub fn can_transition_to(&self, target: Status) -> bool {
    *self != target
}
```

This opens up the previously-blocked transitions:
- `Done → InProgress` (needed for `start` on done)
- `Done → Closed` (needed for `close` on done)
- `Closed → InProgress` (needed for `start` on closed)
- `Closed → Done` (needed for `done` on closed)
- `Closed → Closed` and other self-transitions will be handled as idempotent at the command level

Update `valid_targets` accordingly — since all transitions are now valid, this method may simplify or can be removed if no longer referenced in error paths. If kept, update to reflect reality.

Update the unit tests in `crates/core/src/issue_tests.rs` and `crates/cli/src/models/issue_tests.rs` to match the new transition rules.

### Phase 3: Make lifecycle commands lenient with idempotency

Modify each command in `crates/cli/src/commands/lifecycle.rs`:

#### `start_single` (line 242)
Current: fails unless status is `Todo`.
New behavior:
- If already `InProgress` → succeed silently (print nothing or "Already started {id}")
- If `Todo` → transition to `InProgress` (unchanged)
- If `Done` or `Closed` → transition directly to `InProgress`, log a `Reopened` event followed by a `Started` event (or just a `Started` event with the from-value recording the original state)

```rust
fn start_single(db: &Database, id: &str) -> Result<()> {
    let resolved_id = db.resolve_id(id)?;
    let issue = db.get_issue(&resolved_id)?;

    if issue.status == Status::InProgress {
        return Ok(()); // idempotent
    }

    db.update_issue_status(&resolved_id, Status::InProgress)?;

    apply_mutation(
        db,
        Event::new(resolved_id.clone(), Action::Started).with_values(
            Some(issue.status.to_string()),
            Some("in_progress".to_string()),
        ),
    )?;

    println!("Started {}", resolved_id);
    Ok(())
}
```

#### `done_single` (line 287)
Current: fails from `Done` or `Closed`.
New behavior:
- If already `Done` → succeed silently
- If `Todo` → still require reason (unchanged logic)
- If `InProgress` → transition to `Done` (unchanged)
- If `Closed` → transition to `Done`, log unblocked events

```rust
// At the top of done_single, add idempotent check:
if issue.status == Status::Done {
    return Ok(()); // idempotent
}
// Remove the can_transition_to check; proceed with transition from any state
```

#### `close_single` (line 380)
Current: fails from `Done` or `Closed`.
New behavior:
- If already `Closed` → succeed silently
- Any other state → transition to `Closed` (reason still required)

#### `reopen_single` (line 429)
Current: fails from `Todo`.
New behavior:
- If already `Todo` → succeed silently
- If `InProgress` → transition to `Todo` (no reason needed, unchanged)
- If `Done` or `Closed` → transition to `Todo` (reason required, unchanged)

### Phase 4: Update unit tests

Update `crates/cli/src/commands/lifecycle_tests.rs`:

1. **Change tests that assert `InvalidTransition` errors** for now-valid transitions — these should assert success instead.
2. **Add idempotency tests**: calling `start` on in_progress, `done` on done, `close` on closed, `reopen` on todo should all succeed with no error and no state change.
3. **Add cross-state tests**: `done` on closed, `close` on done, `start` on done, `start` on closed.
4. **Keep reason-required tests**: `done` from `todo` still requires reason, `close` always requires reason, `reopen` from terminal states still requires reason.

## Key Implementation Details

### Idempotency pattern

Every command checks "am I already in the target state?" first and returns `Ok(())` silently. This avoids logging duplicate events and keeps the event log clean.

```rust
if issue.status == TARGET_STATUS {
    return Ok(()); // already there, nothing to do
}
```

For idempotent cases, **do not print** the usual "Started/Completed/Closed/Reopened {id}" message — the caller sees no output, which is the expected UX for a no-op.

### Event logging for cross-state transitions

When a command does a "lenient" transition that wasn't previously allowed (e.g., `start` from `closed`), log a single event with the actual `from` state. Do **not** synthesize intermediate events (e.g., don't emit both `Reopened` and `Started`). The event's `from_value`/`to_value` captures the real transition path.

### Reason requirements stay the same

- `done` from `todo` still requires `--reason` (or auto-generates for humans)
- `close` always requires `--reason`
- `reopen` from `done`/`closed` still requires `--reason`
- `done` from `closed` should require `--reason` (same logic as from `todo` — skipping the normal flow)
- `start` never requires `--reason` (even from terminal states)

### Batch operations

The bulk_operation pattern doesn't change. Since individual operations now succeed more often, batch success rates go up naturally. The partial-update semantics remain for truly invalid cases (unknown IDs).

### Two Status enums

Both `crates/core/src/issue.rs::Status` and `crates/cli/src/models/issue.rs::Status` have `can_transition_to`. Both must be updated. Consider whether `can_transition_to` and `valid_targets` are still useful after this change — if the CLI commands handle all logic directly, the core `can_transition_to` may become trivially `true` for all non-self transitions and `valid_targets` can be simplified or removed.

## Verification Plan

1. **Unit tests**: `cargo test` — verify all updated tests in `lifecycle_tests.rs`, `issue_tests.rs`, and `issue_tests.rs` (CLI) pass
2. **BATS specs**: `make spec ARGS='--file cli/unit/lifecycle.bats'` — verify all lifecycle integration tests pass
3. **Full spec suite**: `make spec-cli` — verify no regressions in other CLI specs
4. **Fast check**: `make check-fast` — fmt, clippy, build, test all pass
5. **Manual smoke test** (optional):
   - Create an issue, close it, then `wk done {id}` → should succeed
   - Create an issue, complete it, then `wk start {id}` → should succeed
   - Run `wk start {id}` twice → second should succeed silently
