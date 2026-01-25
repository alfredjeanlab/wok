# Implementation Plan: Benchmark Scenarios

**Root Feature:** `wok-527b`

## Overview

Add new benchmark scenarios for write operations (new/edit/close) and search functionality. Extract shared hyperfine wrapper logic to a dedicated `lib/bench.sh` module to DRY up scenario files and support mutation-specific benchmarking patterns.

## Project Structure

```
checks/benchmarks/
├── lib/
│   ├── common.sh          # General utilities (colors, info, setup_db)
│   └── bench.sh           # NEW: Hyperfine wrappers extracted from common.sh
├── scenarios/
│   ├── list.sh            # Existing list benchmarks
│   ├── ready.sh           # Existing ready benchmarks
│   ├── write.sh           # NEW: Mutation benchmarks (new/edit/close)
│   └── search.sh          # NEW: Text search benchmarks
└── run.sh                 # Update to include new scenarios
```

## Dependencies

No new dependencies. Uses existing tools:
- `hyperfine` - Benchmark runner
- `jq` - JSON processing
- `sqlite3` - Database restoration
- `bc` - Floating point math

## Implementation Phases

### Phase 1: Extract Hyperfine Wrappers to lib/bench.sh

Move benchmark-specific functions from `lib/common.sh` to `lib/bench.sh`:

**lib/bench.sh** should contain:
```bash
# Hyperfine wrapper functions
run_benchmark()      # Standard benchmark (warmup 3, runs 30)
run_benchmark_cold() # Cold-start benchmark (warmup 0, runs 20)
run_comparison()     # Multi-command comparison

# Result extraction
get_mean()           # Extract mean from JSON
get_stddev()         # Extract stddev from JSON
get_p95()            # Calculate p95 approximation
format_ms()          # Convert seconds to milliseconds

# NEW: Mutation-specific wrappers
run_benchmark_mutation()    # Benchmark with DB restore between runs
run_benchmark_batch()       # Batch operation benchmark
```

**lib/common.sh** should retain:
- Color definitions and output functions (info, success, warn, error)
- `setup_db()` / `restore_db()`
- `check_dependencies()` / `check_wk_binary()`
- `generate_latest_json()`

Update sourcing in `run.sh`:
```bash
source "$SCRIPT_DIR/lib/common.sh"
source "$SCRIPT_DIR/lib/bench.sh"
```

**Verification:** Run existing benchmarks to confirm no regression.

### Phase 2: Create scenarios/write.sh

Implement mutation benchmarks for `new`, `edit`, and `close` commands.

**Key challenge:** Mutations modify database state, so benchmarks need special handling:
1. Use hyperfine's `--prepare` to restore DB before each run
2. Use `--cleanup` for any post-run cleanup

**Benchmark categories:**

1. **Sequential Creation** - Single issue creation:
   ```bash
   benchmark_new_sequential() {
       setup_db large
       local restore_cmd="sqlite3 .wok/issues.db < $SCRIPT_DIR/setup/large.sql"

       hyperfine \
           --warmup 3 \
           --min-runs 30 \
           --prepare "$restore_cmd" \
           --export-json "$RESULTS_DIR/new_sequential.json" \
           "$WK_BIN new task 'Benchmark task'"
   }
   ```

2. **Batch Creation** - Multiple issues at once:
   ```bash
   benchmark_new_batch() {
       # Benchmark creating 10, 50, 100 issues in sequence
       for count in 10 50 100; do
           setup_db large
           # Generate commands file or use loop
           run_benchmark "new_batch_${count}" ...
       done
   }
   ```

3. **Edit Operations**:
   - Edit title (benchmark-1 through benchmark-100)
   - Edit description
   - Edit type
   - Edit assignee

4. **Close Operations**:
   - Single close: `wk close bench-1 -r "Benchmark"`
   - Batch close: `wk close bench-1 bench-2 ... bench-10 -r "Benchmark"`

**Benchmark functions:**
```bash
run_write_benchmarks()      # All write benchmarks
benchmark_new_sequential()  # Single new
benchmark_new_batch()       # Batch new (10, 50, 100)
benchmark_edit_title()      # Edit title
benchmark_edit_type()       # Edit type
benchmark_edit_assignee()   # Edit assignee
benchmark_close_single()    # Single close
benchmark_close_batch()     # Batch close (10, 50, 100)
```

**Verification:** Run `./run.sh write` and verify results in `results/`.

### Phase 3: Create scenarios/search.sh

Implement text search benchmarks across database sizes.

**Benchmark categories:**

1. **Basic Search** - Simple term search:
   ```bash
   benchmark_search_basic() {
       for size in small medium large xlarge; do
           setup_db "$size"
           run_benchmark "search_basic_${size}" "$WK_BIN" search "task"
       done
   }
   ```

2. **Search with Filters**:
   - Status filter: `wk search "task" --status todo`
   - Type filter: `wk search "bug" --type bug`
   - Label filter: `wk search "alpha" --label project:alpha`
   - Assignee filter: `wk search "task" --assignee alice`
   - Combined: `wk search "priority" --status todo --type task --label priority:1`

3. **Result Limits**:
   - Default limit (25)
   - Custom limits: 10, 50, 100
   - Unlimited: `--limit 0`

4. **Output Formats**:
   - Text output (default)
   - JSON output
   - IDs only

