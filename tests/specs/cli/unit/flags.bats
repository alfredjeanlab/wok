#!/usr/bin/env bats
load '../../helpers/common'


# Tests verifying all documented --flags actually work
# Based on REQUIREMENTS.md command reference
@test "init flags: --prefix and --path" {
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"

    # --prefix creates project with custom prefix
    run "$WK_BIN" init --prefix myproj
    assert_success
    id=$(create_issue task "Test task")
    [[ "$id" == myproj-* ]]
    rm -rf .wok

    # --path creates project in specified path
    mkdir -p subdir
    run "$WK_BIN" init --path subdir --prefix test
    assert_success
    [ -d "subdir/.wok" ]

    rm -rf "$tmpdir"
}

@test "new flags: --label and --note" {
    # --label adds label to new issue
    id=$(create_issue task "Test task" --label "mylabel")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "mylabel"

    # --label multiple times adds all labels
    id=$(create_issue task "Test task" --label "label1" --label "label2")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"

    # --note adds initial note
    id=$(create_issue task "Test task" --note "Initial note content")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial note content"

    # --label and --note work together
    id=$(create_issue task "Test task" --label "mylabel" --note "My note")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "mylabel"
    assert_output --partial "My note"
}

@test "done/close flags: --reason records in log" {
    # done --reason allows todo to done transition
    id=$(create_issue task "Test done")
    run "$WK_BIN" done "$id" --reason "Already fixed upstream"
    assert_success
    run "$WK_BIN" log "$id"
    assert_output --partial "Already fixed"

    # close --reason closes issue with reason
    id=$(create_issue task "Test close")
    run "$WK_BIN" close "$id" --reason "Duplicate issue"
    assert_success
    run "$WK_BIN" log "$id"
    assert_output --partial "Duplicate"
}

@test "reopen flags: --reason records in log" {
    # reopen --reason reopens closed issue with reason
    id=$(create_issue task "Test reopen")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id" --reason "Found regression"
    assert_success
    run "$WK_BIN" log "$id"
    assert_output --partial "regression"
}

@test "edit positional args: title and type" {
    # edit title changes issue title
    id=$(create_issue task "Original title")
    run "$WK_BIN" edit "$id" title "New title"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "New title"

    # edit type changes issue type
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" type bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[bug]"

    # edit title and type work sequentially
    id=$(create_issue task "Original")
    "$WK_BIN" edit "$id" title "Changed"
    "$WK_BIN" edit "$id" type feature
    run "$WK_BIN" show "$id"
    assert_output --partial "Changed"
    assert_output --partial "[feature]"
}

@test "list --status filters by status" {
    id1=$(create_issue task "StatusFlag Todo")
    id2=$(create_issue task "StatusFlag Started")
    id3=$(create_issue task "StatusFlag Done")
    id4=$(create_issue task "StatusFlag Closed")
    "$WK_BIN" start "$id2"
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    "$WK_BIN" close "$id4" --reason "Won't fix"

    run "$WK_BIN" list --status todo
    assert_success
    assert_output --partial "StatusFlag Todo"
    refute_output --partial "StatusFlag Started"

    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "StatusFlag Started"
    refute_output --partial "StatusFlag Todo"

    run "$WK_BIN" list --status done
    assert_success
    assert_output --partial "StatusFlag Done"
    refute_output --partial "StatusFlag Todo"

    run "$WK_BIN" list --status closed
    assert_success
    assert_output --partial "StatusFlag Closed"
    refute_output --partial "StatusFlag Todo"
}

@test "list --type filters by type (including short flag)" {
    create_issue task "TypeFlag My task"
    create_issue feature "TypeFlag My feature"
    create_issue bug "TypeFlag My bug"

    run "$WK_BIN" list --type feature --all
    assert_success
    assert_output --partial "TypeFlag My feature"
    refute_output --partial "TypeFlag My task"

    run "$WK_BIN" list --type task --all
    assert_success
    assert_output --partial "TypeFlag My task"
    refute_output --partial "TypeFlag My bug"

    run "$WK_BIN" list --type bug --all
    assert_success
    assert_output --partial "TypeFlag My bug"
    refute_output --partial "TypeFlag My task"

    # -t short form works
    run "$WK_BIN" list -t bug --all
    assert_success
    assert_output --partial "TypeFlag My bug"
    refute_output --partial "TypeFlag My task"

    # comma-separated types
    run "$WK_BIN" list -t bug,task --all
    assert_success
    assert_output --partial "TypeFlag My task"
    assert_output --partial "TypeFlag My bug"
    refute_output --partial "TypeFlag My feature"
}

