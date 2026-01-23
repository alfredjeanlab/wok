# Architecture and Performance Analysis Report

**Date:** 2026-01-22
**Scope:** wok codebase - distributed issue tracking system
**Focus:** Architectural limitations and pervasive abstractions affecting performance

---

## Executive Summary

The wok codebase demonstrates a well-architected distributed issue tracker with thoughtful separation of concerns. However, several performance bottlenecks emerge as the system scales:

- **String serialization overhead** for HLC timestamps
- **O(n) oplog reads** at initialization and sync
- **Complex join queries** without optimization for large datasets
- **Per-operation fsync** causing I/O bottlenecks
- **Graph traversal** on every dependency operation
- **Full file rewrites** for queue operations

These issues are not critical for small projects (<1,000 issues) but become significant as databases grow.

---

## 1. Directory Structure and Organization

### Current Architecture

```
wok/
├── crates/
│   ├── core/     (1,500+ LOC) - CRDT, HLC, operations, merge logic
│   ├── cli/      (6,000+ LOC) - Commands, database, sync, daemon
│   └── remote/   (1,200+ LOC) - WebSocket server for fleet sync
├── checks/       - Integration tests
└── plans/        - Planning/tracking files
```

### Observations

**Strength:** Clear separation with wk-core providing sync primitives reused by both CLI and remote server.

**Limitation:** The CLI crate has grown to 6,000+ LOC and handles too many responsibilities:
- Command parsing and execution
- SQLite database operations
- Sync client and daemon logic
- Filter expression parsing
- Display formatting

---

## 2. Core Data Structures

### 2.1 Hybrid Logical Clock (HLC)

**Location:** `crates/core/src/hlc.rs`

```rust
pub struct Hlc {
    pub wall_ms: u64,   // Wall clock time
    pub counter: u32,   // Logical counter
    pub node_id: u32,   // Node identifier
}
```

**Performance Issue: String-Based Serialization**

Every HLC timestamp is stored as text (e.g., "1234567890-42-1001"):
- Text parsing on every database read/write/sync operation
- Located in `crates/core/src/db.rs:118-129` (`parse_hlc_opt` function)
- Every oplog read loads all HLC values from text: `crates/core/src/oplog.rs:46`

**Recommendation:** Binary HLC encoding (12 bytes fixed) would reduce storage and eliminate parsing overhead.

### 2.2 Operation Log (Oplog)

**Location:** `crates/core/src/oplog.rs`

```rust
pub struct Oplog {
    path: PathBuf,
    seen_ids: HashSet<Hlc>,  // All operation IDs loaded into memory
}
```

**Performance Issue #1: O(n) Reads on Initialization**

- `Oplog::open()` reads entire JSONL file and parses every operation (`oplog.rs:37-49`)
- `ops_since()` also reads entire file and filters in memory (`oplog.rs:91-116`)
- No indexing or seeking to skip old operations
- **Impact:** Sync becomes slower linearly with operation history

**Performance Issue #2: Immediate fsync() on Every Append**

- Line `oplog.rs:81` triggers full disk synchronization per operation
- On batch operations, this creates significant I/O overhead

**Recommendations:**
- Add memory-mapped index file for operation_id → file_offset
- Batch fsync or use write-ahead log with periodic commits

### 2.3 Database Schema

**Locations:** `crates/core/src/db.rs`, `crates/cli/src/db/`

**Architectural Limitation: Dual Database Design**

The codebase maintains TWO separate databases:
1. `wk_core::Database` - Minimal schema for sync operations
2. `wkrs::db::Database` - Enhanced schema for CLI with additional fields

This creates schema synchronization challenges and duplicated logic.

---

## 3. Query Performance Hotspots

### 3.1 List Issues Query (HIGH IMPACT)

**Location:** `crates/cli/src/db/issues.rs:165-246`

```sql
SELECT DISTINCT i.id, ...
FROM issues i
LEFT JOIN notes n ON n.issue_id = i.id
LEFT JOIN labels l ON l.issue_id = i.id
LEFT JOIN links lk ON lk.issue_id = i.id
WHERE ... (6 different LIKE conditions with OR)
```

**Problems:**
- Complex subquery for `closed_at` computed per row (lines 174-181)
- LEFT JOINs multiply result set before DISTINCT
- String parsing for every field (type_str, status_str, etc.)

### 3.2 Cycle Detection (HIGH IMPACT)

**Location:** `crates/cli/src/db/deps.rs:53-68`

```rust
// Runs recursive CTE for EVERY dependency addition
fn would_create_cycle(...) {
    // WITH RECURSIVE without limits can traverse entire dependency graph
}
```

**Problem:** For large projects with thousands of dependencies, adding a single dep becomes O(n).

### 3.3 Queue Remove First (MEDIUM IMPACT)

**Location:** `crates/cli/src/sync/queue.rs:114-130`

**Problem:** Full file rewrite on every call:
- Reads entire queue into memory
- Filters out first item
- Rewrites all remaining operations
- Compounds quickly during batch syncing

---

## 4. Concurrency and Async Patterns

### 4.1 Daemon State Management

**Location:** `crates/cli/src/daemon/runner.rs`

**Issue: Complex State in Async Context**

```rust
// Lines 191-195: Multiple clones before async blocks
let db_path_clone = db_path.clone();
let oplog_path_clone = client_oplog_path.clone();
let sync_config_clone = sync_config.clone();
let queue_path_clone = queue_path.clone();
```

PathBuf and SyncConfig cloned on every heartbeat/sync iteration. Could use `Arc<>` for shared ownership instead.

### 4.2 Transaction Handling

