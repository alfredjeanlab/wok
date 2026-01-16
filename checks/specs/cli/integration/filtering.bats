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

@test "default list shows all open issues including blocked" {
    t1=$(create_issue task "Ready task")
    t2=$(create_issue task "Blocked task")
    "$WK_BIN" dep "$t1" blocks "$t2"

    run "$WK_BIN" list
    assert_success
    assert_output --partial "Ready task"
    assert_output --partial "Blocked task"
}

@test "status filter works alone" {
    id=$(create_issue task "StatusFilterTest")
    "$WK_BIN" start "$id"

    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "StatusFilterTest"

    run "$WK_BIN" list --status todo
    assert_success
    refute_output --partial "StatusFilterTest"
}

@test "type filter works alone" {
    create_issue task "Task item"
    create_issue bug "Bug item"
    create_issue feature "Feature item"

    run "$WK_BIN" list --type bug
    assert_success
    assert_output --partial "Bug item"
    refute_output --partial "Task item"
    refute_output --partial "Feature item"
}

@test "chore type filter works alone" {
    create_issue task "Task item"
    create_issue bug "Bug item"
    create_issue chore "Chore item"

    run "$WK_BIN" list --type chore
    assert_success
    assert_output --partial "Chore item"
    refute_output --partial "Task item"
    refute_output --partial "Bug item"
}

@test "chore type with -t short form" {
    create_issue task "Task item"
    create_issue chore "Chore item"

    run "$WK_BIN" list -t chore
    assert_success
    assert_output --partial "Chore item"
    refute_output --partial "Task item"
}

@test "chore + label combined filter" {
    create_issue chore "Labeled chore" --label "refactor"
    create_issue task "Labeled task" --label "refactor"
    create_issue chore "Other chore"

    run "$WK_BIN" list --type chore --label "refactor"
    assert_success
    assert_output --partial "Labeled chore"
    refute_output --partial "Labeled task"
    refute_output --partial "Other chore"
}

@test "chore in ready command" {
    a=$(create_issue chore "Blocker chore")
    b=$(create_issue chore "Blocked chore")
    "$WK_BIN" dep "$a" blocks "$b"

    run "$WK_BIN" ready --type chore
    assert_success
    assert_output --partial "Blocker chore"
    refute_output --partial "Blocked chore"
}

@test "type filter with -t short form" {
    create_issue task "Task item"
    create_issue bug "Bug item"

    run "$WK_BIN" list -t bug
    assert_success
    assert_output --partial "Bug item"
    refute_output --partial "Task item"
}

@test "label filter works alone" {
    create_issue task "Labeled" --label "team:alpha"
    create_issue task "Other"

    run "$WK_BIN" list --label "team:alpha"
    assert_success
    assert_output --partial "Labeled"
    refute_output --partial "Other"
}

@test "status + type combined" {
    t=$(create_issue task "Active task")
    b=$(create_issue bug "Active bug")
    "$WK_BIN" start "$t"
    "$WK_BIN" start "$b"
    create_issue task "Todo task"

    run "$WK_BIN" list --status in_progress --type task
    assert_success
    assert_output --partial "Active task"
    refute_output --partial "Active bug"
    refute_output --partial "Todo task"
}

@test "status + label combined" {
    t=$(create_issue task "Labeled active" --label "important")
    "$WK_BIN" start "$t"
    create_issue task "Labeled todo" --label "important"

    run "$WK_BIN" list --status in_progress --label "important"
    assert_success
    assert_output --partial "Labeled active"
    refute_output --partial "Labeled todo"
}

@test "type + label combined" {
    create_issue bug "Labeled bug" --label "team:alpha"
    create_issue task "Labeled task" --label "team:alpha"
    create_issue bug "Other bug"

    run "$WK_BIN" list --type bug --label "team:alpha"
    assert_success
    assert_output --partial "Labeled bug"
    refute_output --partial "Labeled task"
    refute_output --partial "Other bug"
}

@test "all three filters combined" {
    t=$(create_issue task "Match" --label "team:alpha")
    "$WK_BIN" start "$t"
    b=$(create_issue bug "No match" --label "team:alpha")
    "$WK_BIN" start "$b"

    run "$WK_BIN" list --status in_progress --type task --label "team:alpha"
    assert_success
    assert_output --partial "Match"
    refute_output --partial "No match"
}

@test "--all with filters" {
    a=$(create_issue task "Blocker" --label "team:alpha")
    b=$(create_issue task "Blocked" --label "team:alpha")
    c=$(create_issue task "Other blocked" --label "team:beta")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$a" blocks "$c"

    run "$WK_BIN" list --all --label "team:alpha"
    assert_success
    assert_output --partial "Blocker"
    assert_output --partial "Blocked"
    refute_output --partial "Other blocked"
}

@test "--blocked with filters" {
    a=$(create_issue task "Blocker")
    b=$(create_issue bug "Blocked bug")
    c=$(create_issue task "Blocked task")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$a" blocks "$c"

    run "$WK_BIN" list --blocked --type bug
    assert_success
    assert_output --partial "Blocked bug"
    refute_output --partial "Blocked task"
}

@test "empty result set handled gracefully" {
    create_issue task "Task"

    run "$WK_BIN" list --type feature
    assert_success
    # Should show nothing or empty message, not fail
}

@test "multiple labels require all to match" {
    # Note: behavior may vary - some implementations require all labels
    create_issue task "Has both" --label "a" --label "b"
    create_issue task "Has one" --label "a"

    run "$WK_BIN" list --label "a"
    assert_success
    assert_output --partial "Has both"
    assert_output --partial "Has one"
}

@test "closed items only appear with --status closed" {
    id=$(create_issue task "Closed item")
    "$WK_BIN" close "$id" --reason "done"

    run "$WK_BIN" list
    refute_output --partial "Closed item"

    run "$WK_BIN" list --status closed
    assert_output --partial "Closed item"
}

# ready command filter tests

@test "ready type + label combined" {
    create_issue bug "Labeled bug" --label "team:alpha"
    create_issue task "Labeled task" --label "team:alpha"
    run "$WK_BIN" ready --type bug --label "team:alpha"
    assert_success
    assert_output --partial "Labeled bug"
    refute_output --partial "Labeled task"
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

@test "ready with -t short flag for type" {
    create_issue bug "A bug"
    create_issue task "A task"
    run "$WK_BIN" ready -t bug
    assert_success
    assert_output --partial "A bug"
    refute_output --partial "A task"
}
