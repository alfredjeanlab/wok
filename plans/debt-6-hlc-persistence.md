# Plan: HlcPersistence Type for Unified HLC Storage

**Root Feature:** `wok-e74e`

## Overview

Create a generic `HlcPersistence` type to eliminate duplicate `last_hlc`/`server_hlc` function pairs in `crates/cli/src/commands/mod.rs`. Currently, there are 8 nearly-identical functions (4 operations × 2 HLC types) that can be unified into a single struct with 4 methods, reducing code by ~50% and improving maintainability.

## Project Structure

```
crates/cli/src/
├── commands/
│   ├── mod.rs              # Current location of duplicate functions (lines 51-118)
│   └── hlc_persistence.rs  # NEW: HlcPersistence type
└── daemon/
    └── runner.rs           # Primary consumer of HLC persistence functions
```

Key files affected:
- `crates/cli/src/commands/mod.rs:51-118` - 8 duplicate functions to replace
- `crates/cli/src/commands/hlc_persistence.rs` - New module for HlcPersistence
- `crates/cli/src/daemon/runner.rs` - Update call sites (lines 246, 525-536, 629-630, 671)

## Dependencies

No new external dependencies required. Uses only:
- `std::fs` for file operations
- `std::io::Write` for writing
- `std::path::{Path, PathBuf}` for path handling
- `wok_core::Hlc` for the HLC type

## Implementation Phases

### Phase 1: Create HlcPersistence Type

**Goal**: Implement the generic HlcPersistence struct in a new module.

**Files**:
- Create `crates/cli/src/commands/hlc_persistence.rs`
- Update `crates/cli/src/commands/mod.rs` to add `mod hlc_persistence; pub use hlc_persistence::HlcPersistence;`

**Implementation**:

```rust
// crates/cli/src/commands/hlc_persistence.rs
use anyhow::Result;
use std::path::{Path, PathBuf};
use wok_core::Hlc;

/// Generic HLC persistence abstraction for different HLC kinds
/// (e.g., local "last_hlc", server-side "server_hlc")
pub struct HlcPersistence {
    path: PathBuf,
}

impl HlcPersistence {
    /// Create persistence handler for a specific HLC file
    pub fn new(daemon_dir: &Path, filename: &str) -> Self {
        HlcPersistence {
            path: daemon_dir.join(filename),
        }
    }

    /// Get the path to the HLC file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read HLC from disk, returns None if file doesn't exist or is invalid
    pub fn read(&self) -> Option<Hlc> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        content.trim().parse().ok()
    }

    /// Write HLC to disk with fsync for durability
    pub fn write(&self, hlc: Hlc) -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(&self.path)?;
        write!(file, "{}", hlc)?;
        file.sync_all()?;
        Ok(())
    }

    /// Update HLC only if the given value is greater (high-water mark pattern)
    pub fn update(&self, hlc: Hlc) -> Result<()> {
        if let Some(current) = self.read() {
            if hlc > current {
                self.write(hlc)?;
            }
        } else {
            self.write(hlc)?;
        }
        Ok(())
    }
}
```

**Verification**:
- [ ] `cargo check -p wok-cli`
- [ ] `cargo test -p wok-cli`

---

### Phase 2: Add Constants and Convenience Constructors

**Goal**: Define standard HLC filenames as constants and add convenience constructors.

**Files**:
- Update `crates/cli/src/commands/hlc_persistence.rs`

**Implementation**:

```rust
impl HlcPersistence {
    /// Filename for locally-generated HLC high-water mark
    pub const LAST_HLC: &'static str = "last_hlc.txt";

    /// Filename for server-confirmed HLC high-water mark
    pub const SERVER_HLC: &'static str = "server_hlc.txt";

    /// Create persistence for local HLC (last_hlc.txt)
    pub fn last(daemon_dir: &Path) -> Self {
        Self::new(daemon_dir, Self::LAST_HLC)
    }

    /// Create persistence for server HLC (server_hlc.txt)
    pub fn server(daemon_dir: &Path) -> Self {
        Self::new(daemon_dir, Self::SERVER_HLC)
    }
}
```

