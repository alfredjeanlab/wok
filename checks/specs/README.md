# wk Test Suite

A portable test suite that validates the `wk` CLI against REQUIREMENTS.md.

## Requirements

- Bash 4.0+
- BATS (system-installed or downloaded locally)
- **Optional:** GNU parallel or [rush](https://github.com/shenwei356/rush) for parallel test execution

## Setup

BATS can be used from a system installation or downloaded locally:

```bash
# Option 1: System bats (preferred for faster setup)
brew install bats-core bats-assert bats-support  # macOS

# Option 2: Local installation (automatic)
make test-init
```

The Makefile automatically detects system bats and uses it when available.

## Running Tests

```bash
# Build the CLI first
(cd crates/cli && cargo build --release)

# Run all specs (recommended)
make spec

# Run specific spec groups
make spec-cli              # All CLI specs
make spec-remote           # All remote specs
make spec-cli-unit         # CLI unit tests only
make spec-cli-integration  # CLI integration tests only
make spec-cli-edge-cases   # CLI edge cases only

# Check bats configuration
make bats-check

# Test with specific binary
WK_BIN=crates/cli/target/release/wk make spec
```

### Using run.sh directly

The legacy `run.sh` script is still available:

```bash
# Test the default wk in PATH
./run.sh

# Test the built binary
WK_BIN=$(pwd)/../../crates/cli/target/release/wk ./run.sh

# Run specific test file
./run.sh unit/init.bats

# Force serial execution
./run.sh --jobs 1

# Run with specific parallelism (requires parallel or rush)
./run.sh --jobs 8

# Verbose output
./run.sh --verbose

# TAP output for CI
./run.sh --formatter tap
```

## Performance

The test suite uses file-level setup to reduce overhead and supports parallel execution.

| Mode | Time | Notes |
|------|------|-------|
| Serial | ~50s | Default without parallel binary |
| Parallel (4 jobs) | ~15s | Requires GNU parallel or rush |

To install parallel for faster tests:
```bash
# macOS
brew install parallel

# Ubuntu/Debian
sudo apt install parallel
```

## Test Structure

```
specs/
├── bats/                     # BATS framework (local installation)
│   ├── install.sh            # Downloads bats libraries
│   ├── bats-core/            # Downloaded by install.sh
│   ├── bats-assert/          # Downloaded by install.sh
│   └── bats-support/         # Downloaded by install.sh
├── helpers/
│   └── common.bash           # Shared setup/teardown utilities
├── unit/                     # Per-command tests
│   ├── help.bats
│   ├── init.bats
│   ├── new.bats
│   ├── lifecycle.bats
│   ├── list.bats
│   ├── show.bats
│   ├── dep.bats
│   ├── label.bats
│   ├── note.bats
│   ├── log.bats
│   ├── edit.bats
│   ├── tree.bats
│   ├── export.bats
│   └── sync.bats
├── integration/              # Cross-feature tests
│   ├── workflow.bats
│   ├── state_machine.bats
│   ├── dependencies.bats
│   └── filtering.bats
├── edge_cases/               # Edge case and error tests
│   ├── cycles.bats
│   ├── collisions.bats
│   ├── special_chars.bats
│   ├── empty_db.bats
│   └── errors.bats
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `WK_BIN` | Path to wk executable | `wk` (searches PATH) |

## Writing Tests

Tests use BATS with bats-assert and bats-support helpers.

### File-Level Setup (Recommended)

Most tests should use file-level setup to share a single `wk init` across all tests in a file:

```bash
#!/usr/bin/env bats
load '../helpers/common'

setup_file() {
    file_setup           # Creates temp dir, sets HOME
    init_project_once    # Initialize once for all tests
}

teardown_file() {
    file_teardown        # Cleanup temp dir
}

setup() {
    test_setup           # Reset to file temp dir before each test
}

@test "example test" {
    run "$WK_BIN" new "Test task"
    assert_success
    assert_output --partial "[task]"
}
```

### Per-Test Setup (For Isolation)

Use per-test setup only for tests that need a clean slate (e.g., testing init itself):

```bash
#!/usr/bin/env bats
load '../helpers/common'

# Default setup/teardown from common.bash handles per-test isolation

@test "example test needing isolation" {
    init_project
    run "$WK_BIN" new "Test task"
    assert_success
}
```

### Available Helpers

**Per-test helpers:**
- `init_project [prefix]` - Initialize a test project (per-test)
- `create_issue <type> <title> [options...]` - Create issue and return ID
- `get_status <id>` - Get issue status

**File-level helpers:**
- `file_setup` - Create temp directory for file-level tests
- `file_teardown` - Cleanup file-level temp directory
- `test_setup` - Reset to file temp dir before each test
- `init_project_once [prefix]` - Initialize project once (idempotent)

