#!/usr/bin/env bats
load '../../helpers/common'

@test "tree shows issue and children" {
    feature=$(create_issue feature "Parent feature")
    task=$(create_issue task "Child task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" tree "$feature"
    assert_success
    assert_output --partial "Parent feature"
    assert_output --partial "Child task"
}

@test "tree shows status of children" {
    feature=$(create_issue feature "Parent")
    task=$(create_issue task "Child")
    "$WK_BIN" dep "$feature" tracks "$task"
    "$WK_BIN" start "$task"
    run "$WK_BIN" tree "$feature"
    assert_success
    assert_output --partial "in_progress"
}

@test "tree shows nested hierarchy" {
    feature=$(create_issue feature "Feature")
    sub=$(create_issue task "Subtask")
    subsub=$(create_issue task "Sub-subtask")
    "$WK_BIN" dep "$feature" tracks "$sub"
    "$WK_BIN" dep "$sub" tracks "$subsub"
    run "$WK_BIN" tree "$feature"
    assert_success
    assert_output --partial "Feature"
    assert_output --partial "Subtask"
    assert_output --partial "Sub-subtask"
}

@test "tree shows blocking relationships" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" tree "$b"
    assert_success
    assert_output --partial "blocked"
}

@test "tree with no children shows just the issue" {
    id=$(create_issue task "Standalone task")
    run "$WK_BIN" tree "$id"
    assert_success
    assert_output --partial "Standalone task"
}

@test "tree nonexistent issue fails" {
    run "$WK_BIN" tree "test-nonexistent"
    assert_failure
}

@test "tree requires issue ID" {
    run "$WK_BIN" tree
    assert_failure
}

@test "tree shows multiple children" {
    feature=$(create_issue feature "Feature")
    t1=$(create_issue task "Task 1")
    t2=$(create_issue task "Task 2")
    t3=$(create_issue task "Task 3")
    "$WK_BIN" dep "$feature" tracks "$t1" "$t2" "$t3"
    run "$WK_BIN" tree "$feature"
    assert_success
    assert_output --partial "Task 1"
    assert_output --partial "Task 2"
    assert_output --partial "Task 3"
}

@test "tree accepts multiple issue IDs" {
    f1=$(create_issue feature "Feature 1")
    f2=$(create_issue feature "Feature 2")
    run "$WK_BIN" tree "$f1" "$f2"
    assert_success
    assert_output --partial "Feature 1"
    assert_output --partial "Feature 2"
}

@test "tree separates multiple issues with ---" {
    f1=$(create_issue feature "Feature 1")
    f2=$(create_issue feature "Feature 2")
    run "$WK_BIN" tree "$f1" "$f2"
    assert_success
    assert_output --partial "---"
}

@test "tree with multiple IDs fails fast on invalid ID" {
    f1=$(create_issue feature "Valid feature")
    run "$WK_BIN" tree "$f1" "nonexistent-id"
    assert_failure
}

@test "tree renders each issue with its own children" {
    f1=$(create_issue feature "Feature 1")
    f2=$(create_issue feature "Feature 2")
    t1=$(create_issue task "Task for F1")
    t2=$(create_issue task "Task for F2")
    "$WK_BIN" dep "$f1" tracks "$t1"
    "$WK_BIN" dep "$f2" tracks "$t2"
    run "$WK_BIN" tree "$f1" "$f2"
    assert_success
    assert_output --partial "Feature 1"
    assert_output --partial "Task for F1"
    assert_output --partial "Feature 2"
    assert_output --partial "Task for F2"
}
