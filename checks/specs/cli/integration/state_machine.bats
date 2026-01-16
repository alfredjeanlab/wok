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

# Valid transitions

@test "todo -> in_progress via start" {
    id=$(create_issue task "Test")
    [ "$(get_status "$id")" = "(todo)" ]
    run "$WK_BIN" start "$id"
    assert_success
    [ "$(get_status "$id")" = "(in_progress)" ]
}

@test "todo -> closed via close with reason" {
    id=$(create_issue task "Test")
    [ "$(get_status "$id")" = "(todo)" ]
    run "$WK_BIN" close "$id" --reason "won't fix"
    assert_success
    [ "$(get_status "$id")" = "(closed)" ]
}

@test "todo -> done via done with reason (prior)" {
    id=$(create_issue task "Test")
    [ "$(get_status "$id")" = "(todo)" ]
    run "$WK_BIN" done "$id" --reason "already completed"
    assert_success
    [ "$(get_status "$id")" = "(done)" ]
}

@test "in_progress -> todo via reopen" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    [ "$(get_status "$id")" = "(in_progress)" ]
    run "$WK_BIN" reopen "$id"
    assert_success
    [ "$(get_status "$id")" = "(todo)" ]
}

@test "in_progress -> done via done" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    [ "$(get_status "$id")" = "(in_progress)" ]
    run "$WK_BIN" done "$id"
    assert_success
    [ "$(get_status "$id")" = "(done)" ]
}

@test "in_progress -> closed via close with reason" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    [ "$(get_status "$id")" = "(in_progress)" ]
    run "$WK_BIN" close "$id" --reason "abandoned"
    assert_success
    [ "$(get_status "$id")" = "(closed)" ]
}

@test "done -> todo via reopen with reason" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    [ "$(get_status "$id")" = "(done)" ]
    run "$WK_BIN" reopen "$id" --reason "regression"
    assert_success
    [ "$(get_status "$id")" = "(todo)" ]
}

@test "closed -> todo via reopen with reason" {
    id=$(create_issue task "Test")
    "$WK_BIN" close "$id" --reason "duplicate"
    [ "$(get_status "$id")" = "(closed)" ]
    run "$WK_BIN" reopen "$id" --reason "not duplicate"
    assert_success
    [ "$(get_status "$id")" = "(todo)" ]
}

# Invalid transitions

@test "cannot reopen from todo" {
    id=$(create_issue task "Test")
    run "$WK_BIN" reopen "$id"
    assert_failure
}

@test "cannot done from todo without reason" {
    id=$(create_issue task "Test")
    run "$WK_BIN" done "$id"
    assert_failure
}

@test "cannot start from done" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" start "$id"
    assert_failure
}

@test "cannot start from closed" {
    id=$(create_issue task "Test")
    "$WK_BIN" close "$id" --reason "skip"
    run "$WK_BIN" start "$id"
    assert_failure
}

@test "cannot done from closed" {
    id=$(create_issue task "Test")
    "$WK_BIN" close "$id" --reason "skip"
    run "$WK_BIN" done "$id"
    assert_failure
}

# Reason requirements

@test "close requires reason" {
    id=$(create_issue task "Test")
    run "$WK_BIN" close "$id"
    assert_failure
}

@test "reopen requires reason" {
    id=$(create_issue task "Test")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" reopen "$id"
    assert_failure
}

# Multiple transitions

@test "can cycle through states" {
    id=$(create_issue task "Test")

    # todo -> in_progress -> todo -> in_progress -> done -> todo -> in_progress -> done
    "$WK_BIN" start "$id"
    "$WK_BIN" reopen "$id"
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" reopen "$id" --reason "more work"
    "$WK_BIN" start "$id"
    run "$WK_BIN" done "$id"
    assert_success
    [ "$(get_status "$id")" = "(done)" ]
}
