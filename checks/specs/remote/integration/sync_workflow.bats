#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Full Lifecycle Sync
# ============================================================================

@test "full issue lifecycle syncs between clients" {
    start_server

    # Setup clients
    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates and starts issue
    cd "$dir_a"
    local id
    id=$(create_issue task "Lifecycle sync test")
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B completes the issue
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    # Find the issue ID in B's database
    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Lifecycle sync test" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" done "$b_id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs and sees done status
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    local status
    status=$(get_status "$id")
    [ "$status" = "(done)" ]
}

@test "notes accumulate correctly across clients" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates issue with note
    cd "$dir_a"
    local id
    id=$(create_issue task "Notes test")
    run "$WK_BIN" note "$id" "Note from A"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B adds another note
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Notes test" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" note "$b_id" "Note from B"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs and sees both notes
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Note from A"
    assert_output --partial "Note from B"
}

@test "labels merge correctly across clients" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates issue with label
    cd "$dir_a"
    local id
    id=$(create_issue task "Labels test" --label frontend)
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B adds another label
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Labels test" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" label "$b_id" backend
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs and sees both labels
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "frontend"
    assert_output --partial "backend"
}

@test "dependency chains sync correctly" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates parent and first child
    cd "$dir_a"
    local feature_id
    feature_id=$(create_issue feature "Parent feature")
    local task1_id
    task1_id=$(create_issue task "Task 1")
    run "$WK_BIN" dep "$task1_id" tracked-by "$feature_id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B adds second child
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_feature
    b_feature=$("$WK_BIN" list --all 2>/dev/null | grep "Parent feature" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    local task2_id
    task2_id=$(create_issue task "Task 2")
    run "$WK_BIN" dep "$task2_id" tracked-by "$b_feature"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs and sees full tree
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" tree "$feature_id"
    assert_success
    assert_output --partial "Task 1"
    assert_output --partial "Task 2"
}

@test "sync after offline period merges correctly" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    mkdir -p "$dir_a"
    cd "$dir_a"
    init_remote_project "prja"

    # Create and sync some issues
    local id1
    id1=$(create_issue task "Before offline")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # Stop server (simulate offline)
    stop_server

    # Make changes while offline (daemon will queue them)
    local id2
    id2=$(create_issue task "During offline")

    # Try to sync (should queue locally)
    run "$WK_BIN" remote sync
    # May fail or succeed with queuing

    # Stop any daemon
    "$WK_BIN" remote stop 2>/dev/null || true

    # Restart server
    start_server

    # Sync should succeed and merge
    run "$WK_BIN" remote sync
    assert_success

    # Both issues should be visible
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Before offline"
    assert_output --partial "During offline"
}

@test "bulk operations sync efficiently" {
    start_server
    init_remote_project

    # Create 20 issues
    for i in $(seq 1 20); do
        create_issue task "Bulk issue $i"
    done

    # Sync all at once
    run "$WK_BIN" remote sync
    assert_success

    # Wait for full sync
    wait_synced 200

    # Verify all synced (pending should be 0)
    local pending
    pending=$(get_pending_ops)
    [ "${pending:-1}" = "0" ]

    # All issues should be visible
    run "$WK_BIN" list --all
    assert_success

    local count
    count=$(echo "$output" | grep -c "Bulk issue" || echo 0)
    [ "$count" -eq 20 ]
}

@test "issue type changes sync correctly" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates task
    cd "$dir_a"
    local id
    id=$(create_issue task "Type change test")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B changes type to bug
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Type change test" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" edit "$b_id" type bug
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A syncs and sees type change
    cd "$dir_a"
    run "$WK_BIN" remote sync
    assert_success

    local type
    type=$(get_type "$id")
    [ "$type" = "[bug]" ]
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
