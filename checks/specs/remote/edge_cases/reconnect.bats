#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Reconnection Behavior
# ============================================================================

@test "daemon reconnects after server restart" {
    start_server
    init_remote_project

    # Connect
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    # Remember port for restart
    local port="$SERVER_PORT"
    local data_dir="$TEST_DIR/server_data"

    # Stop server
    stop_server

    # Wait for daemon to detect disconnect
    wait_daemon_disconnected

    # Restart server on same port
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" &
    SERVER_PID=$!
    SERVER_PORT=$port
    SERVER_URL="ws://127.0.0.1:$port"

    wait_server_ready "$port"

    # Daemon should reconnect
    wait_daemon_connected

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "connected"
}

@test "daemon status shows disconnected when server down" {
    start_server
    init_remote_project

    # Connect
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    # Stop server
    stop_server

    # Wait for disconnect detection
    wait_daemon_disconnected

    run "$WK_BIN" remote status
    assert_output --partial "disconnected"
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
