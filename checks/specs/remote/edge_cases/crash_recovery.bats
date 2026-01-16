#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Daemon Crash Recovery
# ============================================================================

@test "data survives daemon crash" {
    start_server
    init_remote_project

    # Create and sync data
    local id
    id=$(create_issue task "Crash test issue")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Get daemon PID and kill it hard
    local pid
    pid=$(get_daemon_pid)
    [ -n "$pid" ]

    kill -9 "$pid" 2>/dev/null || true
    sleep 0.2

    # Data should still be in local database
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Crash test issue"
}

@test "daemon auto-respawns after crash on next command" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    local pid1
    pid1=$(get_daemon_pid)

    # Kill daemon
    kill -9 "$pid1" 2>/dev/null || true
    sleep 0.2

    # Next sync should respawn
    run "$WK_BIN" remote sync
    assert_success

    local pid2
    pid2=$(get_daemon_pid)

    # Should be a new PID
    [ "$pid1" != "$pid2" ]
}

@test "stale socket files cleaned up on next command" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    local pid
    pid=$(get_daemon_pid)

    # Kill daemon hard (leaves stale socket)
    kill -9 "$pid" 2>/dev/null || true
    sleep 0.2

    # Next command should clean up and work
    run "$WK_BIN" remote status
    # May show "not running" or successfully restart
    assert_success
}

# ============================================================================
# Server Crash Recovery
# ============================================================================

@test "operations survive server crash and sync after restart" {
    start_server
    init_remote_project

    # Create some issues
    local id
    id=$(create_issue task "Pre-crash issue")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Remember port and data dir
    local port="$SERVER_PORT"
    local data_dir="$TEST_DIR/server_data"

    # Kill server hard
    kill -9 "$SERVER_PID" 2>/dev/null || true
    SERVER_PID=""
    sleep 0.2
    wait_port_released "$port"

    # Create more issues while server is down
    local id2
    id2=$(create_issue task "During-crash issue")

    # Stop daemon (can't sync anyway)
    "$WK_BIN" remote stop 2>/dev/null || true

    # Restart server
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" &
    SERVER_PID=$!
    SERVER_PORT=$port

    wait_server_ready "$port"

    # Sync should work and include queued operations
    run "$WK_BIN" remote sync
    assert_success

    # Both issues should be present locally
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Pre-crash issue"
    assert_output --partial "During-crash issue"
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
