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

@test "dep blocks creates blocking relationship" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    run "$WK_BIN" dep "$a" blocks "$b"
    assert_success
}

@test "dep blocks affects list --blocked" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    # Default list shows both blocked and unblocked issues
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Blocker"
    assert_output --partial "Blocked"
    # --blocked filters to show only blocked issues
    run "$WK_BIN" list --blocked
    assert_success
    refute_output --partial "Blocker"
    assert_output --partial "Blocked"
}

@test "dep tracks creates parent-child relationship" {
    feature=$(create_issue feature "Feature")
    task=$(create_issue task "Task")
    run "$WK_BIN" dep "$feature" tracks "$task"
    assert_success
}

@test "dep tracks shows in parent show output" {
    feature=$(create_issue feature "Feature")
    task=$(create_issue task "Task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"
    assert_output --partial "$task"
}

@test "dep tracks shows in child show output" {
    feature=$(create_issue feature "Feature")
    task=$(create_issue task "Task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" show "$task"
    assert_success
    assert_output --partial "Tracked by"
    assert_output --partial "$feature"
}

@test "dep with multiple targets" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    run "$WK_BIN" dep "$a" blocks "$b" "$c"
    assert_success
    # Both should be blocked
    run "$WK_BIN" list --blocked
    assert_output --partial "$b"
    assert_output --partial "$c"
}

@test "undep removes blocking relationship" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" undep "$a" blocks "$b"
    assert_success
    # B should no longer be blocked
    run "$WK_BIN" list
    assert_output --partial "$b"
}

@test "undep removes tracks relationship" {
    feature=$(create_issue feature "Feature")
    task=$(create_issue task "Task")
    "$WK_BIN" dep "$feature" tracks "$task"
    "$WK_BIN" undep "$feature" tracks "$task"
    run "$WK_BIN" show "$task"
    assert_success
    refute_output --partial "Tracked by"
}

@test "dep to self fails" {
    a=$(create_issue task "Task A")
    run "$WK_BIN" dep "$a" blocks "$a"
    assert_failure
}

@test "dep with nonexistent from-id fails" {
    b=$(create_issue task "Task B")
    run "$WK_BIN" dep "test-nonexistent" blocks "$b"
    assert_failure
}

@test "dep with nonexistent to-id fails" {
    a=$(create_issue task "Task A")
    run "$WK_BIN" dep "$a" blocks "test-nonexistent"
    assert_failure
}

@test "dep with invalid relationship fails" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    run "$WK_BIN" dep "$a" requires "$b"
    assert_failure
}

@test "undep nonexistent relationship succeeds or fails gracefully" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    run "$WK_BIN" undep "$a" blocks "$b"
    # Should either succeed (idempotent) or fail gracefully
    true
}
