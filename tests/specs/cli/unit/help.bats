#!/usr/bin/env bats
load '../../helpers/common'


# Help command tests - verifies help output, flags, and structure.
# NOTE: These tests only check help output and don't need wk init.

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

@test "help displays usage information and available commands" {
    run "$WK_BIN" help
    assert_success
    [ -n "$output" ]
    [ "${#output}" -gt 100 ]
    assert_output --partial "wk"
    assert_output --partial "collaborative"
    assert_output --partial "offline-first"
    assert_output --partial "AI-friendly"
    assert_output --partial "issue tracker"
    assert_output --partial "init"
    assert_output --partial "new"
    assert_output --partial "list"
}

@test "--help and -h flags work" {
    run "$WK_BIN" --help
    assert_success
    assert_output --partial "wk"

    run "$WK_BIN" -h
    assert_success
    assert_output --partial "collaborative"
    assert_output --partial "issue tracker"

    # -h and --help produce same output for commands
    run "$WK_BIN" list -h
    local h_output="$output"
    run "$WK_BIN" list --help
    [ "$h_output" = "$output" ]
}

@test "all commands support -h flag" {
    # Note: 'help' subcommand excluded - it expects a command name, not flags
    local commands=(init new start done close reopen edit list show tree dep undep label unlabel note log export)
    for cmd in "${commands[@]}"; do
        run "$WK_BIN" "$cmd" -h
        assert_success
        assert_output --partial "$cmd"
    done
}

@test "all commands support --help flag" {
    local commands=(init new start done close reopen edit list show tree dep undep label unlabel note log export)
    for cmd in "${commands[@]}"; do
        run "$WK_BIN" "$cmd" --help
        assert_success
        assert_output --partial "$cmd"
    done
}

@test "help <command> works for all commands" {
    local commands=(init new start done close reopen edit list show tree dep undep label unlabel note log export help)
    for cmd in "${commands[@]}"; do
        run "$WK_BIN" help "$cmd"
        assert_success
        assert_output --partial "$cmd"
    done
}

@test "command help shows usage and options" {
    # init shows prefix option
    run "$WK_BIN" help init
    assert_success
    assert_output --partial "prefix" || assert_output --partial "--"

    # list shows filter options
    run "$WK_BIN" help list
    assert_success
    assert_output --partial "status" || \
    assert_output --partial "tag" || \
    assert_output --partial "type" || \
    assert_output --partial "--"

    # dep shows relationship types
    run "$WK_BIN" help dep
    assert_success
    assert_output --partial "blocks" || \
    assert_output --partial "tracks" || \
    assert_output --partial "relationship"
}

@test "help for unknown command fails gracefully" {
    run "$WK_BIN" help nonexistent
    assert_failure
}

@test "hidden flags not shown in help" {
    # --priority and --description hidden in new --help
    run "$WK_BIN" new --help
    assert_success
    refute_output --partial "priority"
    refute_output --partial "--description"
    # but --note is shown
    assert_output --partial "--note"
    assert_output --partial "-n"

    # hidden in help new too
    run "$WK_BIN" help new
    assert_success
    refute_output --partial "priority"
    refute_output --partial "--description"

    # hidden in main --help
    run "$WK_BIN" --help
    assert_success
    refute_output --partial "priority"
    refute_output --partial "--description"
}
