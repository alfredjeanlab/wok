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

@test "log shows activity and issue history" {
    # Log shows recent activity
    create_issue task "LogBasic Task 1"
    create_issue task "LogBasic Task 2"
    run "$WK_BIN" log
    assert_success
    assert_output --partial "created"

    # Log for specific issue shows its history
    id=$(create_issue task "LogBasic Specific task")
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "created"

    # Log with empty database shows nothing or message
    run "$WK_BIN" log
    assert_success
}

@test "log shows lifecycle events" {
    # Start event
    id=$(create_issue task "LogEvent Start task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "started"

    # Reopen event without reason from in_progress
    id=$(create_issue task "LogEvent Reopen inprog task")
    "$WK_BIN" start "$id"
    "$WK_BIN" reopen "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "reopened"

    # Done event
    id=$(create_issue task "LogEvent Done task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "done"

    # Close event with reason
    id=$(create_issue task "LogEvent Close task")
    "$WK_BIN" close "$id" --reason "duplicate"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "closed"
    assert_output --partial "duplicate"

    # Reopen event with reason
    id=$(create_issue task "LogEvent Reopen done task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" reopen "$id" --reason "regression"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "reopened"
    assert_output --partial "regression"
}

@test "log shows label and note events" {
    # Labeled event
    id=$(create_issue task "LogMeta Label task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "labeled"

    # Noted event
    id=$(create_issue task "LogMeta Note task")
    "$WK_BIN" note "$id" "My note"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "noted"
}

@test "log options and error handling" {
    # --limit restricts output
    id=$(create_issue task "LogOpts Limit task")
    "$WK_BIN" start "$id"
    "$WK_BIN" reopen "$id"
    "$WK_BIN" start "$id"
    run "$WK_BIN" log --limit 2
    assert_success

    # Nonexistent issue fails
    run "$WK_BIN" log "test-nonexistent"
    assert_failure

    # Rejects -l shorthand
    run "$WK_BIN" log -l 5
    assert_failure
    assert_output --partial "unexpected argument '-l'"
}
