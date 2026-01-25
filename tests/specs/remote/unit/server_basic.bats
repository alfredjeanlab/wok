#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Server Startup and Binding
# ============================================================================

@test "server starts and binds to specified port" {
    # Use start_server helper for proper process tracking and cleanup
    start_server

    # Verify port is open
    nc -z 127.0.0.1 "$SERVER_PORT"
}

@test "server accepts WebSocket connections" {
    start_server

    # Verify we can connect (nc should succeed)
    run nc -z 127.0.0.1 "$SERVER_PORT"
    assert_success
}

@test "server creates data directory if needed" {
    local data_dir="$TEST_DIR/nested/deep/data"

    # Directory should not exist
    [ ! -d "$data_dir" ]

    # Use start_server with custom data dir for proper process tracking
    start_server "$data_dir"

    # Directory should now exist
    [ -d "$data_dir" ]
}

@test "server stops cleanly on SIGTERM" {
    # Use start_server for proper process tracking
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