**Verification**:
- [ ] `cargo check -p wok-cli`

---

### Phase 3: Replace Existing Functions with Thin Wrappers

**Goal**: Replace the 8 duplicate functions with thin wrappers that delegate to HlcPersistence. This maintains backward compatibility while centralizing logic.

**Files**:
- Update `crates/cli/src/commands/mod.rs` (lines 51-118)

**Before** (showing one pair, the other is identical):
```rust
pub fn get_last_hlc_path(daemon_dir: &Path) -> std::path::PathBuf {
    daemon_dir.join("last_hlc.txt")
}

pub fn read_last_hlc(daemon_dir: &Path) -> Option<Hlc> {
    let path = get_last_hlc_path(daemon_dir);
    let content = std::fs::read_to_string(&path).ok()?;
    content.trim().parse().ok()
}

pub fn write_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    use std::io::Write;
    let path = get_last_hlc_path(daemon_dir);
    let mut file = std::fs::File::create(&path)?;
    write!(file, "{}", hlc)?;
    file.sync_all()?;
    Ok(())
}

pub fn update_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    if let Some(current) = read_last_hlc(daemon_dir) {
        if hlc > current { write_last_hlc(daemon_dir, hlc)?; }
    } else { write_last_hlc(daemon_dir, hlc)?; }
    Ok(())
}
```

**After**:
```rust
pub fn get_last_hlc_path(daemon_dir: &Path) -> std::path::PathBuf {
    HlcPersistence::last(daemon_dir).path().to_path_buf()
}

pub fn read_last_hlc(daemon_dir: &Path) -> Option<Hlc> {
    HlcPersistence::last(daemon_dir).read()
}

pub fn write_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    HlcPersistence::last(daemon_dir).write(hlc)
}

pub fn update_last_hlc(daemon_dir: &Path, hlc: Hlc) -> Result<()> {
    HlcPersistence::last(daemon_dir).update(hlc)
}

// Similarly for server_hlc using HlcPersistence::server(daemon_dir)
```

**Verification**:
- [ ] `cargo check -p wok-cli`
- [ ] `cargo test -p wok-cli`
- [ ] `make spec-cli` - All existing tests should pass unchanged

---

### Phase 4: Migrate Call Sites to Direct HlcPersistence Usage

**Goal**: Update daemon/runner.rs to use HlcPersistence directly, making the code clearer.

**Files**:
- Update `crates/cli/src/daemon/runner.rs`

**Before** (line 525-536):
```rust
let _ = crate::commands::update_server_hlc(daemon_dir, op.id);
let _ = crate::commands::update_last_hlc(daemon_dir, op.id);
```

**After**:
```rust
use crate::commands::HlcPersistence;

let last_hlc = HlcPersistence::last(daemon_dir);
let server_hlc = HlcPersistence::server(daemon_dir);

let _ = server_hlc.update(op.id);
let _ = last_hlc.update(op.id);
```

**Call sites to update**:
| Location | Current Usage | New Usage |
|----------|---------------|-----------|
| runner.rs:246 | `read_server_hlc(daemon_dir)` | `HlcPersistence::server(daemon_dir).read()` |
| runner.rs:525 | `update_server_hlc(daemon_dir, op.id)` | `server_hlc.update(op.id)` |
| runner.rs:527 | `update_last_hlc(daemon_dir, op.id)` | `last_hlc.update(op.id)` |
| runner.rs:535-536 | Both update functions | Both persistence updates |
| runner.rs:629-630 | Both update functions | Both persistence updates |
| runner.rs:671 | `read_server_hlc(daemon_dir)` | `HlcPersistence::server(daemon_dir).read()` |
| mod.rs:127 | `read_last_hlc(daemon_dir)` | Keep or migrate |
| mod.rs:185 | `update_last_hlc(&daemon_dir, hlc)` | Keep or migrate |

