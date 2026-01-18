#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Offline Queue Behavior
# ============================================================================

@test "operations queue locally when server unavailable" {
    # Initialize with unreachable server
    run "$WK_BIN" init --prefix test --local
    assert_success

    cat >> .wok/config.toml << 'EOF'

[remote]
url = "ws://127.0.0.1:19999"
EOF
    configure_fast_timeouts

    # Create issues - should be stored locally
    create_issue task "Offline issue 1"
    create_issue task "Offline issue 2"

    # Try to sync (server unreachable)
    run "$WK_BIN" remote sync
    # May fail or report connection error

    # Stop daemon if started
    "$WK_BIN" remote stop 2>/dev/null || true

    # Issues should still be in local database
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Offline issue 1"
    assert_output --partial "Offline issue 2"
}

@test "queued ops flush when server becomes available" {
    # Start with server down
    run "$WK_BIN" init --prefix test --local
    assert_success

    # Configure with a port we'll start later
    local port
    port=$(find_free_port)

    cat >> .wok/config.toml << EOF

[remote]
url = "ws://127.0.0.1:$port"
EOF
    configure_fast_timeouts

    # Create issues offline
    create_issue task "Queue flush test"

    # Try sync (will fail)
    "$WK_BIN" remote sync 2>/dev/null || true
    "$WK_BIN" remote stop 2>/dev/null || true

    # Start server
    local data_dir="$TEST_DIR/server_data"
    mkdir -p "$data_dir"
    "$WK_REMOTE_BIN" --bind "127.0.0.1:$port" --data "$data_dir" >"$TEST_DIR/server.log" 2>&1 &
    SERVER_PID=$!
    SERVER_PORT=$port
    SERVER_URL="ws://127.0.0.1:$port"
    disown "$SERVER_PID" 2>/dev/null || true

    wait_server_ready "$port"

    # Sync should now succeed
    run "$WK_BIN" remote sync
    assert_success

    # Should report synced operations
    run "$WK_BIN" remote status
    assert_success
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
