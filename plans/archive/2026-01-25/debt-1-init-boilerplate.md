# Plan: Consolidate Database Initialization Boilerplate

**Root Feature:** `wok-0791`

## Overview

Refactor CLI command files to consistently use the `open_db()` helper instead of repeating the 4-line database initialization pattern. The existing helper needs to be extended to also return `work_dir`, which is required by commands that call `queue_op()`.

## Current State

### Existing Helper (`crates/cli/src/commands/mod.rs:39-46`)
```rust
pub fn open_db() -> Result<(Database, Config)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    Ok((db, config))
}
```

### The 4-Line Pattern Being Eliminated
```rust
let work_dir = find_work_dir()?;
let config = Config::load(&work_dir)?;
let db_path = get_db_path(&work_dir, &config);
let db = Database::open(&db_path)?;
```

### Files Already Using `open_db()`
- `show.rs` - `let (db, _) = open_db()?;`
- `list.rs` - `let (db, config) = open_db()?;`
- `ready.rs` - `let (db, config) = open_db()?;`
- `search.rs` - `let (db, _) = open_db()?;`
- `tree.rs` - `let (db, _) = open_db()?;`
- `log.rs` - `let (db, _) = open_db()?;`
- `export.rs` - `let (db, _) = open_db()?;`
- `import.rs` - `let (db, config) = open_db()?;`
- `link.rs` - `let (db, _) = open_db()?;`

### Files Still Using 4-Line Pattern
| File | Occurrences | Why `work_dir` is needed |
|------|-------------|-------------------------|
| `note.rs` | 1 | `queue_op(work_dir, config, ...)` |
| `label.rs` | 2 | `queue_op(work_dir, config, ...)` |
| `edit.rs` | 1 | `queue_op(work_dir, config, ...)` |
| `dep.rs` | 2 | `queue_op(work_dir, config, ...)` |
| `lifecycle.rs` | 4 | `queue_op(work_dir, config, ...)` |
| `new.rs` | 1 | `queue_op(work_dir, config, ...)` |

## Project Structure

```
crates/cli/src/commands/
├── mod.rs          # Contains open_db() helper (modify)
├── note.rs         # 1 occurrence (refactor)
├── label.rs        # 2 occurrences (refactor)
├── edit.rs         # 1 occurrence (refactor)
├── dep.rs          # 2 occurrences (refactor)
├── lifecycle.rs    # 4 occurrences (refactor)
├── new.rs          # 1 occurrence (refactor)
└── [9 files]       # Already using helper (update signature)
```

## Dependencies

None. This is a pure refactoring that uses existing types (`std::path::PathBuf`).

## Implementation Phases

### Phase 1: Extend `open_db()` Helper

**Goal**: Modify helper to return `work_dir` in addition to `Database` and `Config`.

**File**: `crates/cli/src/commands/mod.rs`

**Changes**:
```rust
use std::path::PathBuf;

/// Helper to open the database from the current context
pub fn open_db() -> Result<(Database, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    Ok((db, config, work_dir))
}
```

**Verification**: `cargo check` will show all call sites that need updating.

### Phase 2: Update Existing Callers

**Goal**: Update the 9 files already using `open_db()` to handle the new return signature.

**Pattern transformation**:
```rust
// Before
let (db, _) = open_db()?;
// After
let (db, _, _) = open_db()?;

// Before
let (db, config) = open_db()?;
// After
let (db, config, _) = open_db()?;
```

**Files to update**:
- `show.rs:28` - `let (db, _, _) = open_db()?;`
- `list.rs` - `let (db, config, _) = open_db()?;`
- `ready.rs` - `let (db, config, _) = open_db()?;`
- `search.rs` - `let (db, _, _) = open_db()?;`
- `tree.rs` - `let (db, _, _) = open_db()?;`
- `log.rs` - `let (db, _, _) = open_db()?;`
- `export.rs` - `let (db, _, _) = open_db()?;`
- `import.rs` - `let (db, config, _) = open_db()?;`
- `link.rs` - `let (db, _, _) = open_db()?;`

**Verification**: `cargo check` should pass.

### Phase 3: Refactor Files Using 4-Line Pattern

**Goal**: Replace all 4-line patterns with the helper.

**Pattern transformation**:
```rust
// Before
pub fn run(...) -> Result<()> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    run_impl(&db, &config, &work_dir, ...)
}

// After
pub fn run(...) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    run_impl(&db, &config, &work_dir, ...)
}
```

**Files and changes**:

1. **`note.rs`** (1 change)
   - Line 16-21: Replace pattern in `run()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

2. **`label.rs`** (2 changes)
   - Line 16-21: Replace pattern in `add()`
   - Line 73-78: Replace pattern in `remove()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

3. **`edit.rs`** (1 change)
   - Line 19-24: Replace pattern in `run()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

4. **`dep.rs`** (2 changes)
   - Line 15-20: Replace pattern in `add()`
   - Line 153-158: Replace pattern in `remove()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

5. **`lifecycle.rs`** (4 changes)
   - Line 76-81: Replace pattern in `start()`
   - Line 184-188: Replace pattern in `done()`
   - Line 366-370: Replace pattern in `close()`
   - Line 488-492: Replace pattern in `reopen()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

6. **`new.rs`** (1 change)
   - Line 32-36: Replace pattern in `run()`
   - Remove imports: `find_work_dir`, `get_db_path`
   - Add import: `super::open_db`

**Verification**: `cargo check`, `cargo test`

### Phase 4: Clean Up Unused Imports

**Goal**: Remove now-unused imports from refactored files.

For each refactored file, change:
```rust
// Before
use crate::config::{find_work_dir, get_db_path, Config};
use crate::db::Database;

// After
use crate::config::Config;

use super::open_db;
```

Note: `Config` is still needed for the `_impl` function signatures and `Database` is still used by `_impl` functions. Only `find_work_dir` and `get_db_path` should be removed.

**Verification**: `cargo check` (no unused import warnings)

## Key Implementation Details

### Why Return `PathBuf` Instead of `&Path`

The helper must own the `work_dir` value to return it. Using `PathBuf` allows the caller to borrow it as `&Path` when passing to `run_impl()` functions.

### Preserving `_impl` Function Signatures

The `*_impl()` functions (e.g., `run_impl`, `add_impl`) keep their existing signatures accepting `&Path` for `work_dir`. This:
- Maintains testability (tests can pass any path)
- Avoids touching test code
- Keeps the public API stable

### Import Organization

After refactoring, typical import section:
```rust
use std::path::Path;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;
use crate::error::Result;
// ... other imports

use super::{open_db, queue_op};
```

## Verification Plan

1. **After Phase 1**: `cargo check` shows expected compile errors at call sites
2. **After Phase 2**: `cargo check` passes, `cargo test` passes for affected files
3. **After Phase 3**: `cargo check` passes, `cargo test` passes
4. **After Phase 4**: `cargo clippy` shows no unused import warnings
5. **Final**: `make validate` passes (full test suite)

## Summary Statistics

- **Lines removed**: ~66 (11 occurrences × 6 lines each)
- **Lines added**: ~11 (11 occurrences × 1 line each)
- **Net reduction**: ~55 lines
- **Files modified**: 16 total (1 helper + 9 existing callers + 6 new callers)
