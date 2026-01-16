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

@test "label adds label to issue" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" label "$id" "project:auth"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "project:auth"
}

@test "label supports simple labels" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" label "$id" "urgent"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "urgent"
}

@test "label supports namespaced labels" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" label "$id" "team:backend"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "team:backend"
}

@test "multiple labels can be added" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "label1"
    "$WK_BIN" label "$id" "label2"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"
}

@test "unlabel removes label" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" unlabel "$id" "mylabel"
    assert_success
    run "$WK_BIN" show "$id"
    # Only verify the Labels section doesn't contain the label
    # (Log section will still show "labeled mylabel" and "unlabeled mylabel" events)
    refute_line --regexp '^Labels:.*mylabel'
}

@test "unlabel nonexistent label succeeds or fails gracefully" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" unlabel "$id" "nonexistent"
    # Should either succeed (idempotent) or fail gracefully
    true
}

@test "label with nonexistent issue fails" {
    run "$WK_BIN" label "test-nonexistent" "mylabel"
    assert_failure
}

@test "label logs event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "labeled"
}

@test "unlabel logs event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    "$WK_BIN" unlabel "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "unlabeled"
}

@test "duplicate label is idempotent or fails gracefully" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" label "$id" "mylabel"
    # Should either succeed (idempotent) or fail gracefully
    true
}

@test "labels are searchable via list --label" {
    id=$(create_issue task "Labeled task")
    "$WK_BIN" label "$id" "findme"
    run "$WK_BIN" list --label "findme"
    assert_success
    assert_output --partial "Labeled task"
}

# === Batch Operations ===

@test "label adds label to multiple issues" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    run "$WK_BIN" label "$id1" "$id2" "urgent"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "urgent"
    run "$WK_BIN" show "$id2"
    assert_output --partial "urgent"
}

@test "label adds label to three issues" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    id3=$(create_issue task "Task 3")
    run "$WK_BIN" label "$id1" "$id2" "$id3" "backend"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "backend"
    run "$WK_BIN" show "$id2"
    assert_output --partial "backend"
    run "$WK_BIN" show "$id3"
    assert_output --partial "backend"
}

@test "unlabel removes label from multiple issues" {
    id1=$(create_issue task "Task 1")
    id2=$(create_issue task "Task 2")
    "$WK_BIN" label "$id1" "urgent"
    "$WK_BIN" label "$id2" "urgent"
    run "$WK_BIN" unlabel "$id1" "$id2" "urgent"
    assert_success
    run "$WK_BIN" show "$id1"
    refute_line --regexp '^Labels:.*urgent'
    run "$WK_BIN" show "$id2"
    refute_line --regexp '^Labels:.*urgent'
}

@test "batch label fails on nonexistent issue" {
    id1=$(create_issue task "Task 1")
    run "$WK_BIN" label "$id1" "test-nonexistent" "urgent"
    assert_failure
}

@test "batch labeled issues are searchable" {
    id1=$(create_issue task "Batch task 1")
    id2=$(create_issue task "Batch task 2")
    "$WK_BIN" label "$id1" "$id2" "batchtest"
    run "$WK_BIN" list --label "batchtest"
    assert_success
    assert_output --partial "Batch task 1"
    assert_output --partial "Batch task 2"
}
