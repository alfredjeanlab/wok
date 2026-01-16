#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying no command aliases exist - only canonical names work
#
# Using file-level setup since all tests can share a single init.

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
# Per REQUIREMENTS.md, only the documented canonical command names should work.
# Common aliases that should NOT work:
# - create (use: new)
# - ls (use: list)
# - rm/delete/remove (no delete command)
# - add (use: new)

# Test that common aliases DON'T work

@test "alias 'create' does not exist (use 'new')" {
    run "$WK_BIN" create "Test task"
    assert_failure
    # Should fail with unknown command
}

@test "alias 'ls' does not exist (use 'list')" {
    run "$WK_BIN" ls
    assert_failure
    # Should fail with unknown command
}

@test "alias 'add' does not exist (use 'new')" {
    run "$WK_BIN" add "Test task"
    assert_failure
}

@test "alias 'rm' does not exist" {
    id=$(create_issue task "Test")
    run "$WK_BIN" rm "$id"
    assert_failure
}

@test "alias 'del' does not exist" {
    id=$(create_issue task "Test")
    run "$WK_BIN" del "$id"
    assert_failure
}

@test "alias 'view' does not exist (use 'show')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" view "$id"
    assert_failure
}

@test "alias 'get' does not exist (use 'show')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" get "$id"
    assert_failure
}

@test "alias 'begin' does not exist (use 'start')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" begin "$id"
    assert_failure
}

@test "alias 'finish' does not exist (use 'done')" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    run "$WK_BIN" finish "$id"
    assert_failure
}

@test "alias 'complete' does not exist (use 'done')" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    run "$WK_BIN" complete "$id"
    assert_failure
}

@test "alias 'modify' does not exist (use 'edit')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" modify "$id" --title "New"
    assert_failure
}

@test "alias 'update' does not exist (use 'edit')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" update "$id" --title "New"
    assert_failure
}

@test "alias 'link' does not exist (use 'dep')" {
    id1=$(create_issue task "A")
    id2=$(create_issue task "B")
    run "$WK_BIN" link "$id1" blocks "$id2"
    assert_failure
}

@test "alias 'unlink' does not exist (use 'undep')" {
    id1=$(create_issue task "A")
    id2=$(create_issue task "B")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" unlink "$id1" blocks "$id2"
    assert_failure
}

@test "alias 'comment' does not exist (use 'note')" {
    id=$(create_issue task "Test")
    run "$WK_BIN" comment "$id" "My comment"
    assert_failure
}

@test "alias 'history' does not exist (use 'log')" {
    run "$WK_BIN" history
    assert_failure
}

@test "alias 'events' does not exist (use 'log')" {
    run "$WK_BIN" events
    assert_failure
}

@test "alias 'backup' does not exist (use 'export')" {
    run "$WK_BIN" backup "backup.jsonl"
    assert_failure
}

@test "alias 'dump' does not exist (use 'export')" {
    run "$WK_BIN" dump "dump.jsonl"
    assert_failure
}

# Verify canonical commands DO work

@test "canonical 'new' works" {
    run "$WK_BIN" new "Test task"
    assert_success
}

@test "canonical 'list' works" {
    create_issue task "Test"
    run "$WK_BIN" list
    assert_success
}

@test "canonical 'show' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" show "$id"
    assert_success
}

@test "canonical 'start' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" start "$id"
    assert_success
}

@test "canonical 'reopen' from in_progress works" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    run "$WK_BIN" reopen "$id"
    assert_success
}

@test "canonical 'done' works" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    run "$WK_BIN" done "$id"
    assert_success
}

@test "canonical 'close' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" close "$id" --reason "closed"
    assert_success
}

@test "canonical 'edit' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" edit "$id" title "New title"
    assert_success
}

@test "canonical 'dep' works" {
    id1=$(create_issue task "A")
    id2=$(create_issue task "B")
    run "$WK_BIN" dep "$id1" blocks "$id2"
    assert_success
}

@test "canonical 'undep' works" {
    id1=$(create_issue task "A")
    id2=$(create_issue task "B")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" undep "$id1" blocks "$id2"
    assert_success
}

@test "canonical 'label' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" label "$id" "mylabel"
    assert_success
}

@test "canonical 'unlabel' works" {
    id=$(create_issue task "Test")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" unlabel "$id" "mylabel"
    assert_success
}

@test "canonical 'note' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" note "$id" "My note"
    assert_success
}

@test "canonical 'log' works" {
    create_issue task "Test"
    run "$WK_BIN" log
    assert_success
}

@test "canonical 'export' works" {
    create_issue task "Test"
    run "$WK_BIN" export "export.jsonl"
    assert_success
}

@test "canonical 'tree' works" {
    id=$(create_issue task "Test")
    run "$WK_BIN" tree "$id"
    assert_success
}

@test "canonical 'reopen' works" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "reopened"
    assert_success
}
