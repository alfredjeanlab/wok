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

@test "show displays issue details" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" show "$id"
    assert_success
    # First line: [type] id
    assert_line --index 0 --partial "[task] $id"
    # Separate lines for metadata
    assert_output --partial "Title: Test task"
    assert_output --partial "Status: todo"
    assert_output --partial "Created:"
    assert_output --partial "Updated:"
}

@test "show displays labels" {
    id=$(create_issue task "Labeled task" --label "project:auth")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"
}

@test "show displays multiple labels" {
    id=$(create_issue task "Multi-labeled" --label "label1" --label "label2")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"
}

@test "show displays notes" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "My note content"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "My note content"
}

@test "show groups notes by status" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Todo note"
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "In progress note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Todo note"
    assert_output --partial "In progress note"
}

@test "show displays blockers" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" show "$b"
    assert_success
    assert_output --partial "Blocked by"
    assert_output --partial "$a"
}

@test "show displays blocking relationships" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" show "$a"
    assert_success
    assert_output --partial "Blocks"
    assert_output --partial "$b"
}

@test "show displays parent relationship" {
    feature=$(create_issue feature "Parent feature")
    task=$(create_issue task "Child task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" show "$task"
    assert_success
    assert_output --partial "Tracked by"
    assert_output --partial "$feature"
}

@test "show displays children" {
    feature=$(create_issue feature "Parent feature")
    task=$(create_issue task "Child task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"
    assert_output --partial "$task"
}

@test "show displays log" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Log:"
    assert_output --partial "started"
}

@test "show nonexistent issue fails" {
    run "$WK_BIN" show "test-nonexistent"
    assert_failure
}

@test "show requires issue ID" {
    run "$WK_BIN" show
    assert_failure
}
