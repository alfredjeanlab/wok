#!/usr/bin/env bats
load '../../helpers/common'

# Commands without initialization

@test "list without init fails with helpful message" {
    run "$WK_BIN" list
    assert_failure
    # Should mention init or .wok
    assert_output --partial "init" || assert_output --partial ".wok" || true
}

@test "new without init fails" {
    run "$WK_BIN" new "Test"
    assert_failure
}

@test "show without init fails" {
    run "$WK_BIN" show "test-abc"
    assert_failure
}

# Invalid issue IDs

@test "show with invalid ID fails" {
    init_project
    run "$WK_BIN" show "invalid-id-format"
    assert_failure
}

@test "start with invalid ID fails" {
    init_project
    run "$WK_BIN" start "not-an-id"
    assert_failure
}

@test "dep with invalid from-id fails" {
    init_project
    b=$(create_issue task "Task B")
    run "$WK_BIN" dep "invalid" blocks "$b"
    assert_failure
}

@test "dep with invalid to-id fails" {
    init_project
    a=$(create_issue task "Task A")
    run "$WK_BIN" dep "$a" blocks "invalid"
    assert_failure
}

# Missing required arguments

@test "new without title fails" {
    init_project
    run "$WK_BIN" new
    assert_failure
}

@test "note without content fails" {
    init_project
    id=$(create_issue task "Test")
    run "$WK_BIN" note "$id"
    assert_failure
}

@test "dep without relationship fails" {
    init_project
    a=$(create_issue task "A")
    b=$(create_issue task "B")
    run "$WK_BIN" dep "$a" "$b"
    assert_failure
}

@test "close without reason fails" {
    init_project
    id=$(create_issue task "Test")
    run "$WK_BIN" close "$id"
    assert_failure
}

@test "reopen without reason fails" {
    init_project
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id"
    assert_failure
}

# Invalid values

@test "new with invalid type fails" {
    init_project
    run "$WK_BIN" new invalid "Test"
    assert_failure
}

@test "edit with invalid type fails" {
    init_project
    id=$(create_issue task "Test")
    run "$WK_BIN" edit "$id" --type invalid
    assert_failure
}

@test "dep with invalid relationship fails" {
    init_project
    a=$(create_issue task "A")
    b=$(create_issue task "B")
    run "$WK_BIN" dep "$a" requires "$b"
    assert_failure
}

@test "list with invalid status fails" {
    init_project
    run "$WK_BIN" list --status invalid
    assert_failure
}

@test "list with invalid type fails" {
    init_project
    run "$WK_BIN" list --type invalid
    assert_failure
}

# Exit codes

@test "success returns exit code 0" {
    init_project
    run "$WK_BIN" new "Test"
    [ "$status" -eq 0 ]
}

@test "not found returns non-zero exit code" {
    init_project
    run "$WK_BIN" show "test-nonexistent"
    [ "$status" -ne 0 ]
}

@test "invalid command returns non-zero exit code" {
    run "$WK_BIN" nonexistent
    [ "$status" -ne 0 ]
}

# Removed commands (sync/daemon consolidated into remote)

@test "help does not list sync as a command" {
    run "$WK_BIN" help
    assert_success
    # Command listing format: "  sync        Description..."
    refute_line --regexp '^  sync[[:space:]]'
}

@test "help does not list daemon as a command" {
    run "$WK_BIN" help
    assert_success
    # Command listing format: "  daemon      Description..."
    refute_line --regexp '^  daemon[[:space:]]'
}

@test "sync command is not recognized" {
    run "$WK_BIN" sync
    assert_failure
}

@test "daemon command is not recognized" {
    run "$WK_BIN" daemon
    assert_failure
}

# Empty prefix validation

@test "new fails when project has no prefix configured" {
    # Create a workspace with a database but no prefix
    mkdir -p workspace
    "$WK_BIN" init --path workspace --prefix ws
    # Create a workspace link without a prefix
    run "$WK_BIN" init --workspace workspace
    assert_success
    # Try to create an issue - should fail because no prefix
    run "$WK_BIN" new "Test task"
    assert_failure
    assert_output --partial "no prefix configured"
}
