# Plan: Extract Shared JsonLineStore Trait

**Root Feature:** `wok-9c7b`

## Overview

Extract a `JsonLineStore` trait to eliminate JSONL handling duplication between `crates/core/src/oplog.rs` and `crates/cli/src/sync/queue.rs`. Both files implement nearly identical patterns for:
- Append-only JSONL file operations with fsync
- Line-by-line reading with BufReader
- Empty line filtering
- JSON serialization/deserialization

The trait will live in `wk-core` since the oplog is already there and it's the natural location for shared infrastructure.

## Project Structure

```
crates/core/src/
├── jsonl.rs          # NEW: JsonLineStore trait and helpers
├── oplog.rs          # Refactor to use JsonLineStore
└── lib.rs            # Add `pub mod jsonl`

crates/cli/src/sync/
└── queue.rs          # Refactor to use JsonLineStore from wk-core
```

Key files affected:
- `crates/core/src/oplog.rs:29-117` - JSONL read/write logic to extract
- `crates/cli/src/sync/queue.rs:40-131` - Similar JSONL logic
- `crates/core/src/lib.rs` - Export new module

## Dependencies

No new external dependencies. Uses only:
- `std::fs::{File, OpenOptions}` for file operations
- `std::io::{BufRead, BufReader, Write}` for I/O
- `std::path::{Path, PathBuf}` for path handling
- `serde::{Serialize, de::DeserializeOwned}` for generic serialization

## Implementation Phases

### Phase 1: Create JsonLineStore Module with Helper Functions

**Goal**: Create the new module with standalone helper functions for common JSONL operations.

**Files**:
- Create `crates/core/src/jsonl.rs`
- Update `crates/core/src/lib.rs`

**Implementation**:

```rust
// crates/core/src/jsonl.rs
//! JSONL (JSON Lines) file utilities.
//!
//! Provides durable append-only storage for JSON-serializable records.
//! Each record is stored as a single JSON line with fsync for durability.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::Result;

/// Appends a record to a JSONL file with fsync for durability.
pub fn append<T: Serialize>(path: &Path, record: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    let json = serde_json::to_string(record)?;
    writeln!(file, "{json}")?;
    file.sync_all()?;

    Ok(())
}

/// Reads all records from a JSONL file.
///
/// Skips empty lines and returns an empty vec if the file doesn't exist.
pub fn read_all<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: T = serde_json::from_str(&line)?;
        records.push(record);
    }

    Ok(records)
}

/// Writes all records to a JSONL file, replacing existing content.
///
/// Used for rewriting files after partial consumption (e.g., queue drain).
pub fn write_all<T: Serialize>(path: &Path, records: &[T]) -> Result<()> {
    let mut file = File::create(path)?;

    for record in records {
        let json = serde_json::to_string(record)?;
        writeln!(file, "{json}")?;
    }
    file.sync_all()?;

    Ok(())
}
```

**Update lib.rs**:
```rust
pub mod jsonl;
```

**Verification**:
- [ ] `cargo check -p wk-core`
- [ ] `cargo test -p wk-core`

---

### Phase 2: Refactor Oplog to Use jsonl Module

**Goal**: Refactor `Oplog` to use the new helper functions while preserving its deduplication logic.

**Files**:
- Update `crates/core/src/oplog.rs`

**Before** (simplified):
```rust
pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    // ... manual file reading with BufReader ...
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let op: Op = serde_json::from_str(&line)?;
        seen_ids.insert(op.id);
    }
}

pub fn append(&mut self, op: &Op) -> Result<bool> {
    // ... manual file append with fsync ...
    let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
    let json = serde_json::to_string(op)?;
    writeln!(file, "{json}")?;
    file.sync_all()?;
}
```

