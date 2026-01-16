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

@test "new creates task by default" {
    run "$WK_BIN" new "My task"
    assert_success
    assert_output --partial "[task]"
}

@test "new task creates task type" {
    run "$WK_BIN" new task "My task"
    assert_success
    assert_output --partial "[task]"
}

@test "new feature creates feature type" {
    run "$WK_BIN" new feature "My feature"
    assert_success
    assert_output --partial "[feature]"
}

@test "new bug creates bug type" {
    run "$WK_BIN" new bug "My bug"
    assert_success
    assert_output --partial "[bug]"
}

@test "new chore creates chore type" {
    run "$WK_BIN" new chore "My chore"
    assert_success
    assert_output --partial "[chore]"
}

@test "new issue starts in todo status" {
    run "$WK_BIN" new "My task"
    assert_success
    assert_output --partial "(todo)"
}

@test "new with --label adds label" {
    id=$(create_issue task "Labeled task" --label "project:auth")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"
}

@test "new with multiple --label adds all labels" {
    id=$(create_issue task "Multi-labeled task" --label "project:auth" --label "priority:high")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"
    assert_output --partial "priority:high"
}

@test "new with --note adds note" {
    id=$(create_issue task "Noted task" --note "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"
}

@test "new generates unique ID with prefix" {
    # Use a subdirectory to test with different prefix
    mkdir -p subproj && cd subproj
    "$WK_BIN" init --prefix myproj
    run "$WK_BIN" new "Test task"
    assert_success
    assert_output --partial "myproj-"
}

@test "new requires title" {
    run "$WK_BIN" new
    assert_failure
}

@test "new with empty title fails" {
    run "$WK_BIN" new ""
    assert_failure
}

@test "new invalid type fails" {
    run "$WK_BIN" new epic "My epic"
    assert_failure
}

# Hidden --priority flag tests
@test "new with --priority=0 adds priority:0 label" {
    id=$(create_issue task "Critical task" --priority 0)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:0"
}

@test "new with --priority=4 adds priority:4 label" {
    id=$(create_issue task "Low priority" --priority 4)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:4"
}

@test "new with --priority and --label combines both" {
    id=$(create_issue task "Labeled priority" --priority 1 --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:1"
    assert_output --partial "backend"
}

@test "new with invalid priority value fails" {
    run "$WK_BIN" new task "Invalid" --priority 5
    assert_failure
}

@test "new with negative priority value fails" {
    run "$WK_BIN" new task "Invalid" --priority -1
    assert_failure
}

@test "new with non-numeric priority fails" {
    run "$WK_BIN" new task "Invalid" --priority high
    assert_failure
}

# Hidden --description flag tests
@test "new with --description adds note" {
    id=$(create_issue task "Described task" --description "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"
}

@test "new with --description shows as Description in output" {
    id=$(create_issue task "Described task" --description "My description")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "My description"
}

@test "new with --note works (documented happy path)" {
    id=$(create_issue task "Noted task" --note "Initial note")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "Initial note"
}

@test "new with --description and --label combines both" {
    id=$(create_issue task "Labeled described" --description "Context" --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Context"
    assert_output --partial "backend"
}

# Comma-separated labels tests
@test "new with comma-separated labels adds all labels" {
    id=$(create_issue task "Comma labels" --label "a,b,c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: a, b, c"
}

@test "new with comma-separated and multiple --label combines all" {
    id=$(create_issue task "Mixed labels" --label "a,b" --label "c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels:"
    assert_output --partial "a"
    assert_output --partial "b"
    assert_output --partial "c"
}

@test "new with whitespace around comma-separated labels trims correctly" {
    id=$(create_issue task "Whitespace labels" --label "  x  ,  y  ")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: x, y"
}
