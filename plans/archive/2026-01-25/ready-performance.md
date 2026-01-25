# Ready Command Performance Optimization

**Root Feature:** `wok-fb59`

**Root Issue:** `wk ready` takes >500ms (often 3+ seconds) on a database with only 664 issues.

## Overview

The `wk ready` command suffers from N+1 query patterns that cause performance to degrade linearly (or worse) with database size. The current implementation:

1. Fetches all todo issues
2. For **each** issue, queries labels during filtering (N queries)
3. For **each** sort comparison, queries labels again (O(N log N) queries)
4. Queries the entire dependency graph for blocked status

On a database with 664 issues, this results in **hundreds of database queries** for a single command. The fix is to batch these operations.

## Project Structure

Key files to modify:

```
crates/cli/src/
├── db/
│   ├── labels.rs         # Add batch label fetching
│   └── issues.rs         # Possibly optimize blocked query
├── commands/
│   └── ready.rs          # Use batch fetching, pre-compute sort keys
checks/specs/cli/unit/
└── ready.bats            # Add performance regression test
checks/benchmarks/
├── scenarios/
│   └── ready.sh          # Stress benchmarks (extend if exists)
└── setup/
    └── generate_db.sh    # Ensure stress size available
```

## Dependencies

No new external dependencies. Uses existing:
- `rusqlite` for database queries
- `std::collections::HashMap` for label caching

## Implementation Phases

### Phase 1: Add Batch Label Fetching to Database

**Goal:** Create a single query to fetch labels for multiple issues at once.

**Files:** `crates/cli/src/db/labels.rs`

**New method:**
```rust
/// Get labels for multiple issues in a single query.
/// Returns a map from issue_id to labels vector.
pub fn get_labels_batch(&self, issue_ids: &[&str]) -> Result<HashMap<String, Vec<String>>> {
    if issue_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build query with placeholders
    let placeholders: Vec<_> = (1..=issue_ids.len()).map(|i| format!("?{}", i)).collect();
    let sql = format!(
        "SELECT issue_id, label FROM labels WHERE issue_id IN ({}) ORDER BY issue_id, label",
        placeholders.join(", ")
    );

    let mut stmt = self.conn.prepare(&sql)?;
    let params: Vec<&dyn rusqlite::ToSql> = issue_ids
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let mut rows = stmt.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        let issue_id: String = row.get(0)?;
        let label: String = row.get(1)?;
        map.entry(issue_id).or_default().push(label);
    }

    Ok(map)
}
```

**Verification:**
- Add unit test in `labels_tests.rs`
- `cargo test --package wok-cli -- labels`

### Phase 2: Pre-fetch Labels Before Filtering

**Goal:** Eliminate N+1 queries during label filtering by fetching all labels upfront.

**Files:** `crates/cli/src/commands/ready.rs`

**Current code (lines 126-140):**
```rust
let mut issues = db.list_issues(Some(Status::Todo), None, None)?;

// Apply type filter
if type_groups.is_some() {
    issues.retain(|issue| matches_filter_groups(&type_groups, || issue.issue_type));
}

// Apply label filter - N+1 QUERIES HERE
if label_groups.is_some() {
    issues.retain(|issue| {
        let issue_labels = db.get_labels(&issue.id).unwrap_or_default();  // <-- N queries
        matches_label_groups(&label_groups, &issue_labels)
    });
}
```

**New approach:**
```rust
let mut issues = db.list_issues(Some(Status::Todo), None, None)?;

// Apply type filter first (no DB access needed)
if type_groups.is_some() {
    issues.retain(|issue| matches_filter_groups(&type_groups, || issue.issue_type));
}

// Pre-fetch all labels for remaining issues in one query
let issue_ids: Vec<&str> = issues.iter().map(|i| i.id.as_str()).collect();
let labels_map = db.get_labels_batch(&issue_ids)?;

// Apply label filter using pre-fetched map
if label_groups.is_some() {
    issues.retain(|issue| {
        let issue_labels = labels_map.get(&issue.id).map(|v| v.as_slice()).unwrap_or(&[]);
        matches_label_groups(&label_groups, issue_labels)
    });
}
```

This reduces N queries to 1 query.

**Verification:**
- `make spec ARGS='--file cli/unit/ready.bats'`
- Manual timing test

### Phase 3: Pre-compute Sort Keys

**Goal:** Eliminate O(N log N) label queries during sorting by using the pre-fetched label map.

**Files:** `crates/cli/src/commands/ready.rs`

**Current code (lines 171-193):**
```rust
ready_issues.sort_by(|a, b| {
    // ... recency check ...
    (true, true) => {
        let tags_a = db.get_labels(&a.id).unwrap_or_default();  // <-- Called per comparison!
        let tags_b = db.get_labels(&b.id).unwrap_or_default();  // <-- O(N log N) total
        let priority_a = Database::priority_from_tags(&tags_a);
        let priority_b = Database::priority_from_tags(&tags_b);
        // ...
    }
    // ...
});
```

**New approach:**

The `labels_map` from Phase 2 is already available. Use it in the sort closure:

