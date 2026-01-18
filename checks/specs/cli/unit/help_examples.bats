#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying examples shown in help text are accurate.
# These tests run example commands that should match help output patterns.

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

@test "init examples work" {
    # wk init
    local tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    run "$WK_BIN" init
    assert_success
    rm -rf "$tmpdir"

    # wk init --prefix prj
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    run "$WK_BIN" init --prefix prj
    assert_success
    rm -rf "$tmpdir"

    cd "$BATS_FILE_TMPDIR" || exit 1
}

@test "new examples work" {
    run "$WK_BIN" new "Fix login bug"
    assert_success
    assert_output --partial "[task]"

    run "$WK_BIN" new task "My task title"
    assert_success
    assert_output --partial "[task]"

    run "$WK_BIN" new bug "Memory leak"
    assert_success
    assert_output --partial "[bug]"

    run "$WK_BIN" new feature "User authentication"
    assert_success
    assert_output --partial "[feature]"

    run "$WK_BIN" new task "My task" --label auth --note "Initial context"
    assert_success
}

@test "lifecycle examples work" {
    # start
    id=$(create_issue task "Start test")
    run "$WK_BIN" start "$id"
    assert_success

    # done (after start)
    run "$WK_BIN" done "$id"
    assert_success

    # done with --reason (skip start)
    id=$(create_issue task "Done reason test")
    run "$WK_BIN" done "$id" --reason "already fixed"
    assert_success

    # close with --reason
    id=$(create_issue task "Close test")
    run "$WK_BIN" close "$id" --reason "duplicate of another issue"
    assert_success

    # reopen with --reason
    id=$(create_issue task "Reopen test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "regression found"
    assert_success
}

@test "edit examples work" {
    id=$(create_issue task "Original")
    run "$WK_BIN" edit "$id" title "new title"
    assert_success

    run "$WK_BIN" edit "$id" type feature
    assert_success
}

@test "list examples work" {
    create_issue task "List test"
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"
    create_issue task "Labeled" --label "mylabel"

    run "$WK_BIN" list
    assert_success

    run "$WK_BIN" list --status todo
    assert_success

    run "$WK_BIN" list --type task --all
    assert_success

    run "$WK_BIN" list --label mylabel
    assert_success

    run "$WK_BIN" list --all
    assert_success

    run "$WK_BIN" list --blocked
    assert_success
}

@test "show and tree examples work" {
    id=$(create_issue task "Show test")
    run "$WK_BIN" show "$id"
    assert_success

    run "$WK_BIN" tree "$id"
    assert_success
}

@test "dep and undep examples work" {
    id1=$(create_issue task "Task A")
    id2=$(create_issue task "Task B")
    id3=$(create_issue task "Task C")

    # dep blocks
    run "$WK_BIN" dep "$id1" blocks "$id2"
    assert_success

    # dep blocks multiple
    run "$WK_BIN" dep "$id1" blocks "$id2" "$id3"
    assert_success

    # dep tracks
    feature=$(create_issue feature "My feature")
    t1=$(create_issue task "Task 1")
    t2=$(create_issue task "Task 2")
    run "$WK_BIN" dep "$feature" tracks "$t1" "$t2"
    assert_success

    # undep
    run "$WK_BIN" undep "$id1" blocks "$id2"
    assert_success
}

@test "label and unlabel examples work" {
    id=$(create_issue task "Label test")

    run "$WK_BIN" label "$id" "project:auth"
    assert_success

    run "$WK_BIN" label "$id" urgent
    assert_success

    run "$WK_BIN" unlabel "$id" urgent
    assert_success
}

@test "note example works" {
    id=$(create_issue task "Note test")
    run "$WK_BIN" note "$id" "This is a note about the issue"
    assert_success
}

@test "log examples work" {
    create_issue task "Log test"

    run "$WK_BIN" log
    assert_success

    id=$(create_issue task "Log id test")
    run "$WK_BIN" log "$id"
    assert_success

    run "$WK_BIN" log --limit 5
    assert_success
}

@test "export example works" {
    create_issue task "Export test"
    run "$WK_BIN" export "$BATS_FILE_TMPDIR/issues.jsonl"
    assert_success
    [ -f "$BATS_FILE_TMPDIR/issues.jsonl" ]
}

@test "help examples work" {
    run "$WK_BIN" help
    assert_success

    run "$WK_BIN" help new
    assert_success

    run "$WK_BIN" --help
    assert_success

    run "$WK_BIN" -h
    assert_success
}