**Verification**:
- [ ] `cargo check -p wok-cli`
- [ ] `cargo test -p wok-cli`
- [ ] `make spec-cli`

---

### Phase 5: Remove Deprecated Wrapper Functions

**Goal**: Remove the now-unused wrapper functions from commands/mod.rs.

**Files**:
- Update `crates/cli/src/commands/mod.rs` - remove 8 functions
- Update `crates/cli/src/daemon/runner_tests.rs` - if it uses any wrapper functions

**Functions to remove**:
- `get_last_hlc_path`
- `get_server_hlc_path`
- `read_last_hlc`
- `read_server_hlc`
- `write_last_hlc`
- `write_server_hlc`
- `update_last_hlc`
- `update_server_hlc`

**Verification**:
- [ ] `cargo check -p wok-cli`
- [ ] `cargo test -p wok-cli`
- [ ] `make spec-cli`
- [ ] `make spec-remote`

## Key Implementation Details

### High-Water Mark Pattern

The `update` method implements a high-water mark pattern: it only persists a new HLC if it's greater than the current stored value. This is critical for consistency:

```rust
pub fn update(&self, hlc: Hlc) -> Result<()> {
    if let Some(current) = self.read() {
        if hlc > current {
            self.write(hlc)?;
        }
    } else {
        self.write(hlc)?;
    }
    Ok(())
}
```

This ensures monotonic progress of HLC values on disk.

### Durability via fsync

The `write` method calls `file.sync_all()` to ensure data is flushed to disk before returning. This provides durability guarantees for crash recovery.

### HLC File Format

HLCs are stored as plain text in the format `{wall_ms}-{counter}-{node_id}`:
- Example: `1705932000000-42-12345`
- Parsed via the `FromStr` implementation in `wok_core::Hlc`

### Two HLC Types

1. **last_hlc.txt**: Tracks the highest HLC seen locally, used to generate monotonically increasing IDs
2. **server_hlc.txt**: Tracks the highest HLC confirmed by the server, used for sync cursor

## Verification Plan

### Unit Tests

Add tests to `crates/cli/src/commands/hlc_persistence.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wok_core::HlcClock;

    #[test]
    fn test_read_write_roundtrip() {
        let dir = TempDir::new().unwrap();
        let persistence = HlcPersistence::new(dir.path(), "test_hlc.txt");
        let clock = HlcClock::new(123);
        let hlc = clock.now();

        persistence.write(hlc).unwrap();
        let read_back = persistence.read().unwrap();

        assert_eq!(hlc, read_back);
    }

    #[test]
    fn test_read_nonexistent_returns_none() {
        let dir = TempDir::new().unwrap();
        let persistence = HlcPersistence::new(dir.path(), "nonexistent.txt");

        assert!(persistence.read().is_none());
    }

    #[test]
    fn test_update_only_advances() {
        let dir = TempDir::new().unwrap();
        let persistence = HlcPersistence::new(dir.path(), "test_hlc.txt");
        let clock = HlcClock::new(123);

        let hlc1 = clock.now();
        let hlc2 = clock.now(); // hlc2 > hlc1

        persistence.update(hlc2).unwrap();
        persistence.update(hlc1).unwrap(); // Should not update (hlc1 < hlc2)

        assert_eq!(persistence.read().unwrap(), hlc2);
    }
}
```

### Integration Verification

1. **All existing tests pass**: `cargo test -p wok-cli`
2. **CLI specs pass**: `make spec-cli`
3. **Remote specs pass**: `make spec-remote`
4. **Quality checks**: `cargo clippy -p wok-cli`
5. **Format check**: `cargo fmt --check`

### Manual Verification

1. Start daemon, create issues, verify `last_hlc.txt` is written
2. Connect to remote, sync, verify `server_hlc.txt` is written
3. Restart daemon, verify HLC values persist and sync resumes correctly