```rust
ready_issues.sort_by(|a, b| {
    let a_recent = a.created_at >= cutoff;
    let b_recent = b.created_at >= cutoff;

    match (a_recent, b_recent) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (true, true) => {
            // Use pre-fetched labels - no DB access
            let tags_a = labels_map.get(&a.id).map(|v| v.as_slice()).unwrap_or(&[]);
            let tags_b = labels_map.get(&b.id).map(|v| v.as_slice()).unwrap_or(&[]);
            let priority_a = Database::priority_from_tags(tags_a);
            let priority_b = Database::priority_from_tags(tags_b);
            match priority_a.cmp(&priority_b) {
                std::cmp::Ordering::Equal => a.created_at.cmp(&b.created_at),
                other => other,
            }
        }
        (false, false) => a.created_at.cmp(&b.created_at),
    }
});
```

This reduces O(N log N) queries to 0 additional queries.

**Note:** `priority_from_tags` currently takes `&[String]`. Update signature to accept `&[impl AsRef<str>]` or just `&[String]` with a slice.

**Verification:**
- `make spec ARGS='--file cli/unit/ready.bats'`
- Manual timing test

### Phase 4: Use Labels Map for JSON Output

**Goal:** Eliminate 5 additional label queries during JSON output.

**Files:** `crates/cli/src/commands/ready.rs`

**Current code (lines 208-220):**
```rust
OutputFormat::Json => {
    let mut json_issues = Vec::new();
    for issue in &ready_issues {
        let labels = db.get_labels(&issue.id)?;  // <-- 5 more queries
        json_issues.push(ReadyIssueJson { /* ... */ labels });
    }
    // ...
}
```

**New approach:**
```rust
OutputFormat::Json => {
    let mut json_issues = Vec::new();
    for issue in &ready_issues {
        let labels = labels_map
            .get(&issue.id)
            .cloned()
            .unwrap_or_default();
        json_issues.push(ReadyIssueJson { /* ... */ labels });
    }
    // ...
}
```

**Verification:**
- `wk ready --json` should still output labels correctly
- `make spec ARGS='--filter "ready.*json"'`

### Phase 5: Benchmark and Validate

**Goal:** Verify performance improvements meet targets.

**Performance Targets:**

| Metric | Before | Target |
|--------|--------|--------|
| Query count | ~600+ | ~3-5 |
| Time (664 issues) | >500ms | <50ms |
| Time (20k issues) | unknown | <100ms |

**Validation Steps:**

```bash
# 1. Format and lint
cargo fmt && cargo clippy

# 2. Run tests
cargo test --package wok-cli

# 3. Run specs
make spec ARGS='--file cli/unit/ready.bats'

# 4. Manual performance test
time wk ready  # Should be <50ms

# 5. Optional: Run benchmarks on stress database
cd checks/benchmarks
./setup/generate_db.sh stress  # 20k issues
./scenarios/ready.sh
```

## Key Implementation Details

### Query Count Analysis

**Before optimization:**
- 1 query: `list_issues(Todo)`
- N queries: `get_labels()` during filtering (where N = todo issue count)
- O(N log N) queries: `get_labels()` during sorting
- 1 query: `get_blocked_issue_ids()` recursive CTE
- 5 queries: `get_labels()` during JSON output

**Total:** ~1 + N + N*log(N) + 1 + 5 ≈ 600+ queries for 100 todo issues

**After optimization:**
- 1 query: `list_issues(Todo)`
- 1 query: `get_labels_batch()` for all filtered issues
- 0 queries: sorting uses pre-fetched map
- 1 query: `get_blocked_issue_ids()` (unchanged)
- 0 queries: JSON output uses pre-fetched map

**Total:** 3 queries regardless of issue count

### Label Map Scope

The `labels_map` must remain in scope through filtering, sorting, and output formatting. Structure the code so the map is created early and passed by reference:

```rust
// Create map after type filtering (before label filtering)
let issue_ids: Vec<&str> = issues.iter().map(|i| i.id.as_str()).collect();
let labels_map = db.get_labels_batch(&issue_ids)?;

// All subsequent operations use &labels_map
```

### Blocked Query Consideration

The `get_blocked_issue_ids()` query runs a recursive CTE over the entire dependency graph. This is a single query with fixed cost, not an N+1 pattern. It could be optimized further by:
1. Adding an issue_id filter to the CTE
2. Caching blocked status

However, since it's already a single query, this optimization is lower priority and can be deferred unless benchmarks show it's a bottleneck.

## Verification Plan

### Unit Tests
```bash
cargo test --package wok-cli -- labels
cargo test --package wok-cli -- ready
```

### Spec Tests
```bash
make spec ARGS='--file cli/unit/ready.bats'
```

### Manual Performance Test
```bash
# In a repo with many issues
time wk ready           # Should complete in <50ms
time wk ready --json    # Same performance
time wk ready --label X # Same performance
```

### Regression Test
Add to `checks/specs/cli/unit/ready.bats`:
```bash
@test "ready: completes quickly with many issues" {
    # Create 200 issues
    for i in {1..200}; do
        run wk add "Issue $i"
    done

    # Should complete in under 1 second
    run timeout 1 wk ready
    assert_success
}
```

## Checklist

- [ ] Phase 1: Add `get_labels_batch()` to `db/labels.rs`
- [ ] Phase 2: Pre-fetch labels before filtering in `ready.rs`
- [ ] Phase 3: Use pre-fetched labels in sort closure
- [ ] Phase 4: Use pre-fetched labels in JSON output
- [ ] Phase 5: Validate performance meets targets
- [ ] Update `priority_from_tags` if needed for slice compatibility
- [ ] Add regression test for performance
