# File Attachments Implementation Plan

## Overview

Add support for file attachments (up to 100MB) to tickets in wk. Files are stored using content-addressable storage (CAS), referenced by SHA256 hash. Operations remain lightweight (just metadata), while file content syncs separately through a dedicated blob transfer protocol.

## Project Structure

```
crates/
├── core/src/
│   ├── blob.rs              # Content-addressable blob storage
│   ├── blob_tests.rs
│   ├── attachment.rs        # Attachment metadata types
│   ├── attachment_tests.rs
│   ├── op.rs                # + AddAttachment/RemoveAttachment payloads
│   ├── protocol.rs          # + Blob transfer messages
│   └── db.rs                # + attachments table, blobs table
├── cli/src/
│   ├── cmd_attach.rs        # wk attach <id> <file>
│   ├── cmd_attachments.rs   # wk attachments <id>
│   ├── cmd_export.rs        # wk export <id> <hash> [--output path]
│   └── sync/
│       └── blob_sync.rs     # Blob upload/download logic
└── remote/src/
    ├── blob_store.rs        # Server-side blob storage
    └── server.rs            # + blob transfer handlers

.wok/
├── blobs/                   # Local blob storage (CAS)
│   ├── aa/                  # Sharded by first 2 chars of hash
│   │   └── aabbccdd...      # Full hash as filename
│   └── ...
├── issues.db
└── config.toml
```

## Dependencies

- **sha2**: SHA256 hashing for content addressing
- **hex**: Hex encoding for hash strings
- No new external dependencies for core functionality

## Implementation Phases

### Phase 1: Core Blob Storage

**Goal**: Content-addressable local blob storage with atomic writes.

**Files to create/modify**:
- `crates/core/src/blob.rs` - New file
- `crates/core/src/blob_tests.rs` - New file
- `crates/core/src/lib.rs` - Export blob module

**Schema additions** (`crates/core/src/db.rs`):

```sql
-- Track which blobs exist locally
CREATE TABLE blobs (
    hash TEXT PRIMARY KEY,         -- SHA256 hex
    size INTEGER NOT NULL,         -- Bytes
    created_at TEXT NOT NULL
);
CREATE INDEX idx_blobs_size ON blobs(size);

-- Attachment metadata per issue
CREATE TABLE attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    filename TEXT NOT NULL,        -- Original filename
    hash TEXT NOT NULL,            -- SHA256 of content
    size INTEGER NOT NULL,         -- Bytes
    mime_type TEXT,                -- Optional
    status TEXT NOT NULL,          -- Status when attached
    created_at TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id),
    UNIQUE(issue_id, hash)         -- No duplicate attachments
);
CREATE INDEX idx_attachments_issue ON attachments(issue_id);
CREATE INDEX idx_attachments_hash ON attachments(hash);
```

**BlobStore API**:

```rust
pub struct BlobStore {
    root: PathBuf,  // .wok/blobs/
}

impl BlobStore {
    /// Store content, returns hash
    pub fn store(&self, content: &[u8]) -> Result<String>;

    /// Store from reader (streaming, for large files)
    pub fn store_stream(&self, reader: impl Read) -> Result<String>;

    /// Get blob content by hash
    pub fn get(&self, hash: &str) -> Result<Vec<u8>>;

    /// Get blob reader (streaming)
    pub fn get_reader(&self, hash: &str) -> Result<impl Read>;

    /// Check if blob exists locally
    pub fn exists(&self, hash: &str) -> bool;

    /// Get blob size (without reading)
    pub fn size(&self, hash: &str) -> Result<u64>;

    /// Delete blob (for GC)
    pub fn delete(&self, hash: &str) -> Result<()>;
}
```

**Storage layout**: Shard by first 2 hex chars to avoid too many files in one directory:
```
blobs/aa/aabbccdd1122...
blobs/ff/ffee0011...
```

**Atomic writes**: Write to temp file, fsync, rename to final path.

**Milestone**: `cargo test -p wk-core blob` passes with unit tests for store/get/exists.

---

### Phase 2: Operations & Merge Logic

**Goal**: Add attachment operations to the op system with HLC-based conflict resolution.

**Files to modify**:
- `crates/core/src/op.rs` - Add payloads
- `crates/core/src/merge.rs` - Add merge logic
- `crates/core/src/attachment.rs` - New file for Attachment type