**After**:
```rust
use crate::jsonl;

pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    let path = path.as_ref().to_path_buf();
    let mut seen_ids = HashSet::new();

    // Use jsonl helper for reading
    let ops: Vec<Op> = jsonl::read_all(&path)?;
    for op in ops {
        seen_ids.insert(op.id);
    }

    Ok(Oplog { path, seen_ids })
}

pub fn append(&mut self, op: &Op) -> Result<bool> {
    if self.seen_ids.contains(&op.id) {
        return Ok(false);
    }

    // Use jsonl helper for append (if we have a real path)
    if !self.path.as_os_str().is_empty() {
        jsonl::append(&self.path, op)?;
    }

    self.seen_ids.insert(op.id);
    Ok(true)
}

pub fn ops_since(&self, since: Hlc) -> Result<Vec<Op>> {
    if self.path.as_os_str().is_empty() {
        return Ok(Vec::new());
    }

    // Use jsonl helper for reading, then filter
    let mut ops: Vec<Op> = jsonl::read_all(&self.path)?
        .into_iter()
        .filter(|op| op.id > since)
        .collect();
    ops.sort();
    Ok(ops)
}
```

**Verification**:
- [ ] `cargo check -p wk-core`
- [ ] `cargo test -p wk-core`
- [ ] `make spec-cli` - Oplog behavior unchanged

---

### Phase 3: Add Unit Tests for jsonl Module

**Goal**: Add comprehensive unit tests for the jsonl helper functions.

**Files**:
- Create `crates/core/src/jsonl_tests.rs`
- Update `crates/core/src/jsonl.rs` to include test module

**Implementation**:

```rust
// crates/core/src/jsonl_tests.rs
#![allow(clippy::unwrap_used)]

use super::*;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestRecord {
    id: u32,
    name: String,
}

#[test]
fn append_creates_file_if_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let record = TestRecord { id: 1, name: "first".into() };
    append(&path, &record).unwrap();

    assert!(path.exists());
}

#[test]
fn read_all_returns_empty_for_missing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("missing.jsonl");

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert!(records.is_empty());
}

#[test]
fn append_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let r1 = TestRecord { id: 1, name: "first".into() };
    let r2 = TestRecord { id: 2, name: "second".into() };

    append(&path, &r1).unwrap();
    append(&path, &r2).unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records, vec![r1, r2]);
}

#[test]
fn read_all_skips_empty_lines() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    // Write content with empty lines manually
    std::fs::write(&path, "{\"id\":1,\"name\":\"a\"}\n\n{\"id\":2,\"name\":\"b\"}\n").unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records.len(), 2);
}

#[test]
fn write_all_replaces_content() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("test.jsonl");

    let r1 = TestRecord { id: 1, name: "first".into() };
    append(&path, &r1).unwrap();

    let r2 = TestRecord { id: 2, name: "replaced".into() };
    write_all(&path, &[r2.clone()]).unwrap();

    let records: Vec<TestRecord> = read_all(&path).unwrap();
    assert_eq!(records, vec![r2]);
}
```

**Update jsonl.rs**:
```rust
#[cfg(test)]
#[path = "jsonl_tests.rs"]
mod tests;
```

**Verification**:
- [ ] `cargo test -p wk-core jsonl`

---

### Phase 4: Refactor OfflineQueue to Use jsonl Module

**Goal**: Refactor `OfflineQueue` to use the shared jsonl helpers from wk-core.

**Files**:
- Update `crates/cli/src/sync/queue.rs`

**Before**:
```rust
pub fn enqueue(&mut self, op: &Op) -> QueueResult<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&self.path)?;
    let json = serde_json::to_string(op)?;
    writeln!(file, "{}", json)?;
    file.sync_all()?;
    Ok(())
}

pub fn peek_all(&self) -> QueueResult<Vec<Op>> {
    // ... manual BufReader logic ...
}
```

