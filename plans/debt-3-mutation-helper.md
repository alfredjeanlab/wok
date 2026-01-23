# Plan: apply_mutation Helper

## Overview

Create an `apply_mutation()` helper function to unify the repeated event+log+queue pattern across CLI command files. Currently there are ~38 instances of `db.log_event()` and ~28 instances of `queue_op()` spread across 7 command files. This helper will reduce boilerplate, ensure consistency, and make the mutation pattern easier to maintain.

## Project Structure

```
crates/cli/src/
├── commands/
│   ├── mod.rs          # Add apply_mutation() helper alongside queue_op()
│   ├── dep.rs          # 8 log_event + 12 queue_op → refactor
│   ├── edit.rs         # 5 log_event + 2 queue_op → refactor
│   ├── label.rs        # 2 log_event + 2 queue_op → refactor
│   ├── lifecycle.rs    # 7 log_event + 6 queue_op → refactor
│   ├── link.rs         # 2 log_event + 0 queue_op → partial (no sync)
│   ├── new.rs          # 3 log_event + 3 queue_op → refactor
│   └── note.rs         # 2 log_event + 2 queue_op → refactor
└── models/
    └── event.rs        # Event, Action types (no changes needed)
```

## Dependencies

No new external dependencies required. Uses existing:
- `wk_core::OpPayload` - operation payloads for sync queue
- `crate::models::{Event, Action}` - local event logging
- `crate::db::Database` - database operations

## Implementation Phases

### Phase 1: Design and Implement Core Helper

Create the `apply_mutation()` function in `crates/cli/src/commands/mod.rs`.

**Key design decisions:**
- Helper accepts an `Event` and optional `OpPayload`
- Logs event to database first, then queues operation
- Returns `Result<()>` propagating any errors
- Supports mutations that don't sync (e.g., link.rs has no queue_op)

```rust
/// Apply a mutation by logging an event and optionally queueing a sync operation.
///
/// This helper unifies the common pattern of:
/// 1. Creating an Event
/// 2. Logging it to the local database
/// 3. Queueing an operation for remote sync
///
/// Use this for all issue mutations to ensure consistent audit trail and sync behavior.
pub fn apply_mutation(
    db: &Database,
    work_dir: &Path,
    config: &Config,
    event: Event,
    payload: Option<OpPayload>,
) -> Result<()> {
    // Log event to local database first
    db.log_event(&event)?;

    // Queue operation for sync if provided
    if let Some(p) = payload {
        queue_op(work_dir, config, p)?;
    }

    Ok(())
}
```

**Alternative: Builder pattern for complex mutations**

For cases where the pattern is more complex (e.g., done_single which also adds notes), consider a MutationBuilder:

```rust
pub struct MutationBuilder<'a> {
    db: &'a Database,
    work_dir: &'a Path,
    config: &'a Config,
    event: Option<Event>,
    payload: Option<OpPayload>,
}

impl<'a> MutationBuilder<'a> {
    pub fn new(db: &'a Database, work_dir: &'a Path, config: &'a Config) -> Self { ... }
    pub fn event(mut self, event: Event) -> Self { ... }
    pub fn sync(mut self, payload: OpPayload) -> Self { ... }
    pub fn apply(self) -> Result<()> { ... }
}
```

This phase focuses on the simpler function approach, with the builder as optional enhancement.

**Verification:**
- Unit tests for `apply_mutation()` with and without payload
- Ensure error propagation works correctly

### Phase 2: Refactor Simple Commands (label.rs, note.rs)

Refactor the simplest command files first to validate the pattern.

**label.rs (4 instances):**
```rust
// Before:
let event = Event::new(id.to_string(), Action::Labeled)
    .with_values(None, Some(label.to_string()));
db.log_event(&event)?;
queue_op(work_dir, config, OpPayload::add_label(id.to_string(), label.to_string()))?;

// After:
apply_mutation(
    db,
    work_dir,
    config,
    Event::new(id.to_string(), Action::Labeled)
        .with_values(None, Some(label.to_string())),
    Some(OpPayload::add_label(id.to_string(), label.to_string())),
)?;
```

**note.rs (4 instances):**
Similar pattern for `Noted` action with status conversion.

**Verification:**
- Run `make spec-cli` to ensure behavior unchanged
- Run `cargo test` for unit tests

### Phase 3: Refactor new.rs and edit.rs

**new.rs (6 instances):**
- `Created` event (no values)
- `Labeled` events in loop
- `Noted` event for description

**edit.rs (5 instances + 2 queue_op):**
- `Edited` for title/type/description
- `Assigned`/`Unassigned` for assignee changes
- Note: description edit has no queue_op (local-only)

