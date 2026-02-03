# Project Structure

## Crates (`crates/`)

```text
crates/
├── cli/        # wok - main CLI tool (Rust)
│   ├── src/
│   │   ├── main.rs         # Entry point
│   │   ├── cli/            # Clap argument parsing
│   │   ├── lib.rs          # Library entry, command dispatch
│   │   ├── commands/       # Subcommand implementations
│   │   ├── db/             # SQLite database operations
│   │   ├── models/         # Data types (Issue, Event, Note, etc.)
│   │   └── daemon/         # Daemon client (start, stop, status)
│   └── tests/
├── core/       # wok-core - shared library
│   └── src/
└── daemon/     # wokd - IPC daemon (Rust)
    └── src/
        ├── main.rs         # Entry point
        └── ipc.rs          # Unix socket IPC
```

## Test Suites (`tests/`)

```
tests/
└── specs/          # BATS acceptance tests (validates REQUIREMENTS.md)
    ├── helpers/common.bash # Shared test utilities
    └── cli/                # CLI tests (unit, integration, edge_cases)
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
wok show       # Shows: Usage: wok show [OPTIONS] <ID>
wok start      # Shows: Usage: wok start <ID>
wok label      # Shows: Usage: wok label <ID> <LABEL>
```

This provides actionable guidance instead of cryptic "missing argument" errors.
