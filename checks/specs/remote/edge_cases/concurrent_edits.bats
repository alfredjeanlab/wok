#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Concurrent Edit Handling
# ============================================================================

@test "last write wins for same-field edits" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates issue
    cd "$dir_a"
    local id
    id=$(create_issue task "Original title")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B syncs and edits title
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Original title" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" edit "$b_id" title "B title"
    assert_success

    # Small delay to ensure HLC ordering
    sleep 0.1

    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A also edits title (later timestamp wins)
    cd "$dir_a"
    sleep 0.1
    run "$WK_BIN" edit "$id" title "A title"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B syncs and should see A's title (later write)
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" show "$b_id"
    assert_success
    assert_output --partial "A title"
}

@test "independent fields don't conflict" {
    start_server

    local dir_a="$TEST_DIR/client_a"
    local dir_b="$TEST_DIR/client_b"
    mkdir -p "$dir_a" "$dir_b"

    cd "$dir_a"
    init_remote_project "prja"

    cd "$dir_b"
    init_remote_project "prjb"

    # A creates issue
    cd "$dir_a"
    local id
    id=$(create_issue task "Field test")
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B edits title
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    local b_id
    b_id=$("$WK_BIN" list --all 2>/dev/null | grep "Field test" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    run "$WK_BIN" edit "$b_id" title "New title from B"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # A changes status (independent of title)
    cd "$dir_a"
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced
    run "$WK_BIN" remote stop
    assert_success

    # B syncs - both changes should be present
    cd "$dir_b"
    run "$WK_BIN" remote sync
    assert_success

    run "$WK_BIN" show "$b_id"
    assert_success
    assert_output --partial "New title from B"
    assert_output --partial "in_progress"
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