**Location:** `crates/core/src/merge.rs:184-210`

```rust
fn apply_set_status(...) {
    let issue = self.get_issue(issue_id)?;  // Query 1
    // ...
    self.conn.execute("UPDATE issues SET status = ?1");  // Query 2
    self.conn.execute("UPDATE issues SET last_status_hlc = ?1");  // Query 3
}
```

**Problem:** These three operations should be atomic but aren't. No explicit transaction management visible.

---

## 5. Memory Usage Patterns

### 5.1 Unbounded HashSets

**Location:** `crates/core/src/oplog.rs:26`

```rust
seen_ids: HashSet<Hlc>  // Grows unbounded with operation count
```

For 100,000 operations: ~3.2 MB (Hlc is 16 bytes + hash overhead). No eviction or cleanup strategy.

### 5.2 Full Result Collection

**Location:** `crates/cli/src/db/issues.rs:221-243`

`list_issues()` collects ALL matching issues into Vec in memory. No pagination or streaming results. With 100,000 issues, entire result set allocated at once.

---

## 6. CRDT/Merge Limitations

### 6.1 Last-Write-Wins (LWW) for All Fields

**Location:** `crates/core/src/merge.rs`

All field updates (SetStatus, SetTitle, SetType) use HLC to determine winner. If two clients update same field simultaneously, one silently loses data with no conflict tracking.

### 6.2 No Conflict Resolution UI

No events logged for audit trail when writes are overwritten. Users unaware when their changes are superseded.

---

## 7. Search Scalability

### 7.1 LIKE-Based Full-Text Search

**Location:** `crates/cli/src/db/issues.rs:251-303`

```sql
-- Uses LIKE with escaped wildcards, no FTS
-- 7-way LEFT JOIN without optimization
-- No ranking or relevance sorting
```

Must scan entire database; cannot use indexes efficiently.

---

## 8. Index Analysis

### Current Indexes

```sql
-- From crates/core/src/db.rs:74-80
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_type ON issues(type);
CREATE INDEX idx_deps_to ON deps(to_id);
CREATE INDEX idx_deps_rel ON deps(rel);
CREATE INDEX idx_labels_label ON labels(label);
CREATE INDEX idx_events_issue ON events(issue_id);
```

### Missing Indexes

- Composite index on `(status, type)` for filtered searches
- Composite index on `deps(to_id, rel)` for join queries
- Index on `deps.from_id` (used in recursive CTEs)

---

## 9. Sync Protocol Efficiency

### 9.1 Full Snapshot on Reconnect

**Location:** `crates/core/src/protocol.rs:62-74`

`SnapshotResponse` includes entire database state. For 100,000 issues, serialization overhead is massive. No incremental snapshot or chunking.

---

## 10. Prioritized Recommendations

### Priority 1: Critical (Scalability Blockers)

1. **Replace String HLC with Binary Encoding**
   - Change from "1234567890-42-1001" to 12-byte binary
   - Impact: 50% reduction in oplog file size, faster parsing
   - Location: `crates/core/src/hlc.rs`

2. **Implement Operation Log Indexing**
   - Add memory-mapped index file (operation_id → file_offset)
   - Skip old operations without full scan
   - Location: `crates/core/src/oplog.rs`

3. **Add Transaction Support for HLC Updates**
   - Wrap status + status_hlc updates in single transaction
   - Prevent dirty reads during sync
   - Location: `crates/core/src/merge.rs`

### Priority 2: Important (Performance)

4. **Optimize List/Search Queries**
   - Cache computed `closed_at` values in separate column
   - Add composite indexes on `(issue_id, rel)` and `(status, type)`
   - Location: `crates/cli/src/db/issues.rs`

5. **Implement Dependency Graph Caching**
   - Cache reachability matrix for cycle detection
   - Invalidate on dependency changes
   - Location: `crates/cli/src/db/deps.rs`

6. **Use Connection Pool**
   - Replace per-command database opens with pool
   - Location: `crates/cli/src/commands/mod.rs`

### Priority 3: Nice-to-Have (Maintainability)

7. **Add Binary Serialization Option**
   - Protocol v2 with bincode or msgpack
   - Location: `crates/core/src/protocol.rs`

8. **Implement Pagination**
   - Stream results instead of collecting all
   - Location: `crates/cli/src/commands/list.rs`

9. **Add Query Instrumentation**
   - Log slow queries (>100ms)
   - EXPLAIN QUERY PLAN for analysis
   - Location: `crates/cli/src/db/mod.rs`

---

## Performance Hotspot Summary

| Hotspot | Location | Impact | Problem |
|---------|----------|--------|---------|
| list_issues Query | `cli/src/db/issues.rs:165-246` | HIGH | Complex subquery per row, 6-way joins |
| Oplog Read | `core/src/oplog.rs:37-49` | MEDIUM | O(n) on every daemon start |
| Cycle Detection | `cli/src/db/deps.rs:53-68` | HIGH | Recursive CTE on every dep add |
| Queue Remove | `cli/src/sync/queue.rs:114-130` | MEDIUM | Full file rewrite per operation |
| JSON Serialization | Multiple | LOW-MEDIUM | Serde overhead on every operation |

---

## Conclusion

The architecture is sound for small-to-medium workloads but will face scaling challenges beyond ~10,000 issues. The primary concerns are:

1. **I/O patterns:** fsync per operation, full file rewrites
2. **Query complexity:** Unoptimized joins and subqueries
3. **Serialization:** String-based encoding for frequently-accessed data
4. **Memory:** Unbounded collections, no streaming

Addressing Priority 1 recommendations would significantly extend the system's scalability ceiling while maintaining the current clean architecture.
