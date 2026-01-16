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

@test "help displays usage information" {
    run "$WK_BIN" help
    assert_success
    assert_output --partial "wk"
    assert_output --partial "collaborative"
    assert_output --partial "offline-first"
    assert_output --partial "AI-friendly"
    assert_output --partial "issue tracker"
}

@test "help shows available commands" {
    run "$WK_BIN" help
    assert_success
    assert_output --partial "init"
    assert_output --partial "new"
    assert_output --partial "list"
}

@test "--help flag works" {
    run "$WK_BIN" --help
    assert_success
    assert_output --partial "wk"
}

@test "-h flag works" {
    run "$WK_BIN" -h
    assert_success
    assert_output --partial "wk"
}

@test "help <command> shows command-specific help" {
    run "$WK_BIN" help new
    assert_success
    assert_output --partial "new"
}

@test "help init shows init-specific help" {
    run "$WK_BIN" help init
    assert_success
    assert_output --partial "init"
    assert_output --partial "prefix"
}

@test "<command> --help shows command help" {
    run "$WK_BIN" new --help
    assert_success
    assert_output --partial "new"
}

@test "<command> -h shows command help" {
    run "$WK_BIN" list -h
    assert_success
    assert_output --partial "list"
}

@test "help for unknown command fails gracefully" {
    run "$WK_BIN" help nonexistent
    assert_failure
}

# Hidden flag verification - ensure --priority is not shown in help
@test "wk new --help does not show --priority" {
    run "$WK_BIN" new --help
    assert_success
    refute_output --partial "priority"
}

@test "wk help new does not show --priority" {
    run "$WK_BIN" help new
    assert_success
    refute_output --partial "priority"
}

@test "wk --help does not mention priority flag" {
    run "$WK_BIN" --help
    assert_success
    refute_output --partial "priority"
}

# Hidden flag tests for --description
@test "wk new --help does not show --description" {
    run "$WK_BIN" new --help
    assert_success
    refute_output --partial "--description"
}

@test "wk help new does not show --description" {
    run "$WK_BIN" help new
    assert_success
    refute_output --partial "--description"
}

@test "wk --help does not mention description flag" {
    run "$WK_BIN" --help
    assert_success
    refute_output --partial "--description"
}

@test "wk new --help shows --note (documented path)" {
    run "$WK_BIN" new --help
    assert_success
    assert_output --partial "--note"
    assert_output --partial "-n"
}
