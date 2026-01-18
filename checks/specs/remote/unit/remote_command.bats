#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# remote sync Behavior
# ============================================================================

@test "remote sync forces immediate sync and reports success" {
    start_server
    init_remote_project

    run "$WK_BIN" remote sync
    assert_success
    assert_output --partial "Sync complete"
}

@test "remote sync works with no pending changes" {
    start_server
    init_remote_project

    # Sync with empty database
    run "$WK_BIN" remote sync
    assert_success
    assert_output --partial "0 operations synced"
}

@test "remote sync reports number of ops synced" {
    start_server
    init_remote_project

    # Create some issues
    create_issue task "Issue 1"
    create_issue task "Issue 2"

    run "$WK_BIN" remote sync
    assert_success
    # Should report synced operations
    assert_output --regexp "[0-9]+ operations synced"
}

@test "remote sync fails gracefully when server unreachable" {
    # Initialize with unreachable server
    run "$WK_BIN" init --prefix test --local
    assert_success

    cat >> .wok/config.toml << 'EOF'

[remote]
url = "ws://127.0.0.1:19999"
EOF
    configure_fast_timeouts

    run "$WK_BIN" remote sync
    # Should not crash, but report error
    assert_output --regexp "fail|error|unreachable|refused"
}

# ============================================================================
# remote status Behavior
# ============================================================================

@test "remote status shows connected state and server URL" {
    start_server
    init_remote_project

    # Start and connect
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "connected"
    assert_output --partial "$SERVER_URL"
}

@test "remote status shows pending operation count" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "Pending ops:"
}

@test "remote status shows zero pending when fully synced" {
    start_server
    init_remote_project

    # Sync
    run "$WK_BIN" remote sync
    assert_success

    # Wait for full sync
    wait_synced

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "Pending ops: 0"
}

@test "remote status shows disconnected when server down" {
    start_server
    init_remote_project

    # Start daemon and connect
    run "$WK_BIN" remote sync
    assert_success
    wait_daemon_connected

    # Stop server
    stop_server

    # Wait for disconnection detection
    wait_daemon_disconnected

    run "$WK_BIN" remote status
    assert_output --partial "disconnected"
}

@test "remote status shows not configured in local-only project" {
    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "not applicable"
}

# ============================================================================
# remote stop Behavior
# ============================================================================

@test "remote stop stops the daemon process" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    local pid
    pid=$(get_daemon_pid)
    [ -n "$pid" ]

    # Stop
    run "$WK_BIN" remote stop
    assert_success

    # Process should no longer exist
    sleep 0.1
    run ps -p "$pid"
    assert_failure
}

@test "remote status shows not running after stop" {
    start_server
    init_remote_project

    # Start then stop
    run "$WK_BIN" remote sync
    assert_success
    run "$WK_BIN" remote stop
    assert_success

    run "$WK_BIN" remote status
    assert_output --partial "not running"
}

@test "next sync-requiring command respawns daemon after stop" {
    start_server
    init_remote_project

    # Start, stop, start again
    run "$WK_BIN" remote sync
    assert_success

    local pid1
    pid1=$(get_daemon_pid)

    run "$WK_BIN" remote stop
    assert_success

    run "$WK_BIN" remote sync
    assert_success

    local pid2
    pid2=$(get_daemon_pid)

    # Should be a new PID
    [ "$pid1" != "$pid2" ]
}

# ============================================================================
# Daemon Recovery
# ============================================================================

@test "daemon restart recovers from stale daemon" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    local pid1
    pid1=$(get_daemon_pid)
    [ -n "$pid1" ]

    # Kill daemon forcefully (simulates crash or version mismatch scenario)
    kill -9 "$pid1" 2>/dev/null || true
    sleep 0.2

    # Next command should recover and spawn new daemon
    run "$WK_BIN" remote sync
    assert_success

    local pid2
    pid2=$(get_daemon_pid)
    [ -n "$pid2" ]

    # Should be a new process
    [ "$pid1" != "$pid2" ]
}

@test "daemon responds to version handshake" {
    start_server
    init_remote_project

    # Start daemon via sync
    run "$WK_BIN" remote sync
    assert_success

    # Status should work (implicitly tests daemon communication)
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "daemon running"
}

# ============================================================================
# Error Handling
# ============================================================================

@test "invalid server URL fails clearly" {
    run "$WK_BIN" init --prefix test --local
    assert_success

    cat >> .wok/config.toml << 'EOF'

[remote]
url = "not-a-valid-url"
EOF

    run "$WK_BIN" remote sync
    assert_failure
}

@test "malformed config shows parse error" {
    run "$WK_BIN" init --prefix test --local
    assert_success

    # Add malformed TOML
    echo "this is not valid toml @@@@" >> .wok/config.toml

    run "$WK_BIN" remote status
    assert_failure
}

# ============================================================================
# Timing and Ordering
# ============================================================================

@test "remote status shows last sync timestamp" {
    start_server
    init_remote_project

    # Sync
    run "$WK_BIN" remote sync
    assert_success

    # Wait a bit
    sleep 0.1

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "Last sync:"
}

@test "rapid successive syncs don't corrupt state" {
    start_server
    init_remote_project

    # Create some data
    create_issue task "Test issue"

    # Rapid fire syncs
    for _ in 1 2 3 4 5; do
        run "$WK_BIN" remote sync
        assert_success
    done

    # Verify state is still valid
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Test issue"
}

@test "operation order preserved during sync" {
    start_server
    init_remote_project

    # Create issue
    local id
    id=$(create_issue task "Ordered test")

    # Start
    run "$WK_BIN" start "$id"
    assert_success

    # Complete
    run "$WK_BIN" done "$id"
    assert_success

    # Sync
    run "$WK_BIN" remote sync
    assert_success

    # Verify final state
    local status
    status=$(get_status "$id")
    [ "$status" = "(done)" ]
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
