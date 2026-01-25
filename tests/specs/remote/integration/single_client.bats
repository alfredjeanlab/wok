#!/usr/bin/env bats
load '../helpers/remote_common'

# ============================================================================
# Basic Issue Sync
# ============================================================================

@test "created issue syncs to server" {
    start_server
    init_remote_project

    # Create issue
    local id
    id=$(create_issue task "Test sync issue")

    # Sync to server
    run "$WK_BIN" remote sync
    assert_success

    # Verify synced (pending should be 0)
    wait_synced

    # Issue should still be accessible
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Test sync issue"
}

@test "issue lifecycle syncs correctly" {
    start_server
    init_remote_project

    # Create
    local id
    id=$(create_issue task "Lifecycle test")
    run "$WK_BIN" remote sync
    assert_success

    # Start
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success

    local status
    status=$(get_status "$id")
    [ "$status" = "(in_progress)" ]

    # Done
    run "$WK_BIN" done "$id"
    assert_success
    run "$WK_BIN" remote sync
    assert_success

    status=$(get_status "$id")
    [ "$status" = "(done)" ]
}

@test "labels sync correctly" {
    start_server
    init_remote_project

    # Create with labels
    local id
    id=$(create_issue task "Labeled issue" --label priority --label urgent)

    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Verify labels are present
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority"
    assert_output --partial "urgent"
}

@test "notes sync correctly" {
    start_server
    init_remote_project

    # Create issue
    local id
    id=$(create_issue task "Issue with notes")
    run "$WK_BIN" remote sync
    assert_success

    # Add note
    run "$WK_BIN" note "$id" "This is a test note"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Verify note is present
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "This is a test note"
}

@test "dependencies sync correctly" {
    start_server
    init_remote_project

    # Create parent and child
    local parent_id
    local child_id
    parent_id=$(create_issue feature "Parent feature")
    child_id=$(create_issue task "Child task")

    # Create dependency
    run "$WK_BIN" dep "$child_id" tracked-by "$parent_id"
    assert_success

    run "$WK_BIN" remote sync
    assert_success
    wait_synced

    # Verify dependency exists
    run "$WK_BIN" tree "$parent_id"
    assert_success
    assert_output --partial "$child_id"
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
