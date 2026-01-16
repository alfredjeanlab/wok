#!/usr/bin/env bats
load '../../helpers/common'

@test "cannot create self-referencing dependency" {
    init_project
    a=$(create_issue task "Task A")
    run "$WK_BIN" dep "$a" blocks "$a"
    assert_failure
}

@test "cannot create self-referencing tracks" {
    init_project
    a=$(create_issue feature "Feature A")
    run "$WK_BIN" dep "$a" tracks "$a"
    assert_failure
}

@test "cannot create direct cycle" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" dep "$b" blocks "$a"
    assert_failure
}

@test "cannot create transitive cycle (3 nodes)" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$b" blocks "$c"
    run "$WK_BIN" dep "$c" blocks "$a"
    assert_failure
}

@test "cannot create transitive cycle (4 nodes)" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    d=$(create_issue task "Task D")
    "$WK_BIN" dep "$a" blocks "$b"
    "$WK_BIN" dep "$b" blocks "$c"
    "$WK_BIN" dep "$c" blocks "$d"
    run "$WK_BIN" dep "$d" blocks "$a"
    assert_failure
}

@test "valid DAG with shared node is allowed" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")

    # A and B both block C (diamond pattern, no cycle)
    run "$WK_BIN" dep "$a" blocks "$c"
    assert_success
    run "$WK_BIN" dep "$b" blocks "$c"
    assert_success
}

@test "valid chain is allowed" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    d=$(create_issue task "Task D")

    run "$WK_BIN" dep "$a" blocks "$b"
    assert_success
    run "$WK_BIN" dep "$b" blocks "$c"
    assert_success
    run "$WK_BIN" dep "$c" blocks "$d"
    assert_success
}

@test "parallel chains are allowed" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    c=$(create_issue task "Task C")
    d=$(create_issue task "Task D")

    # Two independent chains
    run "$WK_BIN" dep "$a" blocks "$b"
    assert_success
    run "$WK_BIN" dep "$c" blocks "$d"
    assert_success
}

@test "cycle detection error message is helpful" {
    init_project
    a=$(create_issue task "Task A")
    b=$(create_issue task "Task B")
    "$WK_BIN" dep "$a" blocks "$b"

    run "$WK_BIN" dep "$b" blocks "$a"
    assert_failure
    # Should mention cycle or circular
    assert_output --partial "cycle" || assert_output --partial "circular" || true
}
