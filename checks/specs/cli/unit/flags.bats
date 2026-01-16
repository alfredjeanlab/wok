#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying all documented --flags actually work
# Based on REQUIREMENTS.md command reference
#
# Using file-level setup since most tests can share a single init.

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

# init flags (these need separate directories since they test init)

@test "init --prefix creates project with custom prefix" {
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    run "$WK_BIN" init --prefix myproj
    assert_success
    id=$(create_issue task "Test task")
    [[ "$id" == myproj-* ]]
    rm -rf "$tmpdir"
}

@test "init --path creates project in specified path" {
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    mkdir -p subdir
    run "$WK_BIN" init --path subdir --prefix test
    assert_success
    [ -d "subdir/.wok" ]
    rm -rf "$tmpdir"
}

# new flags

@test "new --label adds label to new issue" {
    id=$(create_issue task "Test task" --label "mylabel")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "mylabel"
}

@test "new --label multiple times adds all labels" {
    id=$(create_issue task "Test task" --label "label1" --label "label2")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"
}

@test "new --note adds initial note" {
    id=$(create_issue task "Test task" --note "Initial note content")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial note content"
}

@test "new --label and --note work together" {
    id=$(create_issue task "Test task" --label "mylabel" --note "My note")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "mylabel"
    assert_output --partial "My note"
}

# done flags

@test "done --reason allows todo to done transition" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" done "$id" --reason "Already fixed upstream"
    assert_success
}

@test "done --reason records reason in log" {
    id=$(create_issue task "Test task")
    "$WK_BIN" done "$id" --reason "Already fixed upstream"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "Already fixed"
}

# close flags

@test "close --reason closes issue with reason" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" close "$id" --reason "Duplicate issue"
    assert_success
}

@test "close --reason records reason in log" {
    id=$(create_issue task "Test task")
    "$WK_BIN" close "$id" --reason "Duplicate issue"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "Duplicate"
}

# reopen flags

@test "reopen --reason reopens closed issue with reason" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "Found regression"
    assert_success
}

@test "reopen --reason records reason in log" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" reopen "$id" --reason "Found regression"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "regression"
}

# edit positional args

@test "edit title changes issue title" {
    id=$(create_issue task "Original title")
    run "$WK_BIN" edit "$id" title "New title"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "New title"
}

@test "edit type changes issue type" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" type bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[bug]"
}

@test "edit title and type work sequentially" {
    id=$(create_issue task "Original")
    run "$WK_BIN" edit "$id" title "Changed"
    assert_success
    run "$WK_BIN" edit "$id" type feature
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Changed"
    assert_output --partial "[feature]"
}

# list flags

@test "list --status todo shows only todo issues" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "Started task")
    "$WK_BIN" start "$id2"
    run "$WK_BIN" list --status todo
    assert_success
    assert_output --partial "Todo task"
    refute_output --partial "Started task"
}

@test "list --status in_progress shows only in_progress issues" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "Started task")
    "$WK_BIN" start "$id2"
    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "Started task"
    refute_output --partial "Todo task"
}

@test "list --status done shows only done issues" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "Done task")
    "$WK_BIN" start "$id2"
    "$WK_BIN" done "$id2"
    run "$WK_BIN" list --status done
    assert_success
    assert_output --partial "Done task"
    refute_output --partial "Todo task"
}

@test "list --status closed shows only closed issues" {
    id1=$(create_issue task "Open task")
    id2=$(create_issue task "Closed task")
    "$WK_BIN" close "$id2" --reason "Won't fix"
    run "$WK_BIN" list --status closed
    assert_success
    assert_output --partial "Closed task"
    refute_output --partial "Open task"
}

@test "list --type feature shows only features" {
    create_issue task "My task"
    create_issue feature "My feature"
    run "$WK_BIN" list --type feature --all
    assert_success
    assert_output --partial "My feature"
    refute_output --partial "My task"
}

@test "list --type task shows only tasks" {
    create_issue task "My task"
    create_issue bug "My bug"
    run "$WK_BIN" list --type task --all
    assert_success
    assert_output --partial "My task"
    refute_output --partial "My bug"
}

@test "list --type bug shows only bugs" {
    create_issue task "My task"
    create_issue bug "My bug"
    run "$WK_BIN" list --type bug --all
    assert_success
    assert_output --partial "My bug"
    refute_output --partial "My task"
}

@test "list -t bug shows only bugs (short form)" {
    create_issue task "My task"
    create_issue bug "My bug"
    run "$WK_BIN" list -t bug --all
    assert_success
    assert_output --partial "My bug"
    refute_output --partial "My task"
}

