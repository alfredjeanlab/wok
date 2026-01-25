#!/usr/bin/env bash

# Load parent common helpers
load '../../helpers/common'

# Default to searching PATH if WK_REMOTE_BIN not set
export WK_REMOTE_BIN="${WK_REMOTE_BIN:-wk-remote}"

# Port range for parallel test execution (17800-18999)
PORT_RANGE_START=17800
PORT_RANGE_END=18999

# Polling interval for wait functions (milliseconds)
POLL_INTERVAL_MS=10

# ============================================================================
# PORT MANAGEMENT
# ============================================================================

# Find an available port in the test range
# Usage: port=$(find_free_port)
find_free_port() {
    local port
    for port in $(shuf -i $PORT_RANGE_START-$PORT_RANGE_END -n 100); do
        if ! nc -z 127.0.0.1 "$port" 2>/dev/null; then
            echo "$port"
            return 0
        fi
    done
    echo "Error: Could not find free port in range $PORT_RANGE_START-$PORT_RANGE_END" >&2
    return 1
}

# Wait until a port is released
# Usage: wait_port_released PORT [max_attempts]
wait_port_released() {
    local port="$1"
    local max_attempts="${2:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if ! nc -z 127.0.0.1 "$port" 2>/dev/null; then
            return 0
        fi
        sleep 0.01
        ((attempt++))
    done

    echo "Error: Port $port not released after $max_attempts attempts" >&2
    return 1
}

# ============================================================================
# PROCESS HELPERS
# ============================================================================

# Kill a process gracefully (SIGTERM), then force (SIGKILL) if still running
# Usage: kill_with_timeout PID [timeout_ms]
# timeout_ms defaults to 1000 (1 second)
kill_with_timeout() {
    local pid="$1"
    local timeout_ms="${2:-1000}"

    # Check if process exists
    if ! kill -0 "$pid" 2>/dev/null; then
        return 0
    fi

    # Send SIGTERM for graceful shutdown
    kill "$pid" 2>/dev/null || return 0

    # Wait for exit with timeout (in 10ms increments)
    local attempts=$((timeout_ms / 10))
    local i
    for ((i = 0; i < attempts; i++)); do
        if ! kill -0 "$pid" 2>/dev/null; then
            return 0
        fi
        sleep 0.01
    done

    # Force kill if still running
    if kill -0 "$pid" 2>/dev/null; then
        kill -9 "$pid" 2>/dev/null || true
        sleep 0.05
    fi
}

# ============================================================================
# SERVER MANAGEMENT
# ============================================================================

# Start wk-remote server
# Usage: start_server [data_dir]
# Exports: SERVER_PID, SERVER_PORT, SERVER_URL, SERVER_LOG
# Note: If SERVER_PORT is already set, reuses that port (useful for testing reconnection)
start_server() {
    local data_dir="${1:-$TEST_DIR/server_data}"

    mkdir -p "$data_dir"

    # Use existing SERVER_PORT if set, otherwise find a free one
    if [ -z "${SERVER_PORT:-}" ]; then
        SERVER_PORT=$(find_free_port)
    fi
    SERVER_URL="ws://127.0.0.1:$SERVER_PORT"
    SERVER_LOG="${TEST_DIR}/server.log"

    # Start server in background, redirect output to log file
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$SERVER_PORT" --data "$data_dir" >"$SERVER_LOG" 2>&1 &
    SERVER_PID=$!
    disown "$SERVER_PID" 2>/dev/null || true

    export SERVER_PID SERVER_PORT SERVER_URL SERVER_LOG

    # Wait for server to be ready (accepting connections)
    wait_server_ready "$SERVER_PORT" || {
        kill "$SERVER_PID" 2>/dev/null || true
        return 1
    }
}

# Wait for server to accept connections
# Usage: wait_server_ready PORT [max_attempts]
wait_server_ready() {
    local port="$1"
    local max_attempts="${2:-200}"  # Increased for CI reliability
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        # Use -w1 for 1-second connect timeout (works on both macOS and Linux netcat)
        # This prevents nc from returning immediately on ECONNREFUSED
        if nc -z -w1 127.0.0.1 "$port" 2>/dev/null; then
            return 0
        fi
        sleep 0.02  # Slightly longer sleep for CI environments
        ((attempt++))
    done

    echo "Error: Server not ready on port $port after $max_attempts attempts" >&2
    return 1
}

# Stop wk-remote server
# Usage: stop_server
stop_server() {
    if [ -n "${SERVER_PID:-}" ]; then
        local port="${SERVER_PORT:-}"

        # Kill server with 500ms timeout before force kill
        kill_with_timeout "$SERVER_PID" 500 2>/dev/null || true

        # Brief wait for port release (don't block long)
        if [ -n "$port" ]; then
            wait_port_released "$port" 20 || true
        fi
        unset SERVER_PID SERVER_PORT SERVER_URL
    fi
}

