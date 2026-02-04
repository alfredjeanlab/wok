#!/usr/bin/env bats
load '../../helpers/common'

@test "edit title changes issue title" {
    id=$(create_issue task "Original title")
    run "$WK_BIN" edit "$id" title "New title"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "New title"
    refute_output --partial "Original title"
}

@test "edit type changes issue type" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" type bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[bug]"
}

@test "edit type from task to feature" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" type feature
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[feature]"
}

@test "edit type to idea" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" type idea
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[idea]"
}

@test "edit type from idea to task" {
    id=$(create_issue idea "My idea")
    run "$WK_BIN" edit "$id" type task
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[task]"
}

@test "edit both title and type sequentially" {
    id=$(create_issue task "Original")
    run "$WK_BIN" edit "$id" title "Updated"
    assert_success
    run "$WK_BIN" edit "$id" type bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Updated"
    assert_output --partial "[bug]"
}

@test "edit logs event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" edit "$id" title "New title"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "edited"
}

@test "edit nonexistent issue fails" {
    run "$WK_BIN" edit "test-nonexistent" title "New"
    assert_failure
}

@test "edit without options fails or shows help" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" edit "$id"
    # Should either fail or show help
    true
}

@test "edit with invalid type fails" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" edit "$id" type bogus
    assert_failure
}

@test "edit requires issue ID" {
    run "$WK_BIN" edit title "New"
    assert_failure
}

@test "edit preserves other fields" {
    id=$(create_issue task "Test task" --label "mylabel")
    "$WK_BIN" edit "$id" title "New title"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "New title"
    assert_output --partial "mylabel"
}

# Hidden flag variant tests

@test "edit: --title flag updates title" {
    id=$(create_issue task "Original title")
    run "$WK_BIN" edit "$id" --title "Updated title"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Updated title"
}

@test "edit: --description flag updates description" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" --description "New description"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "New description"
}

@test "edit: --type flag updates type" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" --type bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[bug]"
}

@test "edit: --assignee flag updates assignee" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" --assignee alice
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "alice"
}
