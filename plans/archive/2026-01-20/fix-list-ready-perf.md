# Fix List/Ready Performance

**Root Feature:** `wok-4742`

## Overview

Add hard limits to `wk ready` and `wk list` commands to ensure consistent, fast performance even with extremely large issue databases. The `ready` command will be capped at 5 issues (since you can only actively work on a few things at once), while `list` will default to 100 results with an optional override. Both commands will be stress-tested against databases with 10k+ issues.

## Project Structure

Key files to modify:

```
crates/cli/src/commands/
├── ready.rs     # Add hard limit of 5
├── list.rs      # Add default limit of 100
checks/specs/cli/unit/
├── ready.bats   # Add limit behavior tests
├── list.bats    # Update for default limit
checks/benchmarks/
├── scenarios/
│   ├── ready.sh # New: stress benchmarks for ready
│   └── list.sh  # Extend with aggressive stress tests
└── setup/
    └── generate_db.sh  # Add stress size (20k issues)
```

## Dependencies

No new external dependencies required. Uses existing:
- `rusqlite` for database queries
- `hyperfine` for benchmarks (already in project)

## Implementation Phases

### Phase 1: Add Hard Limit to `wk ready`

**Goal:** Cap `ready` output at exactly 5 issues regardless of how many match.

**Files:**
- `crates/cli/src/commands/ready.rs`

**Current implementation (line ~190):** Returns all matching ready issues with no limit.

**Changes:**
1. Add constant `const MAX_READY_ISSUES: usize = 5;`
2. Apply truncation after sorting in `run_impl()` (after line 189)

**Code snippet:**
```rust
// In ready.rs, at top of file
const MAX_READY_ISSUES: usize = 5;

// After the sort_by closure (currently around line 189):
// Sort by created_at as tiebreaker
...
});

// Add this line immediately after sorting:
issues.truncate(MAX_READY_ISSUES);
```

**Verification:**
- `make spec ARGS='--file cli/unit/ready.bats'`
- Manual test: create 10+ ready issues, verify only 5 shown

### Phase 2: Add Default Limit to `wk list`

**Goal:** Default to 100 results when `--limit` not specified. Allow `--limit 0` for unlimited.

**Files:**
- `crates/cli/src/commands/list.rs`

**Current implementation (lines 227-230):**
```rust
if let Some(n) = args.limit {
    issues.truncate(n);
}
```
No default limit - returns all matching issues.

**Changes:**
1. Add constant `const DEFAULT_LIMIT: usize = 100;`
2. Apply default when `--limit` not specified
3. Treat `--limit 0` as unlimited (skip truncation)

**Code snippet:**
```rust
// In list.rs, at top of file
const DEFAULT_LIMIT: usize = 100;

// Replace the current limit logic (lines 227-230) with:
let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
if limit > 0 {
    issues.truncate(limit);
}
```

**Verification:**
- `make spec ARGS='--file cli/unit/list.bats'`
- Manual test: verify default shows ≤100, `--limit 0` shows all

### Phase 3: Update Specifications

**Goal:** Add test coverage for new limit behaviors.

**Files:**
- `checks/specs/cli/unit/ready.bats`
- `checks/specs/cli/unit/list.bats`

**New tests for `ready.bats`:**
```bash
@test "ready: returns at most 5 issues" {
    # Create 10 ready issues
    for i in {1..10}; do
        run wk add "Ready issue $i"
    done

    run wk ready
    assert_success

    # Count non-empty lines
    local count=$(echo "$output" | grep -c '^[^ ]')
    [[ $count -le 5 ]]
}

@test "ready: shows highest priority issues when >5 available" {
    # Create 3 high priority and 7 low priority
    for i in {1..3}; do
        run wk add "High priority $i" --label priority:0
    done
    for i in {1..7}; do
        run wk add "Low priority $i" --label priority:3
    done

    run wk ready
    assert_success
    # All 3 high priority should be shown
    assert_output --partial "High priority"
}
```

