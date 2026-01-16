#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying examples shown in help text are accurate
# These tests run example commands that should match help output patterns
#
# NOTE: Most tests share a single init. Tests that need their own init
# (e.g., testing init command itself) are in init.bats.

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

# init examples from help (these need separate directories)

@test "init example: wk init works" {
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    run "$WK_BIN" init
    assert_success
    rm -rf "$tmpdir"
}

@test "init example: wk init --prefix prj works" {
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    run "$WK_BIN" init --prefix prj
    assert_success
    rm -rf "$tmpdir"
}

# new examples from help

@test "new example: wk new 'Fix login bug' works" {
    run "$WK_BIN" new "Fix login bug"
    assert_success
    assert_output --partial "[task]"
}

@test "new example: wk new task 'Title' works" {
    run "$WK_BIN" new task "My task title"
    assert_success
    assert_output --partial "[task]"
}

@test "new example: wk new bug 'Title' works" {
    run "$WK_BIN" new bug "Memory leak"
    assert_success
    assert_output --partial "[bug]"
}

@test "new example: wk new feature 'Title' works" {
    run "$WK_BIN" new feature "User authentication"
    assert_success
    assert_output --partial "[feature]"
}

@test "new example: wk new with --label and --note works" {
    run "$WK_BIN" new task "My task" --label auth --note "Initial context"
    assert_success
}

# lifecycle examples from help

@test "start example: wk start <id> works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" start "$id"
    assert_success
}

@test "done example: wk done <id> works" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" done "$id"
    assert_success
}

@test "done example: wk done <id> --reason works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" done "$id" --reason "already fixed"
    assert_success
}

@test "close example: wk close <id> --reason works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" close "$id" --reason "duplicate of another issue"
    assert_success
}

@test "reopen example: wk reopen <id> --reason works" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "regression found"
    assert_success
}

# edit examples from help

@test "edit example: wk edit <id> title 'new title' works" {
    id=$(create_issue task "Original")
    run "$WK_BIN" edit "$id" title "new title"
    assert_success
}

@test "edit example: wk edit <id> type feature works" {
    id=$(create_issue task "Original")
    run "$WK_BIN" edit "$id" type feature
    assert_success
}

# list examples from help

@test "list example: wk list works" {
    create_issue task "Test task"
    run "$WK_BIN" list
    assert_success
}

@test "list example: wk list --status todo works" {
    create_issue task "Test task"
    run "$WK_BIN" list --status todo
    assert_success
}

@test "list example: wk list --type task works" {
    create_issue task "Test task"
    run "$WK_BIN" list --type task --all
    assert_success
}

@test "list example: wk list --label mylabel works" {
    create_issue task "Test task" --label "mylabel"
    run "$WK_BIN" list --label mylabel
    assert_success
}

@test "list example: wk list --all works" {
    create_issue task "Test task"
    run "$WK_BIN" list --all
    assert_success
}

@test "list example: wk list --blocked works" {
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" list --blocked
    assert_success
}

# show example from help

@test "show example: wk show <id> works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" show "$id"
    assert_success
}

# tree example from help

@test "tree example: wk tree <id> works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" tree "$id"
    assert_success
}

# dep examples from help

@test "dep example: wk dep <id> blocks <id> works" {
    id1=$(create_issue task "Task A")
    id2=$(create_issue task "Task B")
    run "$WK_BIN" dep "$id1" blocks "$id2"
    assert_success
}

@test "dep example: wk dep <id> blocks multiple ids works" {
    id1=$(create_issue task "Task A")
    id2=$(create_issue task "Task B")
    id3=$(create_issue task "Task C")
    run "$WK_BIN" dep "$id1" blocks "$id2" "$id3"
    assert_success
}

@test "dep example: wk dep <feature> tracks multiple tasks works" {
    feature=$(create_issue feature "My feature")
    t1=$(create_issue task "Task 1")
    t2=$(create_issue task "Task 2")
    run "$WK_BIN" dep "$feature" tracks "$t1" "$t2"
    assert_success
}

# undep example from help

@test "undep example: wk undep <id> blocks <id> works" {
    id1=$(create_issue task "Task A")
    id2=$(create_issue task "Task B")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" undep "$id1" blocks "$id2"
    assert_success
}

# label examples from help

@test "label example: wk label <id> project:auth works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" label "$id" "project:auth"
    assert_success
}

@test "label example: wk label <id> urgent works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" label "$id" urgent
    assert_success
}

# unlabel example from help

@test "unlabel example: wk unlabel <id> <label> works" {
    id=$(create_issue task "Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" unlabel "$id" "mylabel"
    assert_success
}

# note example from help

@test "note example: wk note <id> 'note content' works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" note "$id" "This is a note about the issue"
    assert_success
}

# log examples from help

@test "log example: wk log works" {
    create_issue task "Test task"
    run "$WK_BIN" log
    assert_success
}

@test "log example: wk log <id> works" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" log "$id"
    assert_success
}

@test "log example: wk log --limit N works" {
    create_issue task "Test task"
    run "$WK_BIN" log --limit 5
    assert_success
}

# export example from help

@test "export example: wk export <filepath> works" {
    create_issue task "Test task"
    run "$WK_BIN" export "issues.jsonl"
    assert_success
    [ -f "issues.jsonl" ]
}

# help examples from help

@test "help example: wk help works" {
    run "$WK_BIN" help
    assert_success
}

@test "help example: wk help <command> works" {
    run "$WK_BIN" help new
    assert_success
}

@test "help example: wk --help works" {
    run "$WK_BIN" --help
    assert_success
}

@test "help example: wk -h works" {
    run "$WK_BIN" -h
    assert_success
}
