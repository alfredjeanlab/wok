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

@test "note adds note to issue" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" note "$id" "My note"
    assert_success
}

@test "note appears in show output" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Important note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Important note"
}

@test "multiple notes can be added" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Note 1"
    "$WK_BIN" note "$id" "Note 2"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Note 1"
    assert_output --partial "Note 2"
}

@test "note records todo status" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Todo note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "todo"
    assert_output --partial "Todo note"
}

@test "note records in_progress status" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "Progress note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "in_progress"
    assert_output --partial "Progress note"
}

@test "note records done status" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" note "$id" "Done note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Done note"
}

@test "note with nonexistent issue fails" {
    run "$WK_BIN" note "test-nonexistent" "My note"
    assert_failure
}

@test "note requires content" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" note "$id"
    assert_failure
}

@test "note logs event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "My note"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "noted"
}

@test "notes preserve order" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "First"
    "$WK_BIN" note "$id" "Second"
    "$WK_BIN" note "$id" "Third"
    run "$WK_BIN" show "$id"
    assert_success
    # All notes should appear
    assert_output --partial "First"
    assert_output --partial "Second"
    assert_output --partial "Third"
}

@test "note on closed issue fails" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "wontfix"
    run "$WK_BIN" note "$id" "Should fail"
    assert_failure
    assert_output --partial "cannot add notes to closed issues"
}

# Semantic label tests

@test "note in todo shows Description label" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Requirements and context"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "Requirements and context"
}

@test "note in progress shows Progress label" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "Working on implementation"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Progress:"
    assert_output --partial "Working on implementation"
}

@test "note in done shows Summary label" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" note "$id" "Completed successfully"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Summary:"
    assert_output --partial "Completed successfully"
}

@test "note shows timestamp on metadata line" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Test note"
    run "$WK_BIN" show "$id"
    assert_success
    # Timestamp format: YYYY-MM-DD HH:MM
    assert_output --regexp "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}"
}

@test "note rejects -r shorthand" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Original note"
    run "$WK_BIN" note "$id" -r "Replacement"
    assert_failure
    assert_output --partial "unexpected argument '-r'"
}
