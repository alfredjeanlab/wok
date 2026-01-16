# wk Benchmarks

Benchmark suite for the `wk list` command, measuring filtering and query performance at various database scales.

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

## Performance Targets

Target performance on large database (5,000 issues):

| Operation | Target Mean | Target p95 |
|-----------|-------------|------------|
| `list` (default) | < 100ms | < 150ms |
| `list --all` | < 200ms | < 300ms |
| `list --status X` | < 80ms | < 120ms |
| `list --label X` | < 100ms | < 150ms |
| `list --blocked` | < 150ms | < 250ms |
| `list (combined)` | < 150ms | < 250ms |

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

```
checks/benchmarks/
├── README.md                 # This file
├── run.sh                    # Main benchmark runner
├── lib/
│   ├── common.sh             # Shared utilities
│   └── report.sh             # Report generator
├── setup/
│   ├── generate_db.sh        # Database generator
│   ├── small.sql             # 100 issue test database
│   ├── medium.sql            # 1,000 issue test database
│   ├── large.sql             # 5,000 issue test database
│   └── xlarge.sql            # 10,000 issue test database
├── scenarios/
│   └── list.sh               # List benchmark scenarios
└── results/
    ├── .gitkeep
    ├── *.json                # Individual benchmark results
    └── report.md             # Generated report
```