**New operation payloads**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpPayload {
    // ... existing variants ...

    /// Add an attachment to an issue
    AddAttachment {
        issue_id: String,
        filename: String,
        hash: String,        // SHA256
        size: u64,
        mime_type: Option<String>,
        status: Status,      // Status when attached
    },

    /// Remove an attachment from an issue
    RemoveAttachment {
        issue_id: String,
        hash: String,        // Identified by hash
    },
}
```

**Merge semantics**:
- `AddAttachment`: Always succeeds (idempotent by unique constraint)
- `RemoveAttachment`: Always succeeds (idempotent)
- Concurrent add+remove of same hash: Last HLC wins (same as label semantics)

**Attachment type**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub issue_id: String,
    pub filename: String,
    pub hash: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
}
```

**Milestone**: Operations can be created, serialized, and merged correctly.

---

### Phase 3: CLI Commands

**Goal**: User-facing commands for managing attachments.

**Files to create/modify**:
- `crates/cli/src/cmd_attach.rs` - New file
- `crates/cli/src/cmd_attachments.rs` - New file
- `crates/cli/src/cmd_export.rs` - New file (or extend existing)
- `crates/cli/src/cli.rs` - Add subcommands

**Commands**:

```bash
# Attach a file to an issue
wk attach <issue-id> <file-path>
# Output: Attached "report.pdf" (2.4 MB) to prj-a3f2

# List attachments for an issue
wk attachments <issue-id>
# Output:
# prj-a3f2: Fix authentication
#   report.pdf      2.4 MB  aabbcc... (in_progress)
#   screenshot.png  156 KB  ddeeff... (todo)

# Export/download an attachment
wk export <issue-id> <filename-or-hash> [--output path]
# Output: Exported "report.pdf" to ./report.pdf

# Show attachment in `wk show`
wk show <issue-id>
# Output includes:
# Attachments:
#   report.pdf (2.4 MB)
#   screenshot.png (156 KB)
```

**Implementation notes**:
- `attach`: Read file, store in blob store, emit AddAttachment op
- `attachments`: Query attachments table, show with size formatting
- `export`: Lookup by filename or hash prefix, copy from blob store
- Size display: Human-readable (KB, MB, GB)

**Milestone**: Full local workflow works without sync.

---

### Phase 4: Sync Protocol Extension

**Goal**: Sync attachments between clients and server.

**Files to modify**:
- `crates/core/src/protocol.rs` - Add blob messages
- `crates/cli/src/sync/client.rs` - Blob sync logic
- `crates/cli/src/sync/blob_sync.rs` - New file
- `crates/remote/src/server.rs` - Blob handlers
- `crates/remote/src/blob_store.rs` - New file

**New protocol messages**:

```rust
pub enum ClientMessage {
    // ... existing ...

    /// Request a blob by hash
    BlobRequest { hash: String },

    /// Push blob content to server
    BlobPush {
        hash: String,
        size: u64,
        data: Vec<u8>,  // Base64 in JSON
    },
}

pub enum ServerMessage {
    // ... existing ...

    /// Blob content response
    BlobResponse {
        hash: String,
        size: u64,
        data: Vec<u8>,  // Base64 in JSON, or null if not found
    },

    /// Acknowledge blob receipt
    BlobAck { hash: String },

    /// Blob not found
    BlobNotFound { hash: String },
}
```

**Sync flow**:

1. **On AddAttachment op received**: Check if blob exists locally
   - If yes: Done
   - If no: Queue blob request

2. **Blob request**: Client sends `BlobRequest`, server responds with `BlobResponse`
   - Stream large files in chunks (see Phase 5)

3. **On local attach**: After storing blob locally, push to server if connected
   - Queue push if offline

4. **Lazy sync**: Don't download blobs until needed (e.g., `wk export`)
   - Track "missing" blobs in a separate table or in-memory set

**Server blob storage**: Same CAS structure as client, stored in `--data` directory.

**Milestone**: Attachments sync between two clients via WebSocket server.

---

### Phase 5: Large File Support

**Goal**: Efficient handling of files up to 100MB.

**Chunked transfers**:
```rust
// For files > 4MB, use chunked transfer
const CHUNK_SIZE: usize = 4 * 1024 * 1024;  // 4MB chunks

pub enum ClientMessage {
    // ... existing ...

    /// Request a chunk of a blob
    ChunkRequest {
        hash: String,
        offset: u64,
        length: u32,
    },

    /// Push a chunk to server
    ChunkPush {
        hash: String,
        offset: u64,
        data: Vec<u8>,
        final_chunk: bool,
    },
}

pub enum ServerMessage {
    // ... existing ...

    ChunkResponse {
        hash: String,
        offset: u64,
        data: Vec<u8>,
    },

    ChunkAck {
        hash: String,
        offset: u64,
    },
}
```

**Progress reporting**:
```rust
pub trait BlobProgress {
    fn on_progress(&self, hash: &str, bytes_done: u64, total: u64);
}
```

