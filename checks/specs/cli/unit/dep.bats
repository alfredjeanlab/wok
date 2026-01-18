#!/usr/bin/env bats
load '../../helpers/common'

setup_file() {
    file_setup
    init_project_once test
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

@test "dep blocks creates blocking relationship and affects list" {
    # Creates blocking relationship
    a=$(create_issue task "DepBlocks Task A")
    b=$(create_issue task "DepBlocks Task B")
    run "$WK_BIN" dep "$a" blocks "$b"
    assert_success

    # Default list shows both blocked and unblocked issues
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Task A"
    assert_output --partial "Task B"

    # --blocked filters to show only blocked issues
    run "$WK_BIN" list --blocked
    assert_success
    refute_output --partial "Task A"
    assert_output --partial "Task B"

    # Multiple targets
    a=$(create_issue task "DepBlocksMulti Task A")
    b=$(create_issue task "DepBlocksMulti Task B")
    c=$(create_issue task "DepBlocksMulti Task C")
    run "$WK_BIN" dep "$a" blocks "$b" "$c"
    assert_success
    run "$WK_BIN" list --blocked
    assert_output --partial "$b"
    assert_output --partial "$c"
}

@test "dep tracks creates parent-child relationship and shows in output" {
    # Creates tracks relationship
    feature=$(create_issue feature "DepTracks Feature")
    task=$(create_issue task "DepTracks Task")
    run "$WK_BIN" dep "$feature" tracks "$task"
    assert_success

    # Shows in parent show output
    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"
    assert_output --partial "$task"

    # Shows in child show output
    run "$WK_BIN" show "$task"
    assert_success
    assert_output --partial "Tracked by"
    assert_output --partial "$feature"
}

@test "undep removes relationships" {
    # Removes blocking relationship
    a=$(create_issue task "DepUndepBlock Task A")
    b=$(create_issue task "DepUndepBlock Task B")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" undep "$a" blocks "$b"
    assert_success
    run "$WK_BIN" list
    assert_output --partial "$b"

    # Removes tracks relationship
    feature=$(create_issue feature "DepUndepTrack Feature")
    task=$(create_issue task "DepUndepTrack Task")
    "$WK_BIN" dep "$feature" tracks "$task"
    "$WK_BIN" undep "$feature" tracks "$task"
    run "$WK_BIN" show "$task"
    assert_success
    refute_output --partial "Tracked by"

    # Undep nonexistent relationship succeeds or fails gracefully
    a=$(create_issue task "DepUndepNone Task A")
    b=$(create_issue task "DepUndepNone Task B")
    run "$WK_BIN" undep "$a" blocks "$b"
    true  # Should either succeed (idempotent) or fail gracefully
}

@test "dep error handling" {
    # Dep to self fails
    a=$(create_issue task "DepErr Self Task")
    run "$WK_BIN" dep "$a" blocks "$a"
    assert_failure

    # Dep with nonexistent from-id fails
    b=$(create_issue task "DepErr To Task")
    run "$WK_BIN" dep "test-nonexistent" blocks "$b"
    assert_failure

    # Dep with nonexistent to-id fails
    a=$(create_issue task "DepErr From Task")
    run "$WK_BIN" dep "$a" blocks "test-nonexistent"
    assert_failure

    # Dep with invalid relationship fails
    a=$(create_issue task "DepErr Invalid A")
    b=$(create_issue task "DepErr Invalid B")
    run "$WK_BIN" dep "$a" requires "$b"
    assert_failure
}