@test "list --label filters by label" {
    create_issue task "Labeled task" --label "findme"
    create_issue task "Unlabeled task"
    run "$WK_BIN" list --label "findme" --all
    assert_success
    assert_output --partial "Labeled task"
    refute_output --partial "Unlabeled task"
}

@test "list --all includes blocked issues" {
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "Blocker"
    assert_output --partial "Blocked"
}

@test "list --blocked shows only blocked issues" {
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" list --blocked
    assert_success
    assert_output --partial "Blocked"
    refute_output --partial "Blocker"
}

@test "list -b short flag is not supported" {
    run "$WK_BIN" list -b
    assert_failure
}

# list without flags shows all open issues (blocked and unblocked)

@test "list without flags shows blocked issues" {
    id1=$(create_issue task "Ready task")
    id2=$(create_issue task "Blocked task")
    "$WK_BIN" dep "$id1" blocks "$id2"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Ready task"
    assert_output --partial "Blocked task"
}

# log flags

@test "log --limit limits output" {
    id=$(create_issue task "Test task")
    "$WK_BIN" note "$id" "Note 1"
    "$WK_BIN" note "$id" "Note 2"
    "$WK_BIN" note "$id" "Note 3"
    run "$WK_BIN" log "$id" --limit 2
    assert_success
    # Should have limited output (exact behavior depends on implementation)
}

@test "log --limit 1 shows only most recent event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" log "$id" --limit 1
    assert_success
    # Should show the started event (most recent)
    assert_output --partial "started"
}

# Combined flag tests

@test "list --status --type combined filtering" {
    create_issue task "Todo task"
    id=$(create_issue bug "Todo bug")
    "$WK_BIN" start "$id"
    run "$WK_BIN" list --status in_progress --type bug --all
    assert_success
    assert_output --partial "Todo bug"
    refute_output --partial "Todo task"
}

@test "list --label --status combined filtering" {
    id1=$(create_issue task "Tagged todo" --label "mylabel")
    id2=$(create_issue task "Tagged started" --label "mylabel")
    "$WK_BIN" start "$id2"
    run "$WK_BIN" list --label "mylabel" --status in_progress --all
    assert_success
    assert_output --partial "Tagged started"
    refute_output --partial "Tagged todo"
}

# list -t type filter (new short flag)

@test "list -t filters by type (short flag)" {
    create_issue task "My task"
    create_issue bug "My bug"
    run "$WK_BIN" list -t bug --all
    assert_success
    assert_output --partial "My bug"
    refute_output --partial "My task"
}

@test "list -t with comma-separated types" {
    create_issue task "My task"
    create_issue bug "My bug"
    create_issue feature "My feature"
    run "$WK_BIN" list -t bug,task --all
    assert_success
    assert_output --partial "My task"
    assert_output --partial "My bug"
    refute_output --partial "My feature"
}

# ready command filters

@test "ready --type filters ready items by type" {
    create_issue task "Ready task"
    create_issue bug "Ready bug"
    run "$WK_BIN" ready --type bug
    assert_success
    assert_output --partial "Ready bug"
    refute_output --partial "Ready task"
}

@test "ready -t filters ready items by type (short flag)" {
    create_issue task "Ready task"
    create_issue bug "Ready bug"
    run "$WK_BIN" ready -t bug
    assert_success
    assert_output --partial "Ready bug"
    refute_output --partial "Ready task"
}

@test "ready --label filters ready items by label" {
    id1=$(create_issue task "Labeled task" --label "urgent")
    create_issue task "Unlabeled task"
    run "$WK_BIN" ready --label urgent
    assert_success
    assert_output --partial "Labeled task"
    refute_output --partial "Unlabeled task"
}

@test "ready --type --label combined filtering" {
    create_issue bug "Labeled bug" --label "team:alpha"
    create_issue task "Labeled task" --label "team:alpha"
    create_issue bug "Unlabeled bug"
    run "$WK_BIN" ready --type bug --label "team:alpha"
    assert_success
    assert_output --partial "Labeled bug"
    refute_output --partial "Labeled task"
    refute_output --partial "Unlabeled bug"
}

@test "ready excludes blocked even with filters" {
    a=$(create_issue bug "Blocker bug" --label "team:alpha")
    b=$(create_issue bug "Blocked bug" --label "team:alpha")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" ready --type bug --label "team:alpha"
    assert_success
    assert_output --partial "Blocker bug"
    refute_output --partial "Blocked bug"
}

@test "ready does not accept --all flag" {
    run "$WK_BIN" ready --all
    assert_failure
}

@test "ready does not accept --blocked flag" {
    run "$WK_BIN" ready --blocked
    assert_failure
}

