# WK_TIMINGS Environment Variable Instrumentation

**Root Feature:** `wok-fbd0`

## Overview

Add `WK_TIMINGS=1` environment variable to output phase durations to stderr for performance debugging. This lightweight instrumentation helps identify performance bottlenecks without external dependencies, using simple `Instant::now()` + `eprintln!` patterns gated by the environment variable check.

**Output format:** `[timings] phase::name XXms`

**Instrumented phases:**
- `db::open` - Database open and migration
- `db::query` - Issue listing query
- `filter::labels` - N+1 label filtering hotspot
- `filter::blocked` - Blocked issues query
- `sort` - Priority-based sorting
- `format` - Output formatting

## Project Structure

```
crates/cli/src/
├── timings.rs           # NEW: Timing helper macro and utilities
├── lib.rs               # Add timings module
├── commands/
│   ├── mod.rs           # Instrument db::open in open_db() (line 49)
│   └── list.rs          # Instrument filter::labels (~line 106), filter::blocked (~line 133),
│                        # sort (~line 138), format (~line 156)
└── db/
    └── issues.rs        # get_blocked_issue_ids called from list.rs (line 307)
```

## Dependencies

None. Uses only standard library:
- `std::time::Instant`
- `std::env::var`

## Implementation Phases

### Phase 1: Create Timing Helper Module

**Goal:** Create a reusable macro for timing code blocks, gated by `WK_TIMINGS` env var.

**File:** `crates/cli/src/timings.rs`