5. **Complex Queries** - Realistic search patterns:
   - Multi-word queries
   - Partial matches
   - No-match queries (empty results)

**Benchmark functions:**
```bash
run_search_benchmarks()        # All search benchmarks
benchmark_search_basic()       # Basic search scaling
benchmark_search_filters()     # Search with filters
benchmark_search_limits()      # Result limit variations
benchmark_search_output()      # Output format comparison
benchmark_search_complex()     # Complex query patterns
```

**Verification:** Run `./run.sh search` and verify results in `results/`.

### Phase 4: Integrate with run.sh

Update `run.sh` to support new scenarios:

```bash
# Source new scenario files
source "$SCRIPT_DIR/scenarios/list.sh"
source "$SCRIPT_DIR/scenarios/ready.sh"
source "$SCRIPT_DIR/scenarios/write.sh"
source "$SCRIPT_DIR/scenarios/search.sh"

# Add new commands to usage and case statement
COMMANDS:
    write           Run write operation benchmarks (new/edit/close)
    search          Run search benchmarks
```

Update `run_all()` to include:
```bash
run_all() {
    ...
    run_list_benchmarks
    run_filter_benchmarks
    run_combined_benchmarks
    run_output_benchmarks
    run_ready_benchmarks     # Add if not present
    run_write_benchmarks     # NEW
    run_search_benchmarks    # NEW
    ...
}
```

**Verification:** Run `./run.sh all` with new scenarios.

### Phase 5: Update Documentation and Report

1. Update `checks/benchmarks/README.md`:
   - Document new scenarios (write, search)
   - Add performance targets for mutations and search
   - Update command examples

2. Update `lib/report.sh` if needed:
   - Add sections for write and search benchmarks
   - Include mutation benchmark caveats (DB restoration overhead)

**Verification:** Generate report with `./run.sh report` and verify completeness.

## Key Implementation Details

### Mutation Benchmark Pattern

Mutations require database restoration between runs. Use hyperfine's `--prepare` flag:

```bash
run_benchmark_mutation() {
    local name="$1"
    local size="$2"
    shift 2
    local cmd="$*"

    local output_file="$RESULTS_DIR/${name}.json"
    local sql_file="$SCRIPT_DIR/setup/${size}.sql"

    mkdir -p "$RESULTS_DIR"

    info "Running mutation benchmark: $name"
    hyperfine \
        --warmup 3 \
        --min-runs 30 \
        --prepare "rm -rf .wok && mkdir -p .wok && sqlite3 .wok/issues.db < $sql_file && echo 'prefix = \"bench\"' > .wok/config.toml" \
        --shell=none \
        --export-json "$output_file" \
        "$cmd"

    success "Results saved to: $output_file"
}
```

### Batch Operation Pattern

For batch benchmarks, measure total time for N operations:

```bash
# Option 1: Shell script wrapper
hyperfine --prepare "restore_db large" \
    "for i in {1..10}; do $WK_BIN new task \"Task \$i\"; done"

# Option 2: Use --parameter-list for variations
hyperfine --parameter-list count 10,50,100 \
    --prepare "restore_db large" \
    "bash -c 'for i in \$(seq 1 {count}); do $WK_BIN new task \"Task \$i\"; done'"
```

### Search Query Selection

Use search terms that match generated data:
- "task" - Matches ~60% of issues (type distribution)
- "bug" - Matches ~20% of issues
- "project" - Appears in labels
- "alice", "bob" - Appear in assignees and notes
- Numeric IDs: "123" - For ID-based search

### Performance Expectations

Based on existing benchmarks and command complexity:

| Benchmark | Target Mean | Target p95 |
|-----------|-------------|------------|
| new (single) | < 50ms | < 80ms |
| edit | < 50ms | < 80ms |
| close (single) | < 50ms | < 80ms |
| close (batch 10) | < 100ms | < 150ms |
| search (small) | < 50ms | < 80ms |
| search (large) | < 150ms | < 250ms |
| search + filters | < 100ms | < 150ms |

## Verification Plan

1. **Phase 1 Verification:**
   ```bash
   # Run existing benchmarks after refactor
   WK_BIN=./target/release/wk ./checks/benchmarks/run.sh list
   # Compare results with baseline
   ./checks/benchmarks/compare.sh
   ```

2. **Phase 2 Verification:**
   ```bash
   # Run write benchmarks
   WK_BIN=./target/release/wk ./checks/benchmarks/run.sh write
   # Verify JSON output files
   ls checks/benchmarks/results/new_*.json
   ls checks/benchmarks/results/edit_*.json
   ls checks/benchmarks/results/close_*.json
   ```

3. **Phase 3 Verification:**
   ```bash
   # Run search benchmarks
   WK_BIN=./target/release/wk ./checks/benchmarks/run.sh search
   # Verify scaling across database sizes
   jq '.results[0].mean' checks/benchmarks/results/search_basic_*.json
   ```

4. **Phase 4 Verification:**
   ```bash
   # Run full benchmark suite
   WK_BIN=./target/release/wk ./checks/benchmarks/run.sh all
   # Generate report
   ./checks/benchmarks/run.sh report
   ```

5. **Regression Testing:**
   ```bash
   # Run comparison against baseline
   ./checks/benchmarks/compare.sh baseline.json
   ```