Handle the special case where `queue_op` is conditionally skipped:
```rust
// edit.rs description case - no sync
apply_mutation(
    db,
    work_dir,
    config,
    Event::new(id.to_string(), Action::Edited)
        .with_values(old_desc, Some(trimmed_desc.clone())),
    None,  // No sync for description edits
)?;
```

**Verification:**
- Run `make spec-cli`
- Check edit tests specifically: `make spec ARGS='--filter edit'`

### Phase 4: Refactor lifecycle.rs

This is the most complex file with 7 `log_event` and 6 `queue_op` calls.

**Functions to refactor:**
- `start_single()` - Started action
- `done_single()` - Done action with optional reason
- `done_single_with_reason()` - Done action (always has reason)
- `close_single()` - Closed action
- `reopen_single()` - Reopened action with destination
- `reopen_single_with_reason()` - Reopened action (always has reason)
- `log_unblocked_events()` - Unblocked actions (no queue_op)

**Special considerations:**
- `done_single` conditionally adds notes and calls `log_unblocked_events`
- Use `apply_mutation(..., None)` for log-only calls in `log_unblocked_events()`

**Verification:**
- Run lifecycle-specific tests: `make spec ARGS='--filter lifecycle'`
- Test bulk operations still work

### Phase 5: Refactor dep.rs

Most repetitive file with 8 `log_event` and 12 `queue_op` calls.

**Pattern in dep.rs:**
- Each relation type (Blocks, BlockedBy, Tracks, TrackedBy) has similar code
- Some cases queue multiple operations (bidirectional Tracks/TrackedBy)

**Handle multiple operations:**
```rust
// Tracks creates bidirectional relationship
apply_mutation(
    db, work_dir, config,
    Event::new(from_id.to_string(), Action::Related)
        .with_values(None, Some(format!("tracks {}", to_id))),
    Some(OpPayload::add_dep(from_id.to_string(), to_id.to_string(), Relation::Tracks)),
)?;
// Second queue_op for reverse direction
queue_op(work_dir, config, OpPayload::add_dep(
    to_id.to_string(), from_id.to_string(), Relation::TrackedBy))?;
```

Alternatively, consider a multi-payload variant or just use two calls.

**Verification:**
- Run dependency tests: `make spec ARGS='--filter dep'`
- Test bidirectional relationships

### Phase 6: Refactor link.rs and Cleanup

**link.rs (2 instances, no queue_op):**
```rust
apply_mutation(
    db,
    work_dir,
    config,
    Event::new(id.to_string(), Action::Linked)
        .with_values(None, Some(url.to_string())),
    None,  // Links don't sync currently
)?;
```

**Final cleanup:**
- Remove any unused imports
- Run full test suite
- Update documentation if needed

**Verification:**
- `make validate` - full validation
- `cargo clippy` - no new warnings
- `make coverage` - maintain ≥85% coverage

## Key Implementation Details

### Error Handling

The helper propagates errors from both `log_event` and `queue_op`:
```rust
pub fn apply_mutation(...) -> Result<()> {
    db.log_event(&event)?;  // Fails here → no queue_op
    if let Some(p) = payload {
        queue_op(work_dir, config, p)?;  // Fails here → event was logged
    }
    Ok(())
}
```

This matches current behavior where event logging is attempted before queueing.

### Type Conversions

Some commands convert CLI types to core types for `OpPayload`. This happens at the call site, not in the helper:
```rust
// Status conversion happens before apply_mutation
let core_status = match issue.status {
    Status::Todo => wk_core::Status::Todo,
    // ...
};
apply_mutation(db, work_dir, config, event,
    Some(OpPayload::set_status(id, core_status, reason)))?;
```

### Conditional Sync

Some mutations don't sync (description edits, links). Pass `None` for payload:
```rust
apply_mutation(db, work_dir, config, event, None)?;
```

### Multiple Operations

For cases like `Tracks` that queue two operations:
1. Call `apply_mutation()` with the first operation
2. Call `queue_op()` directly for additional operations

This keeps the helper simple while allowing complex cases.

## Verification Plan

### Per-Phase Testing
1. **Unit tests**: Add tests in `mod_tests.rs` for `apply_mutation()`
2. **Spec tests**: Run relevant spec tests after each phase
3. **Regression**: Ensure no behavior changes

### Final Verification
```bash
# Full validation
make validate

# Specific checks
cargo check
cargo clippy
cargo test
make spec-cli
make spec-remote
make coverage  # Verify ≥85%
```

### Test Coverage Areas
- Event logging with all Action types
- Event logging with/without values
- Event logging with/without reason
- Sync queueing with various OpPayload types
- Error propagation from log_event
- Error propagation from queue_op
- No-sync mutations (None payload)
