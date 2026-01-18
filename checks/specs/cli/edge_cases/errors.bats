#!/usr/bin/env bats
load '../../helpers/common'

# Error handling tests - uses per-test setup for isolation

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

@test "commands without initialization fail with helpful message" {
    [ ! -d ".wok" ]

    run "$WK_BIN" list
    assert_failure

    run "$WK_BIN" new "Test"
    assert_failure

    run "$WK_BIN" show "test-abc"
    assert_failure
}

@test "invalid issue IDs fail" {
    init_project

    run "$WK_BIN" show "invalid-id-format"
    assert_failure

    run "$WK_BIN" start "not-an-id"
    assert_failure

    b=$(create_issue task "Task B")
    run "$WK_BIN" dep "invalid" blocks "$b"
    assert_failure

    a=$(create_issue task "Task A")
    run "$WK_BIN" dep "$a" blocks "invalid"
    assert_failure
}

@test "missing required arguments fail" {
    init_project

    # new without title
    run "$WK_BIN" new
    assert_failure

    # note without content
    id=$(create_issue task "Test")
    run "$WK_BIN" note "$id"
    assert_failure

    # dep without relationship
    a=$(create_issue task "A")
    b=$(create_issue task "B")
    run "$WK_BIN" dep "$a" "$b"
    assert_failure

    # close without reason
    id2=$(create_issue task "Test2")
    run "$WK_BIN" close "$id2"
    assert_failure

    # reopen without reason (from done)
    id3=$(create_issue task "Test3")
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    run "$WK_BIN" reopen "$id3"
    assert_failure
}

@test "invalid values fail" {
    init_project

    # new with invalid type
    run "$WK_BIN" new invalid "Test"
    assert_failure

    # edit with invalid type
    id=$(create_issue task "Test")
    run "$WK_BIN" edit "$id" --type invalid
    assert_failure

    # dep with invalid relationship
    a=$(create_issue task "A")
    b=$(create_issue task "B")
    run "$WK_BIN" dep "$a" requires "$b"
    assert_failure

    # list with invalid status
    run "$WK_BIN" list --status invalid
    assert_failure

    # list with invalid type
    run "$WK_BIN" list --type invalid
    assert_failure
}

@test "exit codes: success returns 0, not found and invalid return non-zero" {
    init_project

    # success returns exit code 0
    run "$WK_BIN" new "Test"
    [ "$status" -eq 0 ]

    # not found returns non-zero exit code
    run "$WK_BIN" show "test-nonexistent"
    [ "$status" -ne 0 ]

    # invalid command returns non-zero exit code
    run "$WK_BIN" nonexistent
    [ "$status" -ne 0 ]
}

@test "removed commands (sync/daemon) are not available" {
    # help does not list sync as a command
    run "$WK_BIN" help
    assert_success
    refute_line --regexp '^  sync[[:space:]]'
    refute_line --regexp '^  daemon[[:space:]]'

    # Commands are not recognized
    run "$WK_BIN" sync
    assert_failure

    run "$WK_BIN" daemon
    assert_failure
}

@test "new fails when project has no prefix configured" {
    mkdir -p workspace
    "$WK_BIN" init --path workspace --prefix ws
    run "$WK_BIN" init --workspace workspace
    assert_success
    run "$WK_BIN" new "Test task"
    assert_failure
    assert_output --partial "no prefix configured"
}
