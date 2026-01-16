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

@test "blocks relationship affects list --blocked view" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"

    # Default list shows both blocked and unblocked issues
    run "$WK_BIN" list
    assert_success
    assert_output --partial "$a"
    assert_output --partial "$b"

    # --blocked shows only blocked issues
    run "$WK_BIN" list --blocked
    assert_success
    refute_output --partial "$a"
    assert_output --partial "$b"
}

@test "list --blocked shows blocked issues" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"

    run "$WK_BIN" list --blocked
    assert_success
    assert_output --partial "$b"
}

@test "list --all shows all issues" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"

    run "$WK_BIN" list --all
    assert_success
    assert_output --partial "$a"
    assert_output --partial "$b"
}

@test "completing blocker unblocks dependent" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" start "$a"
    "$WK_BIN" done "$a"

    run "$WK_BIN" list
    assert_success
    assert_output --partial "$b"
}

@test "transitive blocking works" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$b" blocks "$c"

    # list shows all open issues
    run "$WK_BIN" list
    assert_success
    assert_output --partial "$a"
    assert_output --partial "$b"
    assert_output --partial "$c"

    # ready shows only unblocked (A), not B or C
    run "$WK_BIN" ready
    assert_success
    assert_output --partial "$a"
    refute_output --partial "$b"
    refute_output --partial "$c"
}

@test "transitive blocking unblocks in chain" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$b" blocks "$c"

    # Complete A - B becomes ready, C still blocked
    "$WK_BIN" start "$a"
    "$WK_BIN" done "$a"

    run "$WK_BIN" ready
    assert_output --partial "$b"
    refute_output --partial "$c"

    # Complete B - C becomes ready
    "$WK_BIN" start "$b"
    "$WK_BIN" done "$b"

    run "$WK_BIN" ready
    assert_output --partial "$c"
}

@test "tracks creates tracks relationship" {
    feature=$(create_issue feature "My Feature")
    task=$(create_issue task "My Task")
    "$WK_BIN" dep "$feature" tracks "$task"

    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"
    assert_output --partial "$task"

    run "$WK_BIN" show "$task"
    assert_success
    assert_output --partial "Tracked by"
    assert_output --partial "$feature"
}

@test "tracks does not block" {
    feature=$(create_issue feature "My Feature")
    task=$(create_issue task "My Task")
    "$WK_BIN" dep "$feature" tracks "$task"

    # Task should be ready (tracks doesn't block)
    run "$WK_BIN" list
    assert_success
    assert_output --partial "$task"
}

@test "multiple blockers all must complete" {
    a=$(create_issue task "Blocker A")
    b=$(create_issue task "Blocker B")
    c=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$c"
    "$WK_BIN" dep "$b" blocks "$c"

    # C is blocked
    run "$WK_BIN" list --blocked
    assert_output --partial "$c"

    # Complete A - C still blocked by B
    "$WK_BIN" start "$a"
    "$WK_BIN" done "$a"

    run "$WK_BIN" list --blocked
    assert_output --partial "$c"

    # Complete B - C now ready
    "$WK_BIN" start "$b"
    "$WK_BIN" done "$b"

    run "$WK_BIN" list
    assert_output --partial "$c"
}

@test "blocking does not prevent status transitions" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"

    # Can still start blocked issue (informational only)
    run "$WK_BIN" start "$b"
    assert_success

    run "$WK_BIN" done "$b"
    assert_success
}

@test "dependencies remain after completion" {
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" start "$a"
    "$WK_BIN" done "$a"

    # Dependency still exists in show output
    run "$WK_BIN" show "$a"
    assert_output --partial "Blocks"
    assert_output --partial "$b"
}

@test "show displays all blockers" {
    a=$(create_issue task "Blocker A")
    b=$(create_issue task "Blocker B")
    c=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$c"
    "$WK_BIN" dep "$b" blocks "$c"

    run "$WK_BIN" show "$c"
    assert_success
    assert_output --partial "Blocked by"
    assert_output --partial "$a"
    assert_output --partial "$b"
}
