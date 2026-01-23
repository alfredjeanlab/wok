# Plan: Criterion Microbenchmarks

## Overview

Add criterion microbenchmarks to `crates/cli/benches/` targeting CPU-bound parsing, evaluation, and data mapping operations. These benchmarks enable regression detection in CI without full hyperfine runs, focusing on isolated function performance rather than end-to-end CLI timing.

## Project Structure

```
crates/cli/
├── Cargo.toml          # Add criterion dev-dependency + [[bench]] entries
├── benches/
│   ├── filter.rs       # Filter parsing and evaluation benchmarks
│   ├── priority.rs     # Priority extraction from label lists
│   └── row_mapping.rs  # SQL row to Issue struct mapping
└── src/
    └── db/mod.rs       # Expose open_in_memory() for bench use
```

## Dependencies

Add to `crates/cli/Cargo.toml`:

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "filter"
harness = false

[[bench]]
name = "priority"
harness = false

[[bench]]
name = "row_mapping"
harness = false
```

## Implementation Phases

### Phase 1: Project Setup

1. Add criterion dependency to `crates/cli/Cargo.toml`
2. Add `[[bench]]` entries for each benchmark file
3. Create `crates/cli/benches/` directory
4. Expose `Database::open_in_memory()` for benchmarks by changing `#[cfg(test)]` to `#[cfg(any(test, feature = "bench"))]` or making it always available

**Verification:** `cargo check -p wk --benches` compiles without errors

### Phase 2: Filter Benchmarks (`benches/filter.rs`)

Create benchmarks for filter parsing and evaluation logic.

**Target functions:**
- `filter::parser::parse_filter()` - Parse filter expressions like `"age < 3d"`
- `filter::parser::parse_duration()` - Parse duration strings like `"3d"`, `"1w"`
- `FilterExpr::matches()` - Evaluate filter against an Issue

**Benchmark groups:**
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use chrono::{Duration, Utc};
use wkrs::filter::{parse_filter, FilterExpr};
use wkrs::models::{Issue, Status};

fn filter_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_parsing");

    let inputs = [
        ("duration_simple", "age < 3d"),
        ("duration_word_op", "age lte 1w"),
        ("date", "created > 2024-01-01"),
        ("bare_status", "closed"),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("parse_filter", name), input, |b, i| {
            b.iter(|| parse_filter(i))
        });
    }
    group.finish();
}

fn filter_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_evaluation");
    let now = Utc::now();

    // Create test issues with varying characteristics
    let recent_issue = create_issue(now - Duration::hours(1));
    let old_issue = create_issue(now - Duration::weeks(4));

    let filters = [
        parse_filter("age < 3d").unwrap(),
        parse_filter("age > 1w").unwrap(),
        parse_filter("updated >= 2024-01-01").unwrap(),
    ];

    for (i, filter) in filters.iter().enumerate() {
        group.bench_function(format!("matches_recent_{}", i), |b| {
            b.iter(|| filter.matches(&recent_issue, now))
        });
    }
    group.finish();
}

criterion_group!(benches, filter_parsing, filter_evaluation);
criterion_main!(benches);
```

**Verification:** `cargo bench -p wk --bench filter` runs and produces results

### Phase 3: Priority Benchmarks (`benches/priority.rs`)

Benchmark priority extraction from label lists.

**Target function:**
- `Database::priority_from_tags()` - Extract priority (0-4) from label list

**Benchmark groups:**
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use wkrs::db::Database;

fn priority_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_extraction");

    let cases: &[(&str, Vec<String>)] = &[
        ("empty", vec![]),
        ("no_priority", vec!["bug".into(), "frontend".into()]),
        ("priority_prefix", vec!["bug".into(), "priority:high".into()]),
        ("p_prefix", vec!["p:1".into(), "feature".into()]),
        ("both_prefixes", vec!["p:3".into(), "priority:1".into()]),  // priority: wins
        ("many_labels", (0..20).map(|i| format!("label{}", i)).chain(["priority:0".into()]).collect()),
    ];

    for (name, labels) in cases {
        group.bench_with_input(BenchmarkId::new("priority_from_tags", name), labels, |b, l| {
            b.iter(|| Database::priority_from_tags(l))
        });
    }
    group.finish();
}

criterion_group!(benches, priority_extraction);
criterion_main!(benches);
```

