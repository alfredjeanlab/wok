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

@test "start transitions todo to in_progress" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: in_progress"
}

@test "reopen transitions in_progress to todo" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" reopen "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: todo"
}

@test "done transitions in_progress to done" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" done "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: done"
}

@test "close requires --reason" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" close "$id"
    assert_failure
}

@test "close with --reason succeeds from todo" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" close "$id" --reason "duplicate"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: closed"
}

@test "close with --reason succeeds from in_progress" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" close "$id" --reason "abandoned"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: closed"
}

@test "reopen requires --reason" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id"
    assert_failure
}

@test "reopen with --reason succeeds from done" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "regression found"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: todo"
}

@test "reopen with --reason succeeds from closed" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "duplicate"
    run "$WK_BIN" reopen "$id" --reason "not actually a duplicate"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: todo"
}

@test "cannot reopen from todo" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" reopen "$id"
    assert_failure
}

@test "cannot done from todo without reason" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" done "$id"
    assert_failure
}

@test "done with --reason succeeds from todo (prior)" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" done "$id" --reason "already completed"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: done"
}

@test "cannot start from done" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" start "$id"
    assert_failure
}

@test "cannot start from closed" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "duplicate"
    run "$WK_BIN" start "$id"
    assert_failure
}

# === Reason Notes ===

@test "close --reason creates note in Close Reason section" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "duplicate of other-123"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Close Reason:"
    assert_output --partial "duplicate of other-123"
}

@test "done --reason from todo creates note in Summary section" {
    id=$(create_issue task "Test task")
    "$WK_BIN" done "$id" --reason "already completed upstream"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Summary:"
    assert_output --partial "already completed upstream"
}

@test "reopen --reason creates note in Description section" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" reopen "$id" --reason "regression found in v2"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "regression found in v2"
}

# === Batch Operations ===

@test "start transitions multiple from todo to in_progress" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    run "$WK_BIN" start "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: in_progress"
}

@test "reopen transitions multiple from in_progress to todo" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    "$WK_BIN" start "$id1"
    "$WK_BIN" start "$id2"
    run "$WK_BIN" reopen "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: todo"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: todo"
}

@test "done transitions multiple from in_progress to done" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    "$WK_BIN" start "$id1"
    "$WK_BIN" start "$id2"
    run "$WK_BIN" done "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: done"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: done"
}

@test "done with --reason transitions multiple from todo to done" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    run "$WK_BIN" done "$id1" "$id2" --reason "already completed"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: done"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: done"
}

@test "close with --reason closes multiple" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    run "$WK_BIN" close "$id1" "$id2" --reason "duplicate"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: closed"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: closed"
}

@test "reopen with --reason reopens multiple" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    "$WK_BIN" start "$id1"
    "$WK_BIN" start "$id2"
    "$WK_BIN" done "$id1"
    "$WK_BIN" done "$id2"
    run "$WK_BIN" reopen "$id1" "$id2" --reason "regression"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: todo"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: todo"
}

@test "batch start fails on invalid status" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" start "$id1" "$id2"
    assert_failure
}