**After**:
```rust
use wk_core::jsonl;

pub fn enqueue(&mut self, op: &Op) -> QueueResult<()> {
    jsonl::append(&self.path, op)?;
    Ok(())
}

pub fn peek_all(&self) -> QueueResult<Vec<Op>> {
    let ops = jsonl::read_all(&self.path)?;
    Ok(ops)
}

pub fn remove_first(&mut self, count: usize) -> QueueResult<()> {
    let ops = self.peek_all()?;
    if count >= ops.len() {
        return self.clear();
    }
    let remaining = &ops[count..];
    jsonl::write_all(&self.path, remaining)?;
    Ok(())
}
```

**Error Handling Note**: The `QueueError` type has `From<std::io::Error>` and `From<serde_json::Error>`, which matches the errors returned by jsonl functions via `wk_core::Error`. The `?` operator will convert appropriately. If needed, add:
```rust
impl From<wk_core::Error> for QueueError {
    fn from(e: wk_core::Error) -> Self {
        match e {
            wk_core::Error::Io(e) => QueueError::Io(e),
            wk_core::Error::Json(e) => QueueError::Serialization(e),
            other => QueueError::Io(std::io::Error::other(other.to_string())),
        }
    }
}
```

**Verification**:
- [ ] `cargo check -p wk-cli`
- [ ] `cargo test -p wk-cli`
- [ ] `make spec-cli`
- [ ] `make spec-remote`

---

### Phase 5: Cleanup and Final Verification

**Goal**: Remove any remaining duplicate code and verify all tests pass.

**Files**:
- `crates/core/src/oplog.rs` - Remove unused imports
- `crates/cli/src/sync/queue.rs` - Remove unused imports

**Cleanup tasks**:
1. Remove unused `use std::fs::{File, OpenOptions}` from queue.rs
2. Remove unused `use std::io::{BufRead, BufReader, Write}` from queue.rs
3. Remove unused `use std::io::{BufRead, BufReader}` from oplog.rs (keep Write if needed)
4. Run `cargo clippy` to catch any remaining issues
5. Run `cargo fmt` to format

**Verification**:
- [ ] `cargo check`
- [ ] `cargo clippy`
- [ ] `cargo fmt --check`
- [ ] `cargo test`
- [ ] `make spec`

## Key Implementation Details

### Generic Serialization Bounds

The jsonl functions use Serde's generic bounds:
- `Serialize` for writing: any type that implements Serialize can be appended
- `DeserializeOwned` for reading: any type that can be deserialized from owned data

```rust
pub fn append<T: Serialize>(path: &Path, record: &T) -> Result<()>
pub fn read_all<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>>
```

This allows the same functions to work with `Op` (for both Oplog and OfflineQueue) and any future JSONL storage needs.

### Durability Guarantees

All write operations call `file.sync_all()` to ensure data is flushed to disk before returning. This provides durability guarantees:
- Appended records survive process crashes
- Rewritten files are complete before the old content is lost

### Empty Line Handling

The `read_all` function skips empty lines (`line.trim().is_empty()`), which:
- Tolerates trailing newlines
- Handles accidental blank lines
- Matches the existing behavior in both Oplog and OfflineQueue

### Error Propagation

The jsonl module uses `wk_core::error::Result<T>` which wraps both I/O and JSON errors. Callers can use the `?` operator for ergonomic error handling.

## Verification Plan

### Unit Tests

The new `jsonl_tests.rs` module tests:
1. File creation on first append
2. Empty vec for missing files
3. Append/read roundtrip
4. Empty line skipping
5. Write-all replacing content

### Regression Tests

Existing tests validate that the refactoring preserves behavior:
1. `cargo test -p wk-core` - Oplog tests cover sync operations
2. `cargo test -p wk-cli` - Queue tests cover offline sync
3. `make spec-cli` - CLI integration tests
4. `make spec-remote` - Remote sync tests

### Manual Verification

1. Create issues while offline, verify they queue correctly
2. Reconnect, verify operations sync to server
3. Verify oplog contains all operations after sync
