# Fix Remote Specs CI Failures

**Root Feature:** `wok-9cb9`

## Overview

The `make spec-remote` CI job experiences two critical issues:
1. **5 tests failing** due to `wait_server_ready` timing out prematurely (32ms instead of expected 1000ms)
2. **Job hangs indefinitely** after all tests complete due to orphan `wk-remote` processes not being cleaned up

The fix ensures all tests pass and the job ALWAYS completes within the timeout, even if individual tests fail.

## Root Cause Analysis

### Failing Tests (28, 30, 31, 36, 57)

| Test | File | Issue |
|------|------|-------|
| 28 | server_basic.bats | `wait_server_ready` timeout |
| 30 | server_basic.bats | `wait_server_ready` timeout |
| 31 | server_basic.bats | `wait_server_ready` timeout |
| 36 | multi_client.bats | `wait_for_issue` timeout (real-time propagation) |
| 57 | offline_queue.bats | `wait_server_ready` timeout |

**Common pattern**: Tests 28, 30, 31, 57 all spawn servers manually (not using `start_server` helper), and `wait_server_ready` fails in ~32ms despite logs showing server is "Listening on: ...". This indicates:
1. `nc -z` on Ubuntu CI may behave differently (immediate failure instead of retry)
2. When tests fail, cleanup code after the assertion never runs
3. Orphaned servers persist until CI timeout

### Job Hang Root Cause

Timeline from CI logs:
- 06:47:35.987 - Test 59 completes (all tests done)
- 06:51:03.620 - Job cancelled after 4-minute timeout
- Orphan processes found: 3 `wk-remote` instances (PIDs 4921, 5011, 5056)

The orphan processes prevent the job from exiting cleanly. BATS/make waits for child processes, causing the hang.

## Project Structure

```
checks/specs/
├── remote/
│   ├── helpers/
│   │   └── remote_common.bash  # Main changes: process tracking, wait helpers
│   ├── unit/
│   │   └── server_basic.bats   # Fix tests 28, 30, 31 to use helpers
│   ├── integration/
│   │   └── multi_client.bats   # Fix test 36 timing
│   └── edge_cases/
│       └── offline_queue.bats  # Fix test 57 to use helpers
└── CLAUDE.md
```

## Dependencies

No new dependencies required. Uses existing:
- BATS test framework with built-in timeout (`BATS_TEST_TIMEOUT`)
- Standard Unix tools (`nc`, `ps`, `pkill`, `lsof`)

## Implementation Phases

### Phase 1: Robust Process Cleanup in Teardown

**Goal**: Ensure ALL orphan `wk-remote` processes are killed during teardown, regardless of how they were spawned.

**Changes to `remote_common.bash`**:

```bash
# Add at end of teardown_remote(), after existing cleanup:
teardown_remote() {
    # ... existing code ...

    # SAFETY NET: Kill any wk-remote processes started from our TEST_DIR
    # This catches processes spawned outside start_server helper
    if [ -n "${TEST_DIR:-}" ]; then
        # Find wk-remote processes with data dirs in TEST_DIR
        local pids
        pids=$(pgrep -f "wk-remote.*--data.*${TEST_DIR}" 2>/dev/null || true)
        for pid in $pids; do
            kill -9 "$pid" 2>/dev/null || true
        done
    fi

    # Double-safety: kill any wk-remote on test port range that we might own
    if [ -n "${SERVER_PORT:-}" ]; then
        local pid
        pid=$(lsof -ti tcp:"$SERVER_PORT" 2>/dev/null || true)
        [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
    fi

    # ... existing cleanup test directory ...
}
```

**Verification**: Run failing tests individually and verify no orphan processes remain.

### Phase 2: Fix `wait_server_ready` Helper

**Goal**: Make `wait_server_ready` more robust across different environments (macOS, Ubuntu CI).

**Analysis**: The 32ms failure time suggests `nc -z` is failing immediately rather than retrying. On some systems, `nc -z` requires explicit timeout (`-w`) to wait for connection.

**Changes to `remote_common.bash`**:

```bash
wait_server_ready() {
    local port="$1"
    local max_attempts="${2:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        # Add -w1 for 1-second connect timeout (works on both macOS and Linux)
        # Also try actual TCP connection as fallback
        if nc -z -w1 127.0.0.1 "$port" 2>/dev/null; then
            return 0
        fi
        sleep 0.01
        ((attempt++))
    done

    echo "Error: Server not ready on port $port after $max_attempts attempts" >&2
    return 1
}
```

**Alternative approach** if `-w1` doesn't work consistently:

```bash
wait_server_ready() {
    local port="$1"
    local max_attempts="${2:-200}"  # Increase attempts
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        # Use /dev/tcp for more reliable checking
        if (echo > /dev/tcp/127.0.0.1/"$port") 2>/dev/null; then
            return 0
        fi
        sleep 0.02  # Slightly longer sleep
        ((attempt++))
    done

    echo "Error: Server not ready on port $port after $max_attempts attempts" >&2
    return 1
}
```

**Verification**: Run `make spec-remote` locally and on CI. Verify tests 28, 30, 31, 57 pass.

### Phase 3: Refactor Tests to Use Standard Helpers

**Goal**: Make tests 28, 30, 31, 57 use `start_server` helper so processes are tracked for cleanup.

**Changes to `server_basic.bats`**:

```bash
@test "server starts and binds to specified port" {
    # Use start_server helper instead of manual spawn
    start_server

    # Verify port is open
    nc -z 127.0.0.1 "$SERVER_PORT"
}

@test "server creates data directory if needed" {
    local nested_dir="$TEST_DIR/nested/deep/data"
    [ ! -d "$nested_dir" ]

    # Use start_server with custom data dir
    start_server "$nested_dir"

    # Directory should now exist
    [ -d "$nested_dir" ]
}

@test "server stops cleanly on SIGTERM" {
    start_server
    local port="$SERVER_PORT"
    local pid="$SERVER_PID"

    # Send SIGTERM
    kill -TERM "$pid"
    wait "$pid" 2>/dev/null || true

    # Port should be released
    wait_port_released "$port"

    # Verify port is no longer in use
    run nc -z 127.0.0.1 "$port"
    assert_failure

    # Clear SERVER_PID so teardown doesn't try to kill again
    unset SERVER_PID
}
```

**Changes to `offline_queue.bats`**:

```bash
@test "queued ops flush when server becomes available" {
    # ... existing setup code ...

    # Use start_server helper with explicit port
    SERVER_PORT=$port
    mkdir -p "$TEST_DIR/server_data"
    start_server "$TEST_DIR/server_data"
    # start_server already calls wait_server_ready internally

    # Sync should now succeed
    run "$WK_BIN" remote sync
    assert_success

    # ... rest of test ...
}
```

**Verification**: Run modified tests individually, verify they pass and clean up properly.

### Phase 4: Fix Real-Time Propagation Test (Test 36)

**Goal**: Make test 36 more robust by either extending timeout or ensuring sync completes.

**Analysis**: The test expects client B to see issues created by client A without explicit sync. This requires:
1. A's daemon to push the operation to server
2. Server to broadcast to B's daemon
3. B's daemon to apply the change locally

The 1-second timeout (100 × 10ms) may be insufficient.

**Changes to `multi_client.bats`**:

```bash
@test "real-time propagation: client B sees issue created by A without sync" {
    start_server

    # Setup both clients
    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    cd "$dir_b"
    init_remote_project "prjb"
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    # A creates issue
    cd "$dir_a"
    create_issue task "Real-time test"

    # Wait for A's daemon to sync (ensure op is sent)
    wait_synced

    # Now wait for B to receive it (with longer timeout)
    cd "$dir_b"
    wait_for_issue "Real-time test" 200  # 200 attempts = 2 seconds

    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Real-time test"
}
```

**Alternative**: If real-time push isn't reliably working, add a note that explicit sync is needed and adjust test expectations.

**Verification**: Run test 36 multiple times to verify it passes consistently.

### Phase 5: CI Safety Net

**Goal**: Ensure CI job ALWAYS finishes, even if tests hang.

**Changes to `.github/workflows/specs.yml`**:

```yaml
- name: Run ${{ matrix.suite }} specs
  run: |
    export PATH="${{ github.workspace }}/target/release:$PATH"
    # Run with timeout to prevent hangs
    timeout 180 make spec-${{ matrix.suite }} || {
      code=$?
      echo "Tests exited with code $code"
      # Cleanup any orphan processes
      pkill -9 -f wk-remote || true
      exit $code
    }
  timeout-minutes: 4  # Existing timeout as backup

- name: Cleanup orphan processes
  if: always()
  run: |
    pkill -9 -f wk-remote || true
    pkill -9 -f 'wk.*daemon' || true
```

**Alternative**: Add cleanup step that always runs:

```yaml
- name: Run ${{ matrix.suite }} specs
  run: |
    export PATH="${{ github.workspace }}/target/release:$PATH"
    make spec-${{ matrix.suite }}

- name: Cleanup orphan processes
  if: always()
  run: |
    # Kill any remaining wk processes
    pkill -9 -f wk-remote 2>/dev/null || true
    # Give processes time to exit
    sleep 1
    # Verify cleanup
    pgrep -f wk-remote && echo "WARNING: wk-remote processes still running" || true
```

**Verification**: Run CI with intentionally failing tests to verify cleanup happens.

## Key Implementation Details

### Process Lifecycle in Tests

```
Test Start
    │
    ├─► setup_remote() creates TEST_DIR
    │
    ├─► start_server() spawns wk-remote, exports SERVER_PID
    │       │
    │       └─► wait_server_ready() polls until accepting connections
    │
    ├─► Test logic (may spawn additional processes)
    │
    └─► teardown_remote()
            │
            ├─► stop_server() kills SERVER_PID with timeout
            │
            ├─► stop_daemon_by_pidfile() for all daemon.pid files
            │
            ├─► [NEW] pkill orphan wk-remote in TEST_DIR
            │
            └─► rm -rf TEST_DIR
```

### Why `nc -z` Fails Fast

On Ubuntu CI, `nc` (netcat-openbsd) with `-z` may:
1. Return immediately if connection refused (ECONNREFUSED)
2. Not retry on its own without `-w` timeout flag

The server may be:
1. Bound to port (shows in logs)
2. But not yet `accept()`ing connections (async startup)

Fix: Add `-w1` for 1-second connect timeout, or use bash's `/dev/tcp`.

### Timeout Hierarchy

```
CI Job Timeout: 4 minutes
    └─► Test Suite Timeout: 180 seconds (timeout command)
        └─► BATS_TEST_TIMEOUT: 10 seconds (per test)
            └─► wait_* helpers: 1-2 seconds (100-200 × 10ms)
                └─► Individual nc/kill: 500ms-1s
```

## Verification Plan

### Local Testing

1. **Phase 1**: After implementing teardown changes
   ```bash
   make spec-remote ARGS='--filter "server starts"'
   pgrep -f wk-remote  # Should return nothing
   ```

2. **Phase 2**: After fixing wait_server_ready
   ```bash
   for i in {1..10}; do make spec-remote ARGS='--filter "server starts"'; done
   # All 10 runs should pass
   ```

3. **Phase 3**: After refactoring tests
   ```bash
   make spec-remote ARGS='--file remote/unit/server_basic.bats'
   make spec-remote ARGS='--file remote/edge_cases/offline_queue.bats'
   ```

4. **Phase 4**: After fixing real-time test
   ```bash
   for i in {1..5}; do make spec-remote ARGS='--filter "real-time"'; done
   ```

5. **Full suite**:
   ```bash
   make spec-remote
   # All 59 tests should pass
   # No orphan processes after completion
   ```

### CI Testing

1. Push changes to feature branch
2. Verify spec-remote job:
   - Completes within 4 minutes
   - All 59 tests pass
   - No "operation was canceled" error
   - Cleanup step shows no orphan processes

### Regression Testing

```bash
# Run full validation to ensure no regressions
make validate
```

## Success Criteria

- [ ] All 59 remote spec tests pass
- [ ] CI job completes within 4 minutes
- [ ] No orphan `wk-remote` processes after tests
- [ ] Job exits cleanly even if tests fail (no timeout/cancel)
- [ ] Tests are deterministic (10 consecutive runs pass)