@test "list --label, --all, --blocked filters" {
    create_issue task "LabelFlag Labeled task" --label "findme"
    create_issue task "LabelFlag Unlabeled task"
    id1=$(create_issue task "BlockFlag Blocker")
    id2=$(create_issue task "BlockFlag Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"

    # --label filters by label
    run "$WK_BIN" list --label "findme" --all
    assert_success
    assert_output --partial "LabelFlag Labeled task"
    refute_output --partial "LabelFlag Unlabeled task"

    # --all includes blocked issues
    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "BlockFlag Blocker"
    assert_output --partial "BlockFlag Blocked"

    # --blocked shows only blocked issues
    run "$WK_BIN" list --blocked
    assert_success
    assert_output --partial "BlockFlag Blocked"
    refute_output --partial "BlockFlag Blocker"

    # -b short flag is not supported
    run "$WK_BIN" list -b
    assert_failure

    # list without flags shows blocked issues
    run "$WK_BIN" list
    assert_success
    assert_output --partial "BlockFlag Blocker"
    assert_output --partial "BlockFlag Blocked"
}

@test "list combined filters: --status --type --label" {
    create_issue task "CombFlag Todo task"
    id=$(create_issue bug "CombFlag Todo bug")
    "$WK_BIN" start "$id"
    id2=$(create_issue task "CombFlag Tagged todo" --label "mylabel")
    id3=$(create_issue task "CombFlag Tagged started" --label "mylabel")
    "$WK_BIN" start "$id3"

    # --status --type combined filtering
    run "$WK_BIN" list --status in_progress --type bug --all
    assert_success
    assert_output --partial "CombFlag Todo bug"
    refute_output --partial "CombFlag Todo task"

    # --label --status combined filtering
    run "$WK_BIN" list --label "mylabel" --status in_progress --all
    assert_success
    assert_output --partial "CombFlag Tagged started"
    refute_output --partial "CombFlag Tagged todo"
}

@test "log --limit limits output" {
    id=$(create_issue task "LogLimit task")
    "$WK_BIN" note "$id" "Note 1"
    "$WK_BIN" note "$id" "Note 2"
    "$WK_BIN" note "$id" "Note 3"
    "$WK_BIN" start "$id"

    # --limit limits output
    run "$WK_BIN" log "$id" --limit 2
    assert_success

    # --limit 1 shows only most recent event
    run "$WK_BIN" log "$id" --limit 1
    assert_success
    assert_output --partial "started"
}

@test "ready filters: --type, --label, and combined" {
    create_issue task "ReadyFlag Ready task"
    create_issue bug "ReadyFlag Ready bug"
    create_issue task "ReadyFlag Labeled task" --label "urgent"
    create_issue task "ReadyFlag Unlabeled task"
    create_issue bug "ReadyFlag Labeled bug" --label "team:alpha"
    create_issue task "ReadyFlag Other labeled task" --label "team:alpha"
    a=$(create_issue bug "ReadyFlag Blocker bug" --label "team:alpha")
    b=$(create_issue bug "ReadyFlag Blocked bug" --label "team:alpha")
    "$WK_BIN" dep "$a" blocks "$b"

    # --type filters ready items
    run "$WK_BIN" ready --type bug
    assert_success
    assert_output --partial "ReadyFlag Ready bug"
    refute_output --partial "ReadyFlag Ready task"

    # -t short flag works
    run "$WK_BIN" ready -t bug
    assert_success
    assert_output --partial "ReadyFlag Ready bug"
    refute_output --partial "ReadyFlag Ready task"

    # --label filters ready items
    run "$WK_BIN" ready --label urgent
    assert_success
    assert_output --partial "ReadyFlag Labeled task"
    refute_output --partial "ReadyFlag Unlabeled task"

    # --type --label combined filtering
    run "$WK_BIN" ready --type bug --label "team:alpha"
    assert_success
    assert_output --partial "ReadyFlag Labeled bug"
    assert_output --partial "ReadyFlag Blocker bug"
    refute_output --partial "ReadyFlag Other labeled task"
    refute_output --partial "ReadyFlag Blocked bug"
}

@test "ready rejects --all and --blocked flags" {
    run "$WK_BIN" ready --all
    assert_failure

    run "$WK_BIN" ready --blocked
    assert_failure
}

@test "-C flag: runs command in specified directory" {
    mkdir -p other-project
    "$WK_BIN" init --path other-project --prefix othr

    run "$WK_BIN" -C other-project new task "Remote task"
    assert_success

    run "$WK_BIN" -C other-project list
    assert_success
    assert_output --partial "Remote task"
}

@test "-C flag: error on nonexistent directory" {
    run "$WK_BIN" -C /nonexistent/path list
    assert_failure
    assert_output --partial "cannot change to directory"
}

@test "-C flag: works with init" {
    mkdir -p newproj
    run "$WK_BIN" -C newproj init --prefix np
    assert_success
    [ -d "newproj/.wok" ]
}