```rust
//! Performance timing instrumentation for debugging.
//!
//! Enable with `WK_TIMINGS=1` environment variable.
//! Output goes to stderr in format: `[timings] phase::name XXms`

use std::time::Instant;

/// Check if timings are enabled via WK_TIMINGS environment variable.
#[inline]
pub fn timings_enabled() -> bool {
    std::env::var("WK_TIMINGS").is_ok()
}

/// Print a timing result to stderr if timings are enabled.
#[inline]
pub fn print_timing(phase: &str, start: Instant) {
    if timings_enabled() {
        let elapsed = start.elapsed();
        eprintln!("[timings] {} {}ms", phase, elapsed.as_millis());
    }
}

/// Macro for timing a block of code.
///
/// Usage:
/// ```rust
/// let result = time_phase!("db::open", {
///     Database::open(&path)
/// });
/// ```
#[macro_export]
macro_rules! time_phase {
    ($phase:expr, $block:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $block;
        $crate::timings::print_timing($phase, __start);
        __result
    }};
}
```

**Update:** `crates/cli/src/lib.rs`

Add module declaration:
```rust
pub mod timings;
```

**Verification:**
- `cargo check --package wok-cli`

### Phase 2: Instrument Database Open

**Goal:** Measure time to open database connection and run migrations.

**File:** `crates/cli/src/commands/mod.rs`

**Current code (line 45-51):**
```rust
pub fn open_db() -> Result<(Database, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = Database::open(&db_path)?;
    Ok((db, config, work_dir))
}
```

**New code:**
```rust
pub fn open_db() -> Result<(Database, Config, PathBuf)> {
    let work_dir = find_work_dir()?;
    let config = Config::load(&work_dir)?;
    let db_path = get_db_path(&work_dir, &config);
    let db = time_phase!("db::open", { Database::open(&db_path)? });
    Ok((db, config, work_dir))
}
```

**Verification:**
```bash
WK_TIMINGS=1 wk list 2>&1 | grep "db::open"
# Expected: [timings] db::open 12ms
```

### Phase 3: Instrument Database Query

**Goal:** Measure time for the main issue listing query.

**File:** `crates/cli/src/commands/list.rs`

**Current code (line 89):**
```rust
let mut issues = db.list_issues(None, None, None)?;
```

**New code:**
```rust
let mut issues = time_phase!("db::query", { db.list_issues(None, None, None)? });
```

**Verification:**
```bash
WK_TIMINGS=1 wk list 2>&1 | grep "db::query"
# Expected: [timings] db::query 5ms
```

### Phase 4: Instrument Label Filtering (N+1 Hotspot)

**Goal:** Measure time for the label filtering loop, which is a known N+1 query hotspot.

**File:** `crates/cli/src/commands/list.rs`

**Current code (lines 106-111):**
```rust
// Filter by label groups
if label_groups.is_some() {
    issues.retain(|issue| {
        let issue_labels = db.get_labels(&issue.id).unwrap_or_default();
        matches_label_groups(&label_groups, &issue_labels)
    });
}
```

**New code:**
```rust
// Filter by label groups
if label_groups.is_some() {
    let start = std::time::Instant::now();
    issues.retain(|issue| {
        let issue_labels = db.get_labels(&issue.id).unwrap_or_default();
        matches_label_groups(&label_groups, &issue_labels)
    });
    crate::timings::print_timing("filter::labels", start);
}
```

**Note:** Cannot use macro here due to the closure capturing `db`. Use direct timing calls instead.

**Verification:**
```bash
WK_TIMINGS=1 wk list --label foo 2>&1 | grep "filter::labels"
# Expected: [timings] filter::labels 150ms  (high value indicates N+1 problem)
```

### Phase 5: Instrument Blocked Filter

**Goal:** Measure time for the blocked issues query (recursive CTE).

**File:** `crates/cli/src/commands/list.rs`

**Current code (lines 132-135):**
```rust
if blocked_only {
    let blocked_ids: HashSet<String> = db.get_blocked_issue_ids()?.into_iter().collect();
    issues.retain(|issue| blocked_ids.contains(&issue.id));
}
```

**New code:**
```rust
if blocked_only {
    let blocked_ids: HashSet<String> = time_phase!("filter::blocked", {
        db.get_blocked_issue_ids()?.into_iter().collect()
    });
    issues.retain(|issue| blocked_ids.contains(&issue.id));
}
```

**Verification:**
```bash
WK_TIMINGS=1 wk list --blocked 2>&1 | grep "filter::blocked"
# Expected: [timings] filter::blocked 8ms
```

### Phase 6: Instrument Sorting

**Goal:** Measure time for priority-based sorting (another N+1 hotspot).

**File:** `crates/cli/src/commands/list.rs`

**Current code (lines 137-148):**
```rust
// Sort by priority ASC, then created_at DESC
issues.sort_by(|a, b| {
    let tags_a = db.get_labels(&a.id).unwrap_or_default();
    let tags_b = db.get_labels(&b.id).unwrap_or_default();
    let priority_a = Database::priority_from_tags(&tags_a);
    let priority_b = Database::priority_from_tags(&tags_b);

    match priority_a.cmp(&priority_b) {
        std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at), // DESC
        other => other,
    }
});
```

**New code:**
```rust
// Sort by priority ASC, then created_at DESC
let sort_start = std::time::Instant::now();
issues.sort_by(|a, b| {
    let tags_a = db.get_labels(&a.id).unwrap_or_default();
    let tags_b = db.get_labels(&b.id).unwrap_or_default();
    let priority_a = Database::priority_from_tags(&tags_a);
    let priority_b = Database::priority_from_tags(&tags_b);

    match priority_a.cmp(&priority_b) {
        std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at), // DESC
        other => other,
    }
});
crate::timings::print_timing("sort", sort_start);
```

**Verification:**
```bash
WK_TIMINGS=1 wk list 2>&1 | grep "sort"
# Expected: [timings] sort 200ms  (high value indicates N+1 in sort comparisons)
```

### Phase 7: Instrument Output Formatting

**Goal:** Measure time for formatting and printing output.

**File:** `crates/cli/src/commands/list.rs`

**Current code (lines 156-193):**
```rust
match format {
    OutputFormat::Text => {
        for issue in &issues {
            println!("{}", format_issue_line(issue));
        }
    }
    OutputFormat::Json => { /* ... */ }
    OutputFormat::Ids => { /* ... */ }
}
```

**New code:**
```rust
let format_start = std::time::Instant::now();
match format {
    OutputFormat::Text => {
        for issue in &issues {
            println!("{}", format_issue_line(issue));
        }
    }
    OutputFormat::Json => { /* ... */ }
    OutputFormat::Ids => { /* ... */ }
}
crate::timings::print_timing("format", format_start);
```

**Verification:**
```bash
WK_TIMINGS=1 wk list 2>&1 | grep "format"
# Expected: [timings] format 3ms
```

## Key Implementation Details

### Environment Variable Check Optimization

The `timings_enabled()` function is called for each `print_timing()` call. For hot paths, this adds minimal overhead since `std::env::var()` is cached by the OS. The check is inlined for performance.

### Macro vs Direct Calls

Use the `time_phase!` macro when:
- The block is a simple expression
- No closures capture external state

Use direct `Instant::now()` + `print_timing()` when:
- Code involves closures (like `retain()` or `sort_by()`)
- Multiple statements need timing

### Output to Stderr

Timings go to stderr to avoid interfering with command output that goes to stdout. This allows:
```bash
WK_TIMINGS=1 wk list > issues.txt 2> timings.txt
```

### Expected Output Example

```
$ WK_TIMINGS=1 wk list --label crate:cli
[timings] db::open 12ms
[timings] db::query 5ms
[timings] filter::labels 150ms
[timings] sort 200ms
[timings] format 3ms
- [task] (todo) wok-1234: Implement feature X
- [bug] (todo) wok-1235: Fix issue Y
```

## Verification Plan

### Build Verification
```bash
cargo check --package wok-cli
cargo clippy --package wok-cli
cargo test --package wok-cli
```

### Manual Testing
```bash
# Verify no output without env var
wk list 2>&1 | grep timings
# Expected: no output

# Verify output with env var
WK_TIMINGS=1 wk list 2>&1 | grep timings
# Expected: timing lines for each phase

# Verify all phases appear (with --label filter)
WK_TIMINGS=1 wk list --label test 2>&1 | grep -c timings
# Expected: 5 (db::open, db::query, filter::labels, sort, format)

# Verify basic list (no label filter)
WK_TIMINGS=1 wk list 2>&1 | grep -c timings
# Expected: 4 (db::open, db::query, sort, format)

# Verify blocked filter
WK_TIMINGS=1 wk list --blocked 2>&1 | grep "filter::blocked"
# Expected: timing line appears
```

### Spec Tests
```bash
make spec-cli
```

## Checklist

- [ ] Phase 1: Create `timings.rs` module with helper macro
- [ ] Phase 2: Instrument `db::open` in `commands/mod.rs`
- [ ] Phase 3: Instrument `db::query` in `commands/list.rs`
- [ ] Phase 4: Instrument `filter::labels` in `commands/list.rs`
- [ ] Phase 5: Instrument `filter::blocked` in `commands/list.rs`
- [ ] Phase 6: Instrument `sort` in `commands/list.rs`
- [ ] Phase 7: Instrument `format` in `commands/list.rs`
- [ ] Run `cargo check && cargo clippy && cargo test`
- [ ] Manual verification with `WK_TIMINGS=1`
