# wk Test Suite

A portable test suite that validates the `wk` CLI against REQUIREMENTS.md.

## Requirements

- Bash 4.0+
- BATS (system-installed or downloaded locally)

## Setup

BATS libraries are installed locally on first run:

```bash
# System bats-core (optional, slightly faster)
brew install bats-core  # macOS

# Libraries (bats-support, bats-assert) are auto-installed to checks/specs/bats/
```

The test runner automatically detects system bats and uses it when available, falling back to the local installation.

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

## Test Structure

```text
specs/
├── bats/                     # BATS framework (local installation)
│   ├── install.sh            # Downloads bats libraries
│   ├── bats-core/            # Downloaded by install.sh
│   ├── bats-assert/          # Downloaded by install.sh
│   └── bats-support/         # Downloaded by install.sh
├── helpers/
│   └── common.bash           # Shared setup/teardown utilities
├── cli/                      # CLI tests
│   ├── unit/                 # Per-command tests
│   ├── integration/          # Cross-feature tests
│   ├── edge_cases/           # Edge case and error tests
│   └── consistency/          # Flag consistency tests
└── remote/                   # Remote/sync tests
    ├── unit/                 # Remote command tests
    ├── integration/          # Multi-client tests
    ├── edge_cases/           # Recovery and conflict tests
    └── helpers/              # Remote test utilities
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

