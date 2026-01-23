# Plan: Add From<cli::Status> for wk_core::Status

**Root Feature:** `wok-5007`

## Overview

Add a `From<cli::Status> for wk_core::Status` implementation to eliminate the duplicate match statement in `note.rs` that manually converts between the two identical Status enum types. This provides a standard, reusable conversion path.

## Project Structure

Key files involved:

```
crates/cli/src/
├── models/
│   ├── issue.rs          # cli::Status definition (add From impl here)
│   └── issue_tests.rs    # Add conversion tests
├── commands/
│   └── note.rs           # Has duplicate match (use .into())
└── schema/
    └── mod.rs            # Existing From<cli::Status> for schema::Status (reference pattern)
```

## Dependencies

No new dependencies required. Uses standard library `From` trait.

## Implementation Phases

### Phase 1: Add From Implementation

Add `From<Status> for wk_core::Status` to `crates/cli/src/models/issue.rs` after the existing `FromStr` implementation:

```rust
impl From<Status> for wk_core::Status {
    fn from(status: Status) -> Self {
        match status {
            Status::Todo => wk_core::Status::Todo,
            Status::InProgress => wk_core::Status::InProgress,
            Status::Done => wk_core::Status::Done,
            Status::Closed => wk_core::Status::Closed,
        }
    }
}
```

**Verification:** `cargo check -p wk`

### Phase 2: Add Unit Tests

Add conversion tests to `crates/cli/src/models/issue_tests.rs`:

```rust
#[test]
fn status_converts_to_core_status() {
    assert_eq!(wk_core::Status::Todo, Status::Todo.into());
    assert_eq!(wk_core::Status::InProgress, Status::InProgress.into());
    assert_eq!(wk_core::Status::Done, Status::Done.into());
    assert_eq!(wk_core::Status::Closed, Status::Closed.into());
}
```

**Verification:** `cargo test -p wk status_converts`

### Phase 3: Update note.rs to Use Into

Replace the match statement in `crates/cli/src/commands/note.rs` (lines 49-55):

**Before:**
```rust
// Convert status for sync
let core_status = match issue.status {
    Status::Todo => wk_core::Status::Todo,
    Status::InProgress => wk_core::Status::InProgress,
    Status::Done => wk_core::Status::Done,
    Status::Closed => wk_core::Status::Closed,
};
```

**After:**
```rust
// Convert status for sync
let core_status: wk_core::Status = issue.status.into();
```

**Verification:** `cargo test -p wk note` and `make spec-cli ARGS='--filter note'`

## Key Implementation Details

1. **Location choice**: The `From` impl goes in the CLI crate (`issue.rs`) because:
   - CLI depends on core, not vice versa
   - Follows the orphan rule (impl must be in same crate as trait or type)
   - Matches existing pattern in `schema/mod.rs`

2. **Conversion direction**: Only implementing `cli::Status -> wk_core::Status`:
   - This is the only direction currently needed (CLI creates core OpPayload messages)
   - Adding reverse conversion can be done later if needed

3. **Alternative sites not updated**: The `lifecycle.rs` and `new.rs` files use literal `wk_core::Status::*` variants (e.g., `wk_core::Status::Todo`) rather than converting from cli::Status. These are intentional and don't benefit from the conversion trait since there's no cli::Status value to convert from.

## Verification Plan

1. **Compile check**: `cargo check -p wk`
2. **Unit tests**: `cargo test -p wk`
3. **CLI specs**: `make spec-cli`
4. **Linting**: `cargo clippy -p wk`
5. **Format**: `cargo fmt --check`

## Related Patterns

The existing `From<crate::models::Status> for schema::Status` in `crates/cli/src/schema/mod.rs` (lines 98-107) serves as a reference implementation for this pattern.
