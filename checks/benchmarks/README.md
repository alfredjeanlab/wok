# wk Benchmarks

Benchmark suite for wk commands, measuring performance for list, search, and write operations at various database scales.

## Prerequisites

Install required tools:

```bash
# macOS
brew install hyperfine jq

# Or via cargo
cargo install hyperfine
```

Required tools:
- **hyperfine** - Command-line benchmarking tool
- **jq** - JSON processing
- **bc** - Calculator (usually pre-installed)
- **sqlite3** - Database operations (bundled with macOS)

## Quick Start

```bash
# Build wk in release mode
cargo build --release

# Generate test databases (one-time setup)
WK_BIN=./target/release/wk ./checks/benchmarks/setup/generate_db.sh

# Run all benchmarks
WK_BIN=./target/release/wk ./checks/benchmarks/run.sh all

# Generate report
./checks/benchmarks/run.sh report
```

## Usage

```bash
./checks/benchmarks/run.sh [OPTIONS] <COMMAND>
```

### Commands

| Command | Description |
|---------|-------------|
| `all` | Run all benchmarks |
| `list` | Run core list benchmarks (default, all, limit) |
| `filter` | Run filter benchmarks (status, type, label, etc.) |
| `combined` | Run combined filter benchmarks |
| `output` | Run output format benchmarks |
| `ready` | Run ready command benchmarks |
| `write` | Run write operation benchmarks (new, edit, close) |
| `search` | Run search command benchmarks |
| `generate` | Generate test databases |
| `report` | Generate markdown report from results |

### Options

