#!/usr/bin/env bats
load '../../helpers/common'

@test "list on empty database succeeds" {
    init_project
    run "$WK_BIN" list
    assert_success
}

@test "list --blocked on empty database succeeds" {
    init_project
    run "$WK_BIN" list --blocked
    assert_success
}

@test "list --all on empty database succeeds" {
    init_project
    run "$WK_BIN" list --all
    assert_success
}

@test "log on empty database succeeds" {
    init_project
    run "$WK_BIN" log
    assert_success
}

@test "export empty database succeeds" {
    init_project
    run "$WK_BIN" export "$TEST_DIR/empty.jsonl"
    assert_success
}

@test "show nonexistent on empty database fails" {
    init_project
    run "$WK_BIN" show "test-nonexistent"
    assert_failure
}

@test "start nonexistent on empty database fails" {
    init_project
    run "$WK_BIN" start "test-nonexistent"
    assert_failure
}

@test "status filter on empty database succeeds" {
    init_project
    run "$WK_BIN" list --status in_progress
    assert_success
}

@test "type filter on empty database succeeds" {
    init_project
    run "$WK_BIN" list --type bug
    assert_success
}

@test "tag filter on empty database succeeds" {
    init_project
    run "$WK_BIN" list --label "nonexistent"
    assert_success
}
