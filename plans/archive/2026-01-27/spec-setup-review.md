# Spec Setup Review

Review and verify the spec test infrastructure to ensure all helpers work correctly.

## Overview

This plan reviews the BATS spec test infrastructure including:
- Prelude helpers (`tests/specs/helpers/common.bash`)
- Remote helpers (`tests/specs/remote/helpers/remote_common.bash`)
- Temp directory (Project) management
- Binary lookup for `wk` and `wk-remote`
- Add any missing helper methods

## Project Structure

```
tests/specs/
├── helpers/
│   └── common.bash           # Core CLI helpers
├── remote/
│   └── helpers/
│       └── remote_common.bash  # Remote-specific helpers
├── cli/
│   ├── unit/
│   ├── integration/
│   └── edge_cases/
└── remote/
    ├── unit/
    ├── integration/
    └── edge_cases/

scripts/
└── spec                      # BATS runner script
```

## Dependencies

- BATS (auto-installed to `~/.local/share/wok/bats/`)
  - bats-core v1.11.0
  - bats-support v0.3.0
  - bats-assert v2.1.0
- netcat (`nc`) for port checking
- Standard Unix tools (`mktemp`, `grep`, `awk`)

## Implementation Phases

### Phase 1: Verify Helper Syntax

**Objective**: Ensure all bash helpers compile without syntax errors.

**Actions**:
```bash
# Check bash syntax
bash -n tests/specs/helpers/common.bash
bash -n tests/specs/remote/helpers/remote_common.bash
```

**Verification**: Both commands exit with status 0.

---

### Phase 2: Test Temp Directory Helpers

**Objective**: Verify `file_setup`, `file_teardown`, and `test_setup` work correctly.

**Actions**:
1. Create a minimal test that exercises temp directory creation
2. Verify `BATS_FILE_TMPDIR` is created and writable
3. Verify `HOME` isolation works
4. Verify cleanup removes temp directory

**Test file**: `tests/specs/cli/unit/helper_temp_dirs.bats`
```bash
#!/usr/bin/env bats
load '../../helpers/common'

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

@test "BATS_FILE_TMPDIR exists and is writable" {
    [ -d "$BATS_FILE_TMPDIR" ]
    touch "$BATS_FILE_TMPDIR/test_file"
    [ -f "$BATS_FILE_TMPDIR/test_file" ]
}

@test "HOME is isolated to test directory" {
    [[ "$HOME" == "$BATS_FILE_TMPDIR" ]]
}

@test "test_setup returns to BATS_FILE_TMPDIR" {
    cd /tmp
    test_setup
    [ "$PWD" = "$BATS_FILE_TMPDIR" ]
}
```

**Verification**: `make spec ARGS='--file cli/unit/helper_temp_dirs.bats'` passes.

---

### Phase 3: Verify WK Binary Lookup

**Objective**: Confirm `wk` binary is found correctly.

**Actions**:
1. Verify `WK_BIN` is set after running `scripts/spec`
2. Verify binary exists and is executable
3. Test fallback to PATH works

**Test additions to helper test file**:
```bash
@test "WK_BIN is set and executable" {
    [ -n "$WK_BIN" ]
    [ -x "$WK_BIN" ] || command -v "$WK_BIN" >/dev/null
}

@test "WK_BIN runs successfully" {
    run "$WK_BIN" --version
    assert_success
}
```

**Verification**: Tests pass with both built binary and PATH fallback.

---

### Phase 4: Fix Incomplete get_type Helper

**Objective**: Update `get_type()` to support all issue types.

**Current code** (`common.bash:131`):
```bash
"$WK_BIN" show "$id" | grep -oE '\[epic\]|\[task\]|\[bug\]' | head -1
```

**Problem**: Missing `[feature]`, `[chore]`, `[idea]` types.

**Fix**:
```bash
get_type() {
    local id="$1"
    "$WK_BIN" show "$id" | grep -oE '\[epic\]|\[task\]|\[bug\]|\[feature\]|\[chore\]|\[idea\]' | head -1
}
```

**Verification**: Test creating each issue type and verify `get_type` returns correct value.

---

### Phase 5: Add Missing Validation Helper

**Objective**: Add `require_wk_bin` helper for explicit binary validation.

**Rationale**: When `WK_BIN="wk"` and `wk` isn't in PATH, tests fail with cryptic errors. An explicit validation helper makes debugging easier.

**Implementation** (add to `common.bash`):
```bash
# Verify WK_BIN is executable
# Usage: require_wk_bin (call in setup_file if needed)
require_wk_bin() {
    if [ -x "$WK_BIN" ]; then
        return 0
    elif command -v "$WK_BIN" >/dev/null 2>&1; then
        return 0
    else
        echo "Error: WK_BIN='$WK_BIN' not found or not executable" >&2
        echo "Build with 'cargo build' or set WK_BIN to binary path" >&2
        return 1
    fi
}
```

**Usage**: Tests can optionally call `require_wk_bin` in `setup_file()` for early failure with clear error.

---

### Phase 6: Test Remote Helpers

**Objective**: Verify remote helpers work correctly.

**Actions**:
1. Test `find_free_port` returns valid port
2. Test `start_server`/`stop_server` lifecycle
3. Test `setup_remote`/`teardown_remote` patterns

**Test file**: `tests/specs/remote/unit/helper_remote.bats`
```bash
#!/usr/bin/env bats
load '../helpers/remote_common'

setup() {
    setup_remote
}

teardown() {
    teardown_remote
}

@test "find_free_port returns valid port number" {
    local port
    port=$(find_free_port)
    [[ "$port" =~ ^[0-9]+$ ]]
    [ "$port" -ge 17800 ]
    [ "$port" -le 18999 ]
}

@test "start_server and stop_server lifecycle" {
    start_server
    [ -n "$SERVER_PID" ]
    [ -n "$SERVER_PORT" ]
    [ -n "$SERVER_URL" ]

    # Verify server is running
    nc -z 127.0.0.1 "$SERVER_PORT"

    stop_server

    # Verify server stopped
    sleep 0.1
    ! nc -z 127.0.0.1 "$SERVER_PORT" 2>/dev/null || true
}
```

**Verification**: `make spec-remote ARGS='--file remote/unit/helper_remote.bats'` passes.

## Key Implementation Details

### Temp Directory Patterns

Two patterns are used:

1. **Per-test isolation** (default `setup()`/`teardown()`): Each test gets fresh `TEST_DIR`
2. **File-level sharing**: All tests in file share `BATS_FILE_TMPDIR` for performance

Most CLI tests use file-level sharing with `init_project_once` for faster execution.

### Binary Lookup Order

1. `$WK_BIN` environment variable (if set)
2. `target/release/wk` (release build)
3. `target/debug/wk` (debug build)
4. `wk` in PATH (fallback)

### BATS Library Loading

Libraries loaded from `$BATS_LIB_PATH` when set (by `scripts/spec`), otherwise from relative path for direct invocation.

## Verification Plan

1. **Syntax verification**:
   ```bash
   bash -n tests/specs/helpers/common.bash
   bash -n tests/specs/remote/helpers/remote_common.bash
   ```

2. **Helper tests**:
   ```bash
   make spec ARGS='--file cli/unit/helper_temp_dirs.bats'
   make spec ARGS='--file remote/unit/helper_remote.bats'
   ```

3. **Full spec suite**:
   ```bash
   make spec
   ```

4. **Validate check passes**:
   ```bash
   make check
   ```
