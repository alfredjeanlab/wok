# Project Structure

## Binaries (`bin/`)

```text
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

## Test Suites (`tests/`)

```
tests/
└── specs/          # BATS acceptance tests (validates REQUIREMENTS.md)
    ├── helpers/common.bash # Shared test utilities
    ├── cli/                # CLI tests (unit, integration, edge_cases)
    └── remote/             # Remote tests (unit, integration, edge_cases)
```

## Running Tests

```bash
# Build CLI
cargo build --release

# Run Rust tests
cargo test

# Run BATS acceptance tests
make spec
```

## CLI Behavior

Commands that require arguments show help when called without them:

```bash
wk show       # Shows: Usage: wk show [OPTIONS] <ID>
wk start      # Shows: Usage: wk start <ID>
wk label      # Shows: Usage: wk label <ID> <LABEL>
```

This provides actionable guidance instead of cryptic "missing argument" errors.
