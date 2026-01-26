#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Multi-Client Sync
# ============================================================================

@test "client B sees issue created by client A" {
    start_server

    # Setup client A
    local dir_a="$TEST_DIR/client_a"
    mkdir -p "$dir_a"
    cd "$dir_a"
    init_remote_project "prja"

    # Create issue from A
    local id
    id=$(create_issue task "From client A")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Stop A's daemon
    run "$WK_BIN" remote stop
    assert_success

    # Setup client B
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_b"
    cd "$dir_b"
    init_remote_project "prjb"

    # Sync B to get data from server
    run "$WK_BIN" remote sync
    assert_success

    # B should see the issue
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "From client A"
}

@test "status changes broadcast to other clients" {
    start_server

    # Setup client A
    local dir_a="$TEST_DIR/client_a"
    mkdir -p "$dir_a"
    cd "$dir_a"
    init_remote_project "prja"

    # Create issue from A
    local id
    id=$(create_issue task "Shared issue")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Stop A's daemon
    run "$WK_BIN" remote stop
    assert_success

    # Setup client B
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_b"
    cd "$dir_b"
    init_remote_project "prjb"

    # Sync B
    run "$WK_BIN" remote sync
    assert_success

    # B changes status
    # First find the issue in B's list
    local issue_line
    issue_line=$("$WK_BIN" list --all 2>/dev/null | grep "Shared issue")
    local b_id
    b_id=$(echo "$issue_line" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" start "$b_id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Stop B's daemon
    run "$WK_BIN" remote stop
    assert_success

    # Back to A - sync to get update
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    # A should see the status change
    local status
    status=$(get_status "$id")
    [ "$status" = "(in_progress)" ]
}

@test "multiple clients creating issues concurrently see all issues" {
    start_server

    # Setup client A
    local dir_a="$TEST_DIR/client_a"
    mkdir -p "$dir_a"
    cd "$dir_a"
    init_remote_project "prja"

    # Setup client B
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_b"
    cd "$dir_b"
    init_remote_project "prjb"

    # A creates issues
    cd "$dir_a"
    create_issue task "A issue 1"
    create_issue task "A issue 2"
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B creates issues
    cd "$dir_b"
    create_issue task "B issue 1"
    create_issue task "B issue 2"
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs again
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    # A should see all 4 issues
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "A issue 1"
    assert_output --partial "A issue 2"
    assert_output --partial "B issue 1"
    assert_output --partial "B issue 2"

    run "$WK_BIN" remote stop
    assert_success

    # B syncs again
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    # B should also see all 4 issues
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "A issue 1"
    assert_output --partial "A issue 2"
    assert_output --partial "B issue 1"
    assert_output --partial "B issue 2"
}

# ============================================================================
# Real-Time Propagation (without explicit sync)
# ============================================================================

@test "real-time propagation: client B sees issue created by A without sync" {
    # Requires: CLI commands must generate ops for daemon to send to server
    start_server

    # Setup both clients, both connected
    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"
    run "$WK_BIN" remote sync  # Initial connect
    assert_success
    wait_daemon_connected

    cd "$dir_b"
    init_remote_project "prjb"
    run "$WK_BIN" remote sync  # Initial connect
    assert_success
    wait_daemon_connected

    # A creates issue (daemon sends to server)
    cd "$dir_a"
    local id
    id=$(create_issue task "Real-time test")

    # Wait for A's daemon to sync (ensure op is sent to server)
    wait_synced

    # Wait for propagation to B (with longer timeout for CI reliability)
    cd "$dir_b"
    wait_for_issue "Real-time test" 200  # 200 attempts @ 50ms = ~10 seconds

    # B should see the issue without calling sync
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Real-time test"
}

@test "real-time propagation: status changes appear on other client" {
    # Requires: CLI commands must generate ops for daemon to send to server
    start_server

    # Setup and sync both clients
    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"
    local id
    id=$(create_issue task "Status broadcast test")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    cd "$dir_b"
    init_remote_project "prjb"
    run "$WK_BIN" remote sync
    assert_success

    # B starts the issue
    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Status broadcast" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)
    run "$WK_BIN" start "$b_id"
    assert_success

    # Wait for propagation - A should see status change without explicit sync
    cd "$dir_a"
    wait_for_status "$id" "(in_progress)"
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
