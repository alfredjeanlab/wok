# Project Structure

## Binaries (`bin/`)

```
bin/
├── cli/        # wk - main CLI tool (Rust)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── cli.rs          # Clap argument parsing
│   │   ├── lib.rs          # Library entry, command dispatch
│   │   ├── commands/       # Subcommand implementations
│   │   ├── db/             # SQLite database operations
│   │   ├── models/         # Data types (Issue, Event, Note, etc.)
│   │   ├── daemon/         # Background sync daemon
│   │   └── sync/           # Remote sync client
│   └── tests/
│       └── integration.rs  # CLI integration tests
└── remote/     # wk-remote - sync server (Rust)
    └── src/
        ├── main.rs         # Entry point
        ├── server.rs       # WebSocket server
        └── state.rs        # Server state management
```

## Test Suites (`checks/`)

```
checks/
├── specs/          # BATS acceptance tests (validates REQUIREMENTS.md)
│   ├── run.sh              # Test runner
│   ├── helpers/common.bash # Shared test utilities
│   ├── unit/               # Per-command tests
│   ├── integration/        # Cross-feature tests
│   └── edge_cases/         # Error handling, limits, cycles
├── benchmarks/     # Performance benchmarks
│   ├── run.sh              # Benchmark runner
│   └── scenarios/          # Individual benchmark scripts
└── quality/        # Code quality metrics
    ├── evaluate.sh         # Quality evaluation script
    └── metrics/            # Individual metric collectors
```

## Running Tests

```bash
# Build CLI
(cd bin/cli && cargo build --release)

# Run Rust tests
(cd bin/cli && cargo test)

# Run BATS acceptance tests
WK_BIN=bin/cli/target/release/wk checks/specs/run.sh

# Run benchmarks
checks/benchmarks/run.sh
```

## CLI Behavior

Commands that require arguments show help when called without them:

```bash
wk show       # Shows: Usage: wk show [OPTIONS] <ID>
wk start      # Shows: Usage: wk start <ID>
wk label      # Shows: Usage: wk label <ID> <LABEL>
```

This provides actionable guidance instead of cryptic "missing argument" errors.