**New tests for `list.bats`:**
```bash
@test "list: defaults to 100 results" {
    # Setup: create 150 issues
    for i in {1..150}; do
        run wk add "Issue $i"
    done

    run wk list
    assert_success

    local count=$(echo "$output" | grep -c '^[^ ]')
    [[ $count -eq 100 ]]
}

@test "list: --limit 0 shows all results" {
    # Create 150 issues (from previous test or fresh)
    run wk list --limit 0
    assert_success

    local count=$(echo "$output" | grep -c '^[^ ]')
    [[ $count -eq 150 ]]
}

@test "list: explicit --limit overrides default" {
    run wk list --limit 50
    assert_success

    local count=$(echo "$output" | grep -c '^[^ ]')
    [[ $count -le 50 ]]
}
```

### Phase 4: Add Stress Benchmarks

**Goal:** Verify both commands complete quickly with large databases (10k-20k issues).

**Files:**
- `checks/benchmarks/scenarios/ready.sh` (new)
- `checks/benchmarks/scenarios/list.sh` (extend)
- `checks/benchmarks/setup/generate_db.sh` (add stress size)

**New database size in `generate_db.sh`:**
```bash
# Add new size option
stress)
    ISSUE_COUNT=20000
    LABELS_PER_ISSUE=5
    DEP_PERCENTAGE=25
    ASSIGNEE_COUNT=10
    ;;
```

**New file `ready.sh`:**
```bash
#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCH_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$BENCH_DIR/results"

mkdir -p "$RESULTS_DIR"

echo "=== Ready Command Stress Benchmarks ==="
echo ""

# With hard limit of 5, should be consistently fast regardless of DB size
echo "--- Basic Ready (default filters) ---"
hyperfine --warmup 3 --runs 20 \
    'wk ready' \
    --export-json "$RESULTS_DIR/ready-basic.json"

echo ""
echo "--- Ready with Assignee Filter ---"
hyperfine --warmup 3 --runs 20 \
    'wk ready -a alice' \
    'wk ready -a bob' \
    'wk ready --all-assignees' \
    --export-json "$RESULTS_DIR/ready-assignee.json"

echo ""
echo "--- Ready with Label Filter ---"
hyperfine --warmup 3 --runs 20 \
    'wk ready --label team:backend' \
    'wk ready --label priority:0' \
    --export-json "$RESULTS_DIR/ready-label.json"

echo ""
echo "--- Ready JSON Output ---"
hyperfine --warmup 3 --runs 20 \
    'wk ready --json' \
    --export-json "$RESULTS_DIR/ready-json.json"

echo ""
echo "=== Ready Benchmarks Complete ==="
```

**Extensions to `list.sh`:**
```bash
# Add stress test section
echo ""
echo "=== Stress Tests (should verify limits work) ==="

echo "--- Default Limit (100) on Large DB ---"
hyperfine --warmup 3 --runs 20 \
    'wk list' \
    --export-json "$RESULTS_DIR/list-stress-default.json"

echo "--- Various Explicit Limits ---"
hyperfine --warmup 3 --runs 10 \
    'wk list --limit 10' \
    'wk list --limit 50' \
    'wk list --limit 100' \
    'wk list --limit 500' \
    --export-json "$RESULTS_DIR/list-stress-limits.json"

echo "--- Unlimited (--limit 0) Baseline ---"
hyperfine --warmup 2 --runs 5 \
    'wk list --limit 0' \
    --export-json "$RESULTS_DIR/list-stress-unlimited.json"

echo "--- Filtered Queries on Large DB ---"
hyperfine --warmup 3 --runs 10 \
    'wk list --status todo' \
    'wk list --status in_progress' \
    'wk list --blocked' \
    --export-json "$RESULTS_DIR/list-stress-filtered.json"
```

**Performance targets:**

| Command | Database | Mean | P95 |
|---------|----------|------|-----|
| `ready` | stress (20k) | <50ms | <80ms |
| `list` (default) | stress (20k) | <100ms | <150ms |
| `list --limit 500` | stress (20k) | <150ms | <250ms |
| `list --limit 0` | stress (20k) | <2s | <3s |

