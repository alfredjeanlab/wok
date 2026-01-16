#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Local-Only Mode
# ============================================================================

@test "remote status shows not configured in local-only mode" {
    # Initialize without remote config
    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "not applicable"
    assert_output --partial "no remote configured"
}

# ============================================================================
# Daemon Auto-Spawn
# ============================================================================

@test "daemon auto-spawns on remote sync" {
    start_server
    init_remote_project

    # No daemon should be running yet
    run "$WK_BIN" remote status
    assert_output --partial "not running"

    # Trigger sync - should spawn daemon
    run "$WK_BIN" remote sync
    assert_success

    # Daemon should now be running
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "daemon running"
}

@test "remote status shows connection state and PID" {
    start_server
    init_remote_project

    # Start daemon via sync
    run "$WK_BIN" remote sync
    assert_success

    # Wait for connection
    wait_daemon_connected

    # Check status output
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "daemon running"
    assert_output --partial "PID:"
    assert_output --partial "connected"
}

@test "remote stop terminates daemon" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    # Verify running
    daemon_is_running

    # Stop daemon
    run "$WK_BIN" remote stop
    assert_success

    # Verify stopped
    run "$WK_BIN" remote status
    assert_output --partial "not running"
}

# ============================================================================
# Single Instance Enforcement
# ============================================================================

@test "only one daemon instance per database" {
    start_server
    init_remote_project

    # Start daemon
    run "$WK_BIN" remote sync
    assert_success

    # Get first PID
    local pid1
    pid1=$(get_daemon_pid)
    [ -n "$pid1" ]

    # Try to start another sync (should use existing daemon)
    run "$WK_BIN" remote sync
    assert_success

    # PID should be the same
    local pid2
    pid2=$(get_daemon_pid)
    [ "$pid1" = "$pid2" ]
}

# ============================================================================
# Shared Database Behavior
# ============================================================================

@test "two work dirs with shared database share one daemon" {
    start_server

    # Create a shared database location
    local shared_db="$TEST_DIR/shared"
    mkdir -p "$shared_db"

    # Create first .wok dir
    local dir_a="$TEST_DIR/project_a"
    mkdir -p "$dir_a"
    cd "$dir_a"
    run "$WK_BIN" init --prefix test --local
    assert_success

    # Configure workspace to use shared database
    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    # Create second .wok dir pointing to same database
    local dir_b="$TEST_DIR/project_b"
    mkdir -p "$dir_b"
    cd "$dir_b"
    run "$WK_BIN" init --prefix test --local
    assert_success

    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    # Start daemon from dir A
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    local pid_from_a
    pid_from_a=$(get_daemon_pid)
    [ -n "$pid_from_a" ]

    # Check status from dir B - should show same PID
    cd "$dir_b"
    local pid_from_b
    pid_from_b=$(get_daemon_pid)
    [ "$pid_from_a" = "$pid_from_b" ]
}

@test "daemon started from dir A visible from dir B with shared database" {
    start_server

    local shared_db="$TEST_DIR/shared"
    mkdir -p "$shared_db"

    # Setup two dirs with shared database
    local dir_a="$TEST_DIR/project_a"
    local dir_b="$TEST_DIR/project_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    run "$WK_BIN" init --prefix test --local
    assert_success
    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    cd "$dir_b"
    run "$WK_BIN" init --prefix test --local
    assert_success
    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    # Start daemon from A
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    # Verify single daemon process
    local pid
    pid=$(get_daemon_pid)
    local count
    count=$(ps aux | grep -c "[w]k.*daemon" || echo 0)

    # Status from B should see it
    cd "$dir_b"
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "daemon running"
    assert_output --partial "PID: $pid"
}

@test "remote stop from either dir stops shared daemon" {
    start_server

    local shared_db="$TEST_DIR/shared"
    mkdir -p "$shared_db"

    local dir_a="$TEST_DIR/project_a"
    local dir_b="$TEST_DIR/project_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    run "$WK_BIN" init --prefix test --local
    assert_success
    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    cd "$dir_b"
    run "$WK_BIN" init --prefix test --local
    assert_success
    cat >> .wok/config.toml << EOF
workspace = "$shared_db"

[remote]
url = "$SERVER_URL"
EOF

    # Start from A
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    # Stop from B
    cd "$dir_b"
    run "$WK_BIN" remote stop
    assert_success

    # Verify stopped from both dirs
    cd "$dir_a"
    run "$WK_BIN" remote status
    assert_output --partial "not running"

    cd "$dir_b"
    run "$WK_BIN" remote status
    assert_output --partial "not running"
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