# Restart server on the same port (preserves client configuration)
# Usage: restart_server [data_dir]
# Uses previously bound port, or finds a new one if none set
restart_server() {
    local data_dir="${1:-$TEST_DIR/server_data}"
    local port="${SERVER_PORT:-}"

    # Stop current server if running
    stop_server

    # If we had a port, wait for it to be released then reuse it
    if [ -n "$port" ]; then
        wait_port_released "$port" 50 || true
        SERVER_PORT="$port"
    else
        SERVER_PORT=$(find_free_port)
    fi

    SERVER_URL="ws://127.0.0.1:$SERVER_PORT"
    SERVER_LOG="${TEST_DIR}/server.log"

    # Start server in background, redirect output to log file
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$SERVER_PORT" --data "$data_dir" >>"$SERVER_LOG" 2>&1 &
    SERVER_PID=$!
    disown "$SERVER_PID" 2>/dev/null || true

    export SERVER_PID SERVER_PORT SERVER_URL SERVER_LOG

    # Wait for server to be ready
    wait_server_ready "$SERVER_PORT" || {
        kill "$SERVER_PID" 2>/dev/null || true
        return 1
    }
}

# ============================================================================
# PROJECT INITIALIZATION
# ============================================================================

# Initialize a remote-enabled project
# Usage: init_remote_project [prefix] [server_url]
# Uses SERVER_URL if not provided
init_remote_project() {
    local prefix="${1:-test}"
    local url="${2:-$SERVER_URL}"

    # Initialize the project with --local to avoid default git remote
    run "$WK_BIN" init --prefix "$prefix" --local
    assert_success

    # Add remote configuration
    cat >> .wok/config.toml << EOF

[remote]
url = "$url"
EOF
}

# Initialize a second client directory pointing to the same server
# Usage: init_second_client PREFIX DIR [server_url]
init_second_client() {
    local prefix="$1"
    local dir="$2"
    local url="${3:-$SERVER_URL}"

    mkdir -p "$dir"
    cd "$dir"

    run "$WK_BIN" init --prefix "$prefix" --local
    assert_success

    cat >> .wok/config.toml << EOF

[remote]
url = "$url"
EOF
}

# ============================================================================
# TEST TIMEOUT CONFIGURATION
# ============================================================================

# Configure fast timeouts for testing
# Must be called after init_remote_project to append to the [remote] section
# Usage: configure_fast_timeouts
configure_fast_timeouts() {
    cat >> .wok/config.toml << 'EOF'
reconnect_max_retries = 2
reconnect_max_delay_secs = 1
connect_timeout_secs = 1
heartbeat_interval_ms = 100
heartbeat_timeout_ms = 100
EOF
}

# ============================================================================
# DAEMON STATUS HELPERS
# ============================================================================

# Wait for daemon to reach connected state
# Usage: wait_daemon_connected [max_attempts]
wait_daemon_connected() {
    local max_attempts="${1:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        local output
        output=$("$WK_BIN" remote status 2>&1) || true
        if echo "$output" | grep -q "connected"; then
            return 0
        fi
        sleep 0.01
        ((attempt++))
    done

    echo "Error: Daemon not connected after $max_attempts attempts" >&2
    echo "Last status: $output" >&2
    return 1
}

# Wait for daemon to reach disconnected/offline state
# Usage: wait_daemon_disconnected [max_attempts]
wait_daemon_disconnected() {
    local max_attempts="${1:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        local output
        output=$("$WK_BIN" remote status 2>&1) || true
        if echo "$output" | grep -qE "disconnected|offline|not running"; then
            return 0
        fi
        sleep 0.01
        ((attempt++))
    done

    echo "Error: Daemon not disconnected after $max_attempts attempts" >&2
    echo "Last status: $output" >&2
    return 1
}

# Get daemon PID from status output
# Usage: pid=$(get_daemon_pid)
get_daemon_pid() {
    "$WK_BIN" remote status 2>&1 | grep -oE 'PID: [0-9]+' | grep -oE '[0-9]+' | head -1
}

# Check if daemon is running
# Usage: daemon_is_running
daemon_is_running() {
    local output
    output=$("$WK_BIN" remote status 2>&1) || true
    echo "$output" | grep -q "daemon running"
}

# ============================================================================
# SYNC HELPERS
# ============================================================================

# Trigger a sync and wait for completion
# Usage: sync_and_wait
sync_and_wait() {
    run "$WK_BIN" remote sync
    assert_success
}

# Get pending operation count
# Usage: count=$(get_pending_ops)
get_pending_ops() {
    "$WK_BIN" remote status 2>&1 | grep -oE 'Pending ops: [0-9]+' | grep -oE '[0-9]+' | head -1
}