| Option | Description |
|--------|-------------|
| `-s, --size` | Database size: small, medium, large, xlarge (default: large) |
| `-v, --verbose` | Enable verbose output |
| `-d, --dry-run` | Show what would run without executing |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WK_BIN` | Path to wk binary | `wk` |
| `RESULTS_DIR` | Directory for JSON results | `checks/benchmarks/results` |

## Database Sizes

| Size | Issues | Description |
|------|--------|-------------|
| small | 100 | Quick sanity checks |
| medium | 1,000 | Development testing |
| large | 5,000 | Standard benchmarks |
| xlarge | 10,000 | Stress testing |

### Data Distribution

- **Status:** 40% todo, 30% in_progress, 30% done
- **Types:** 60% task, 20% bug, 15% feature, 5% epic
- **Labels:** project:{alpha,beta,gamma}, priority:{1,2,3}, area:{frontend,backend,infra}
- **Assignees:** 5 users (alice, bob, carol, david, eve), 20% unassigned
- **Dependencies:** 10-20% of issues have blockers

## Benchmark Scenarios

### Core List Benchmarks

- `list_default_{size}` - Default list (open issues only)
- `list_all_{size}` - List all issues
- `list_limit_{n}` - List with limit (10, 50, 100, 500)

### Filter Benchmarks

Status filters:
- `filter_status_todo` - `wk list --status todo`
- `filter_status_in_progress` - `wk list --status in_progress`
- `filter_status_done` - `wk list --status done`
- `filter_status_multi` - `wk list --status todo,in_progress`

Type filters:
- `filter_type_task` - `wk list --type task --all`
- `filter_type_bug` - `wk list --type bug --all`
- `filter_type_multi` - `wk list --type task,bug --all`

Label filters:
- `filter_label_project` - `wk list --label project:alpha --all`
- `filter_label_priority` - `wk list --label priority:1 --all`
- `filter_label_multi_or` - `wk list --label project:alpha,project:beta --all`
- `filter_label_multi_and` - `wk list --label project:alpha --label priority:1 --all`

Other filters:
- `filter_assignee_*` - Assignee filtering
- `filter_blocked` - Blocked issues
- `filter_age_*` - Time-based filters

### Combined Filter Benchmarks

Real-world multi-filter scenarios:
- `combined_open_bugs` - Open bugs
- `combined_priority_tasks` - High priority tasks
- `combined_my_work` - User's in-progress items
- `combined_complex` - Multiple labels + status + type

### Ready Command Benchmarks

- `ready_default_{size}` - Default ready (hard limit of 5)
- `ready_json_{size}` - Ready with JSON output
- `ready_assignee_*` - Ready with assignee filters
- `ready_label_*` - Ready with label filters
- `ready_type_*` - Ready with type filters

### Write Operation Benchmarks

**Note:** Write benchmarks use DB restoration between runs via hyperfine's `--prepare` flag.

New issue creation:
- `new_sequential_{size}` - Create single issue
- `new_batch_{n}` - Create n issues in sequence (10, 50, 100)

Edit operations:
- `edit_title` - Edit issue title
- `edit_type` - Edit issue type
- `edit_assignee` - Edit issue assignee

Lifecycle operations:
- `start_single` - Start an issue (todo → in_progress)
- `done_single` - Complete an issue (in_progress → done)
- `close_single` - Close an issue
- `close_batch_{n}` - Close n issues (10, 50)

### Search Command Benchmarks

Basic search:
- `search_basic_{size}` - Search scaling across DB sizes
- `search_high_match` - High match rate query (~60%)
- `search_medium_match` - Medium match rate query (~20%)
- `search_low_match` - Low match rate query (~15%)
- `search_no_match` - No match query

Search with filters:
- `search_status_*` - Search with status filter
- `search_type_*` - Search with type filter
- `search_label_*` - Search with label filter
- `search_assignee_*` - Search with assignee filter
- `search_combined_*` - Search with multiple filters

Search limits:
- `search_limit_default` - Default limit (25)
- `search_limit_{n}` - Custom limits (10, 50, 100)
- `search_limit_unlimited` - No limit

Search output:
- `search_output_text` - Text output
- `search_output_json` - JSON output

## Performance Targets

Target performance on large database (5,000 issues):

### Read Operations

| Operation | Target Mean | Target p95 |
|-----------|-------------|------------|
| `list` (default) | < 100ms | < 150ms |
| `list --all` | < 200ms | < 300ms |
| `list --status X` | < 80ms | < 120ms |
| `list --label X` | < 100ms | < 150ms |
| `list --blocked` | < 150ms | < 250ms |
| `list (combined)` | < 150ms | < 250ms |
| `ready` | < 80ms | < 120ms |
| `search` (basic) | < 150ms | < 250ms |
| `search` (with filters) | < 100ms | < 150ms |

### Write Operations

| Operation | Target Mean | Target p95 |
|-----------|-------------|------------|
| `new` (single) | < 50ms | < 80ms |
| `edit` | < 50ms | < 80ms |
| `start` | < 50ms | < 80ms |
| `done` | < 50ms | < 80ms |
| `close` (single) | < 50ms | < 80ms |
| `close` (batch 10) | < 100ms | < 150ms |

**Note:** Write operation benchmarks include DB restoration overhead in the `--prepare` phase,
which is not counted in the measured time.

## Results

Results are stored as JSON in `checks/benchmarks/results/`.

View results:
```bash
# Human-readable summary
jq '.results[0] | {mean, stddev, min, max}' results/list_all_large.json

# Mean time in ms
jq '.results[0].mean * 1000' results/list_all_large.json
```

Generate a markdown report:
```bash
./checks/benchmarks/run.sh report
cat checks/benchmarks/results/report.md
```

## Regression Detection

Compare two benchmark runs:
```bash
./checks/benchmarks/lib/report.sh results/baseline results/current
```

Flag any >20% regression for investigation.

## Directory Structure

```text
checks/benchmarks/
├── README.md                 # This file
├── run.sh                    # Main benchmark runner
├── compare.sh                # Compare benchmark results
├── lib/
│   ├── common.sh             # Shared utilities (colors, db setup, deps)
│   ├── bench.sh              # Hyperfine wrappers for benchmarking
│   └── report.sh             # Report generator
├── setup/
│   ├── generate_db.sh        # Database generator
│   ├── small.sql             # 100 issue test database
│   ├── medium.sql            # 1,000 issue test database
│   ├── large.sql             # 5,000 issue test database
│   └── xlarge.sql            # 10,000 issue test database
├── scenarios/
│   ├── list.sh               # List benchmark scenarios
│   ├── ready.sh              # Ready benchmark scenarios
│   ├── write.sh              # Write operation benchmark scenarios
│   └── search.sh             # Search benchmark scenarios
└── results/
    ├── .gitkeep
    ├── *.json                # Individual benchmark results
    └── report.md             # Generated report
```
