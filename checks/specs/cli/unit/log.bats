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

@test "log shows recent activity" {
    create_issue task "Task 1"
    create_issue task "Task 2"
    run "$WK_BIN" log
    assert_success
    assert_output --partial "created"
}

@test "log for specific issue shows its history" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "created"
}

@test "log shows start event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "started"
}

@test "log shows reopen event without reason from in_progress" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" reopen "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "reopened"
}

@test "log shows done event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "done"
}

@test "log shows close event with reason" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "duplicate"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "closed"
    assert_output --partial "duplicate"
}

@test "log shows reopen event with reason" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" reopen "$id" --reason "regression"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "reopened"
    assert_output --partial "regression"
}

@test "log shows labeled event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "labeled"
}

@test "log shows noted event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "My note"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "noted"
}

@test "log with --limit restricts output" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" reopen "$id"
    "$WK_BIN" start "$id"
    run "$WK_BIN" log --limit 2
    assert_success
}

@test "log for nonexistent issue fails" {
    run "$WK_BIN" log "test-nonexistent"
    assert_failure
}

@test "log with empty database shows nothing or message" {
    run "$WK_BIN" log
    assert_success
}

@test "log rejects -l shorthand" {
    run "$WK_BIN" log -l 5
    assert_failure
    assert_output --partial "unexpected argument '-l'"
}
