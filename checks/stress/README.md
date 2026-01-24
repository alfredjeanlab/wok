# Stress Test Suite for `wk` CLI

Stress tests push `wk` implementations to extremes: massive databases, deep dependency chains, concurrent access, and resource exhaustion scenarios.

## Quick Start

```bash
# Run all stress tests safely in Docker (RECOMMENDED)
WK_BIN=./crates/cli/target/release/wk ./checks/stress/docker-run.sh

# Run specific scenario
WK_BIN=./wk ./checks/stress/docker-run.sh massive_db 50000

# Native execution (less safe - uses ulimit)
WK_BIN=./wk ./checks/stress/run.sh limits
```

## Test Categories

| Category | Description | Scenarios |
|----------|-------------|-----------|
| **scale** | Push database size to extremes | massive_db, deep_deps, wide_deps, many_tags, many_notes |
| **limits** | Find hard limits and edge cases | title_length, note_length, tag_length, id_collisions, path_depth |
| **concurrent** | Test database locking | parallel_writes, parallel_reads, mixed_workload, lock_contention |
| **corruption** | Test recovery from failures | interrupted_write, disk_full, corrupt_db, corrupt_config |
| **memory** | Test memory usage under load | memory_limit, large_export, list_all |

## Safety Mechanisms

All tests run within safety constraints to prevent host system damage:

- **Memory limit**: 2GB max (configurable via `STRESS_MAX_MEMORY_MB`)
- **Disk limit**: 5GB max (configurable via `STRESS_MAX_DISK_MB`)
- **Process limit**: 100 max (configurable via `STRESS_MAX_PROCS`)
- **Timeout**: 5 minutes per test (configurable via `STRESS_TIMEOUT_SEC`)

### Docker (Recommended)

Docker provides hard resource limits that cannot be bypassed:

```bash
WK_BIN=./wk ./checks/stress/docker-run.sh all
```

### Native Execution

Native execution uses `ulimit` as a fallback. Limits are advisory and may be exceeded:

```bash
WK_BIN=./wk ./checks/stress/run.sh all
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WK_BIN` | `wk` | Path to wk binary |
| `STRESS_MAX_MEMORY_MB` | 2048 | Max memory per test |
| `STRESS_MAX_DISK_MB` | 5120 | Max disk usage |
| `STRESS_MAX_PROCS` | 100 | Max child processes |
| `STRESS_MAX_FILES` | 1024 | Max open file handles |
| `STRESS_TIMEOUT_SEC` | 300 | Test timeout (5 min) |
| `STRESS_MIN_FREE_DISK_MB` | 1024 | Keep this much free |
| `STRESS_SKIP_DANGEROUS` | 0 | Set to 1 to skip disk_full test |

## Running Individual Tests

```bash
# Scale tests
./checks/stress/run.sh massive_db 100000    # Create 100k issues
./checks/stress/run.sh deep_deps 1000       # 1000-level dependency chain
./checks/stress/run.sh wide_deps 1000       # 1 issue blocking 1000 others

# Limit tests
./checks/stress/run.sh limits               # All limit tests

# Concurrent tests
./checks/stress/run.sh parallel_writes 10 100   # 10 writers, 100 issues each
./checks/stress/run.sh mixed_workload 30        # 30 second mixed workload

# All tests
./checks/stress/run.sh all
```

## Expected Findings

### Known Limits

| Aspect | Expected Limit | Notes |
|--------|---------------|-------|
| Issues | 100k+ | SQLite handles millions |
| Dependency depth | ~1000 | Recursive CTE limit |
| Dependencies per issue | 10k+ | Query performance |
| Title length | Varies | SQLite TEXT unlimited |
| Concurrent writers | ~10-50 | SQLite WAL helps |

### Failure Modes

1. **Cycle detection timeout** - Very deep graphs
2. **Memory exhaustion** - Large exports, list all
3. **Lock timeout** - High concurrency writes
4. **Corruption recovery** - After crashes

## Directory Structure

```text
checks/stress/
├── README.md
├── docker-run.sh          # Primary entry point (Docker)
├── run.sh                  # Main runner
├── lib/
│   ├── safety.sh           # Resource limits and sandbox
│   ├── common.sh           # Shared utilities
│   ├── generators.sh       # Data generation
│   └── monitors.sh         # Resource monitoring
├── scenarios/
│   ├── scale/              # Database size tests
│   ├── limits/             # Edge case tests
│   ├── concurrent/         # Locking tests
│   ├── corruption/         # Recovery tests
│   └── memory/             # Memory tests
└── results/
    └── .gitkeep
```

## Interpreting Results

Each test outputs:
- **PASS**: Test completed within limits
- **FAIL**: Test found breaking point or data loss
- **TIMEOUT**: Test exceeded time limit
- **ABORT**: Test aborted due to resource pressure

Performance metrics include:
- Operations per second
- Peak memory usage
- Database size
- Error counts
