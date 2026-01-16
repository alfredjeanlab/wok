#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Server Startup and Binding
# ============================================================================

@test "server starts and binds to specified port" {
    local port
    port=$(find_free_port)
    local data_dir="$TEST_DIR/server_data"
    mkdir -p "$data_dir"

    # Start server
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" &
    local pid=$!

    # Wait for server to be ready
    wait_server_ready "$port"

    # Verify port is open
    nc -z 127.0.0.1 "$port"

    # Cleanup
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}

@test "server accepts WebSocket connections" {
    start_server

    # Verify we can connect (nc should succeed)
    run nc -z 127.0.0.1 "$SERVER_PORT"
    assert_success
}

@test "server creates data directory if needed" {
    local port
    port=$(find_free_port)
    local data_dir="$TEST_DIR/nested/deep/data"

    # Directory should not exist
    [ ! -d "$data_dir" ]

    # Start server with nested data directory
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" &
    local pid=$!

    # Wait for server to start
    wait_server_ready "$port"

    # Directory should now exist
    [ -d "$data_dir" ]

    # Cleanup
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}

@test "server stops cleanly on SIGTERM" {
    local port
    port=$(find_free_port)
    local data_dir="$TEST_DIR/server_data"
    mkdir -p "$data_dir"

    # Start server
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" &
    local pid=$!

    # Wait for server to be ready
    wait_server_ready "$port"

    # Send SIGTERM
    kill -TERM "$pid"
    wait "$pid" 2>/dev/null || true

    # Port should be released
    wait_port_released "$port"

    # Verify port is no longer in use
    run nc -z 127.0.0.1 "$port"
    assert_failure
}

@test "server rejects invalid bind address" {
    local data_dir="$TEST_DIR/server_data"
    mkdir -p "$data_dir"

    # Try to bind to invalid address
    run "$WK_REMOTE_BIN" --bind "invalid:address" --data "$data_dir"
    assert_failure
}

# ============================================================================
# Setup / Teardown
# ============================================================================

setup() {
    setup_remote
}

teardown() {
    teardown_remote
}