**Resumable transfers**: Track partial downloads/uploads, resume from last chunk.

**Memory efficiency**: Stream directly to/from disk, never hold full file in memory.

**Milestone**: 100MB file attaches and syncs without memory issues.

---

### Phase 6: Git Remote Support

**Goal**: Support attachments with git-based remotes.

**Options considered**:
1. **Git LFS**: Standard large file storage, widely supported
2. **Separate branch**: Store blobs in refs/wk/blobs
3. **Annex-style**: Track hashes in git, content stored separately

**Recommended: Git LFS integration**

```toml
# .wok/config.toml
[remote]
url = "git:."
blob_storage = "lfs"  # or "external" for external blob server
```

**Implementation**:
- On attach: Also `git lfs track` the blob
- On sync: `git lfs push/pull` handles blob transfer
- Blobs stored in `.wok/blobs/` locally, LFS moves to remote

**Alternative for non-LFS repos**: External blob server URL in config:
```toml
[remote]
url = "git:."
blob_url = "https://blobs.example.com"  # Separate blob server
```

**Milestone**: Attachments work with `git:.` remote type.

## Key Implementation Details

### Content Addressing

All blobs are identified by their SHA256 hash:
```rust
fn compute_hash(content: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}
```

Benefits:
- Automatic deduplication (same file attached to multiple issues = one blob)
- Integrity verification (hash mismatch = corrupted transfer)
- Efficient sync (only transfer blobs we don't have)

### Atomic Blob Writes

To prevent corruption from interrupted writes:
```rust
fn store_atomic(&self, hash: &str, content: &[u8]) -> Result<()> {
    let final_path = self.blob_path(hash);
    let temp_path = final_path.with_extension("tmp");

    let mut file = File::create(&temp_path)?;
    file.write_all(content)?;
    file.sync_all()?;

    fs::rename(temp_path, final_path)?;
    Ok(())
}
```

### Garbage Collection

Blobs without references can be cleaned up:
```rust
/// Find blobs not referenced by any attachment
pub fn orphaned_blobs(&self) -> Result<Vec<String>> {
    // Query blobs table LEFT JOIN attachments
    // Return hashes with no attachment references
}

/// Remove orphaned blobs
pub fn gc(&self) -> Result<usize> {
    let orphans = self.orphaned_blobs()?;
    for hash in &orphans {
        self.blob_store.delete(hash)?;
    }
    Ok(orphans.len())
}
```

### Lazy Blob Fetching

Don't download blobs automatically - fetch on demand:
```rust
// In merge.rs, when applying AddAttachment
fn apply_add_attachment(&mut self, op: &Op) -> Result<()> {
    // Store metadata in attachments table
    // Mark blob as "needed" if not present locally
    // Don't block on blob download
}

// In cmd_export.rs
fn export_attachment(&self, issue_id: &str, hash: &str) -> Result<()> {
    if !self.blob_store.exists(hash) {
        // Fetch from remote synchronously
        self.sync_client.fetch_blob(hash)?;
    }
    // Copy from blob store to output
}
```

### MIME Type Detection

Detect MIME type for better UX:
```rust
fn detect_mime_type(path: &Path, content: &[u8]) -> Option<String> {
    // 1. Try extension mapping
    // 2. Try magic bytes detection
    // 3. Return None if unknown
}
```

## Verification Plan

### Unit Tests
- Blob store: store/get/exists/delete with various sizes
- Hash computation: known test vectors
- Atomic writes: Interrupt simulation
- Operation serialization: Round-trip AddAttachment/RemoveAttachment
- Merge logic: Concurrent add/remove scenarios

### Integration Tests
- CLI workflow: attach → list → export
- Sync: Two clients, one attaches, other receives
- Large files: 100MB attachment round-trip
- Offline: Attach while offline, sync when reconnected
- Deduplication: Same file attached to multiple issues

### Manual Testing
```bash
# Local workflow
wk init
wk new "Test attachments"
wk attach prj-xxxx ./large-file.zip
wk attachments prj-xxxx
wk export prj-xxxx large-file.zip --output /tmp/out.zip
diff ./large-file.zip /tmp/out.zip  # Should match

# Sync workflow (two terminals)
# Terminal 1: wk-remote --bind 127.0.0.1:7890 --data /tmp/server
# Terminal 2: wk init --remote ws://127.0.0.1:7890
# Terminal 3: wk init --remote ws://127.0.0.1:7890 (different dir)
# Attach in T2, verify appears in T3
```

### Performance Testing
- Time to attach 1MB, 10MB, 100MB files
- Memory usage during large file transfer
- Sync latency with pending blob requests
