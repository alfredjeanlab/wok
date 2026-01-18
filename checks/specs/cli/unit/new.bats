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

@test "new creates issues with correct type (default task, feature, bug, chore)" {
    # Default creates task
    run "$WK_BIN" new "NewType My task"
    assert_success
    assert_output --partial "[task]"

    # Explicit task
    run "$WK_BIN" new task "NewType My explicit task"
    assert_success
    assert_output --partial "[task]"

    # Feature
    run "$WK_BIN" new feature "NewType My feature"
    assert_success
    assert_output --partial "[feature]"

    # Bug
    run "$WK_BIN" new bug "NewType My bug"
    assert_success
    assert_output --partial "[bug]"

    # Chore
    run "$WK_BIN" new chore "NewType My chore"
    assert_success
    assert_output --partial "[chore]"

    # Starts in todo status
    run "$WK_BIN" new "NewType Status task"
    assert_success
    assert_output --partial "(todo)"
}

@test "new with --label and --note adds metadata" {
    # --label adds label
    id=$(create_issue task "NewMeta Labeled task" --label "project:auth")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"

    # Multiple --label adds all labels
    id=$(create_issue task "NewMeta Multi-labeled task" --label "project:auth" --label "priority:high")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"
    assert_output --partial "priority:high"

    # --note adds note
    id=$(create_issue task "NewMeta Noted task" --note "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"
}

@test "new generates unique ID with prefix and validates inputs" {
    # Generate ID with prefix (use subdirectory for different prefix)
    mkdir -p subproj && cd subproj
    "$WK_BIN" init --prefix myproj
    run "$WK_BIN" new "Test task"
    assert_success
    assert_output --partial "myproj-"
    cd ..

    # Requires title
    run "$WK_BIN" new
    assert_failure

    # Empty title fails
    run "$WK_BIN" new ""
    assert_failure

    # Invalid type fails
    run "$WK_BIN" new epic "My epic"
    assert_failure
}

@test "new with --priority adds priority label" {
    # --priority=0 adds priority:0 label
    id=$(create_issue task "NewPrio Critical task" --priority 0)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:0"

    # --priority=4 adds priority:4 label
    id=$(create_issue task "NewPrio Low priority" --priority 4)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:4"

    # --priority and --label combine
    id=$(create_issue task "NewPrio Labeled priority" --priority 1 --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:1"
    assert_output --partial "backend"

    # Invalid priority values fail
    run "$WK_BIN" new task "Invalid" --priority 5
    assert_failure

    run "$WK_BIN" new task "Invalid" --priority -1
    assert_failure

    run "$WK_BIN" new task "Invalid" --priority high
    assert_failure
}

@test "new with --description adds note as Description" {
    # --description adds note
    id=$(create_issue task "NewDesc Described task" --description "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"

    # Shows as Description in output
    id=$(create_issue task "NewDesc Described task 2" --description "My description")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "My description"

    # --note also works (documented happy path)
    id=$(create_issue task "NewDesc Noted task" --note "Initial note")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "Initial note"

    # --description and --label combine
    id=$(create_issue task "NewDesc Labeled described" --description "Context" --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Context"
    assert_output --partial "backend"
}

@test "new with comma-separated labels adds all labels" {
    # Comma-separated labels
    id=$(create_issue task "NewComma Comma labels" --label "a,b,c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: a, b, c"

    # Comma-separated and multiple --label combine
    id=$(create_issue task "NewComma Mixed labels" --label "a,b" --label "c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels:"
    assert_output --partial "a"
    assert_output --partial "b"
    assert_output --partial "c"

    # Whitespace around comma-separated labels trims correctly
    id=$(create_issue task "NewComma Whitespace labels" --label "  x  ,  y  ")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: x, y"
}