**Verification:** `cargo bench -p wk --bench priority` runs and produces results

### Phase 4: Row Mapping Benchmarks (`benches/row_mapping.rs`)

Benchmark SQL row to Issue struct mapping with an in-memory database.

**Target operations:**
- `parse_timestamp()` - RFC3339 string to DateTime<Utc>
- `parse_db()` - String to enum (Status, IssueType)
- Full row mapping in `list_issues()` / `get_issue()`

**Benchmark groups:**
```rust
use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use wkrs::db::Database;
use wkrs::models::{Issue, IssueType, Status};
use chrono::Utc;

fn row_mapping(c: &mut Criterion) {
    let mut group = c.benchmark_group("row_mapping");

    // Setup: create in-memory DB with test issues
    group.bench_function("get_issue", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().unwrap();
                let issue = create_test_issue("test-001");
                db.create_issue(&issue).unwrap();
                (db, issue.id)
            },
            |(db, id)| db.get_issue(&id),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("list_issues_10", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().unwrap();
                for i in 0..10 {
                    db.create_issue(&create_test_issue(&format!("test-{:03}", i))).unwrap();
                }
                db
            },
            |db| db.list_issues(None, None, None),
            BatchSize::SmallInput,
        )
    });

    group.bench_function("list_issues_100", |b| {
        b.iter_batched(
            || {
                let db = Database::open_in_memory().unwrap();
                for i in 0..100 {
                    db.create_issue(&create_test_issue(&format!("test-{:03}", i))).unwrap();
                }
                db
            },
            |db| db.list_issues(None, None, None),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn create_test_issue(id: &str) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type: IssueType::Task,
        title: "Test issue".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    }
}

criterion_group!(benches, row_mapping);
criterion_main!(benches);
```

**Verification:** `cargo bench -p wk --bench row_mapping` runs and produces results

### Phase 5: Integration & CI Setup

1. Create helper module for shared test fixtures (if needed)
2. Verify all benchmarks run: `cargo bench -p wk`
3. Add benchmark step to CI (optional, for regression tracking)
4. Update `checks/benchmarks/README.md` to reference criterion microbenchmarks

**Verification:** All benchmarks complete successfully, baseline results captured

## Key Implementation Details

### Exposing In-Memory Database

The `Database::open_in_memory()` method is currently `#[cfg(test)]` only. Two options:

**Option A (preferred):** Remove the `#[cfg(test)]` restriction entirely. In-memory databases are safe and useful for testing/benchmarking:

```rust
// db/mod.rs
/// Open an in-memory database (for testing and benchmarks)
pub fn open_in_memory() -> Result<Self> { ... }
```

**Option B:** Add a `bench` feature flag:

```toml
# Cargo.toml
[features]
bench = []

# db/mod.rs
#[cfg(any(test, feature = "bench"))]
pub fn open_in_memory() -> Result<Self> { ... }
```

### Criterion Group Organization

Each benchmark file defines one or more groups. Groups should be named clearly:
- `filter_parsing` - Filter expression parsing
- `filter_evaluation` - Filter matching against issues
- `priority_extraction` - Priority from label lists
- `row_mapping` - Database row to struct conversion

### Avoiding I/O in Hot Paths

Use `iter_batched` for benchmarks requiring setup (database creation). This ensures setup time isn't measured:

```rust
b.iter_batched(
    || setup_database(),        // Setup (not measured)
    |db| db.list_issues(...),   // Benchmark (measured)
    BatchSize::SmallInput,
)
```

### Module Visibility

Benchmarks need access to:
- `wkrs::filter::{parse_filter, FilterExpr}` - Already public via `filter/mod.rs`
- `wkrs::db::Database` - Already public
- `wkrs::models::{Issue, Status, IssueType}` - Already public

## Verification Plan

1. **Compilation check:** `cargo check -p wk --benches`
2. **Individual benchmark runs:**
   - `cargo bench -p wk --bench filter`
   - `cargo bench -p wk --bench priority`
   - `cargo bench -p wk --bench row_mapping`
3. **Full benchmark suite:** `cargo bench -p wk`
4. **Baseline comparison:** Run twice and compare for stability
5. **Landing checklist:** Follow `crates/cli/CLAUDE.md` checklist