### Phase 5: Full Validation

**Goal:** Ensure all changes work correctly and meet performance targets.

**Steps:**
```bash
# 1. Format and lint
cargo fmt
cargo clippy

# 2. Unit tests
cargo test --package wok-cli

# 3. Spec tests
make spec

# 4. Generate stress database and run benchmarks
cd checks/benchmarks
./setup/generate_db.sh stress
./scenarios/ready.sh
./scenarios/list.sh

# 5. Full validation
make validate
```

**Success criteria:**
- All spec tests pass
- All benchmarks meet target times
- No clippy warnings
- Code is properly formatted

## Key Implementation Details

### Why These Specific Limits?

1. **`ready` at 5:** The ready queue shows what you should work on next. More than 5 creates decision paralysis and defeats the purpose. The hard limit also guarantees O(1) output time regardless of database size.

2. **`list` at 100:** Terminal output beyond 100 lines is rarely useful for human reading. Users needing more can explicitly request via `--limit 500` or `--limit 0` (unlimited). This keeps default memory usage predictable.

### Preserving Unlimited Access

For scripting and automation:
- `wk list --limit 0` - Shows all issues (no limit)
- `wk list --json --limit 0` - Full data export for tooling
- `wk ready` has no override - always max 5 (by design)

### Sort-Then-Truncate Strategy

Both commands sort before truncating to ensure most relevant issues are shown:

- **`ready`:** Recent high-priority issues first (48h recency window)
- **`list`:** Priority ASC (high first), then created_at DESC (recent first)

This matches the existing pattern in `search.rs:42` (`DEFAULT_LIMIT = 25`).

### Performance Analysis

The blocking logic uses a recursive CTE (`get_blocked_issue_ids()`):
```sql
WITH RECURSIVE all_blockers(issue_id, blocker_id) AS (
    SELECT to_id, from_id FROM deps WHERE rel = 'blocks'
    UNION
    SELECT ab.issue_id, d.from_id
    FROM all_blockers ab
    JOIN deps d ON d.to_id = ab.blocker_id AND d.rel = 'blocks'
)
SELECT DISTINCT issue_id FROM all_blockers ab
JOIN issues i ON i.id = ab.blocker_id
WHERE i.status IN ('todo', 'in_progress')
```

With limits, the blocking query still runs but:
- Results are truncated early, reducing downstream processing
- Label fetching for sorting is bounded by the limit
- JSON serialization is bounded

## Verification Plan

### Unit Tests
```bash
cargo test --package wok-cli -- ready
cargo test --package wok-cli -- list
```

### Spec Tests
```bash
make spec ARGS='--filter "ready"'
make spec ARGS='--filter "list"'
```

### Manual Stress Test
```bash
# Create test repo with many issues
mkdir test-perf && cd test-perf
wk init
for i in {1..500}; do wk add "Issue $i"; done

# Verify limits
wk ready | wc -l        # Should be exactly 5
wk list | wc -l         # Should be exactly 100
wk list --limit 0 | wc -l  # Should be 500

# Time the commands
time wk ready
time wk list
time wk list --limit 0
```

### Benchmark Suite
```bash
cd checks/benchmarks
./setup/generate_db.sh stress  # 20k issues
./scenarios/ready.sh
./scenarios/list.sh
```

### CI Validation
```bash
make validate
```

## Checklist

- [ ] Phase 1: `ready.rs` - Add `MAX_READY_ISSUES = 5` and truncate
- [ ] Phase 2: `list.rs` - Add `DEFAULT_LIMIT = 100` with `--limit 0` escape
- [ ] Phase 3: Add spec tests for limit behaviors
- [ ] Phase 4: Create `ready.sh` benchmark, extend `list.sh` with stress tests
- [ ] Phase 5: Run full validation, verify performance targets met
- [ ] Remove `todo:implement` tag from any new specs