# Wait for pending ops to reach zero
# Usage: wait_synced [max_attempts]
wait_synced() {
    local max_attempts="${1:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        local pending
        pending=$(get_pending_ops)
        if [ "${pending:-1}" = "0" ]; then
            return 0
        fi
        sleep 0.01
        ((attempt++))
    done

    echo "Error: Not fully synced after $max_attempts attempts (pending: $pending)" >&2
    return 1
}

# ============================================================================
# SETUP/TEARDOWN
# ============================================================================

# Setup for remote tests - call this from setup()
# Creates isolated test directory and HOME
setup_remote() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

# Stop a daemon by its PID file
# Usage: stop_daemon_by_pidfile PIDFILE
stop_daemon_by_pidfile() {
    local pidfile="$1"
    local daemon_pid

    if [ ! -f "$pidfile" ]; then
        return 0
    fi

    daemon_pid=$(cat "$pidfile" 2>/dev/null || true)
    if [ -z "$daemon_pid" ] || ! kill -0 "$daemon_pid" 2>/dev/null; then
        return 0
    fi

    # Try graceful stop with short timeout (500ms)
    kill "$daemon_pid" 2>/dev/null || return 0

    local i
    for ((i = 0; i < 50; i++)); do
        if ! kill -0 "$daemon_pid" 2>/dev/null; then
            return 0
        fi
        sleep 0.01
    done

    # Force kill if still running
    if kill -0 "$daemon_pid" 2>/dev/null; then
        kill -9 "$daemon_pid" 2>/dev/null || true
        sleep 0.05
    fi
}

# Teardown for remote tests - call this from teardown()
# Stops server and cleans up
# Uses short timeouts to avoid blocking if processes are unresponsive
teardown_remote() {
    # Stop server if running
    stop_server

    # Find and stop ALL daemons in test directory (handles multi-client tests)
    if [ -n "${TEST_DIR:-}" ] && [ -d "$TEST_DIR" ]; then
        while IFS= read -r -d '' pidfile; do
            stop_daemon_by_pidfile "$pidfile"
        done < <(find "$TEST_DIR" -name 'daemon.pid' -print0 2>/dev/null)
    fi

    # SAFETY NET: Kill any wk-remote processes started from our TEST_DIR
    # This catches processes spawned outside start_server helper
    if [ -n "${TEST_DIR:-}" ]; then
        local pids
        pids=$(pgrep -f "wk-remote.*--data.*${TEST_DIR}" 2>/dev/null || true)
        for pid in $pids; do
            kill -9 "$pid" 2>/dev/null || true
        done
    fi

    # Double-safety: kill any wk-remote on test port that we might own
    if [ -n "${SERVER_PORT:-}" ]; then
        local pid
        pid=$(lsof -ti tcp:"$SERVER_PORT" 2>/dev/null || true)
        [ -n "$pid" ] && kill -9 "$pid" 2>/dev/null || true
    fi

    # Cleanup test directory
    cd / || exit 1
    if [ -n "${TEST_DIR:-}" ] && [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
    fi
}

# ============================================================================
# ISSUE SYNC VERIFICATION
# ============================================================================

# Wait for an issue with given title to appear in list
# Usage: wait_for_issue TITLE [max_attempts]
wait_for_issue() {
    local title="$1"
    local max_attempts="${2:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if "$WK_BIN" list --all 2>/dev/null | grep -q "$title"; then
            return 0
        fi
        sleep 0.02  # Slightly longer sleep for CI environments
        ((attempt++))
    done

    echo "Error: Issue '$title' not found after $max_attempts attempts" >&2
    return 1
}

# Wait for an issue to have a specific status
# Usage: wait_for_status ID EXPECTED_STATUS [max_attempts]
wait_for_status() {
    local id="$1"
    local expected="$2"
    local max_attempts="${3:-100}"
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        local status
        status=$(get_status "$id" 2>/dev/null) || true
        if [ "$status" = "$expected" ]; then
            return 0
        fi
        sleep 0.02  # Slightly longer sleep for CI environments
        ((attempt++))
    done

    echo "Error: Issue '$id' status not '$expected' after $max_attempts attempts (got: $status)" >&2
    return 1
}

# Create an issue and wait for it to sync
# Usage: id=$(create_and_sync TYPE TITLE [extra args])
create_and_sync() {
    local type="$1"
    local title="$2"
    shift 2

    local id
    id=$(create_issue "$type" "$title" "$@")

    # Trigger sync
    run "$WK_BIN" remote sync
    assert_success

    echo "$id"
}

# Verify issue exists in list
# Usage: issue_exists ID
issue_exists() {
    local id="$1"
    "$WK_BIN" list --all 2>/dev/null | grep -q "$id"
}
