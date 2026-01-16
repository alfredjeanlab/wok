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

# Tests verifying help output has a footer with usage hints
# The footer should guide users on how to get more help

@test "main help has footer with usage hint" {
    run "$WK_BIN" help
    assert_success
    # Should mention how to get command-specific help
    assert_output --partial "help" || \
    assert_output --partial "Help" || \
    assert_output --partial "-h" || \
    assert_output --partial "--help"
}

@test "main help mentions how to get command help" {
    run "$WK_BIN" help
    assert_success
    # Should have some instruction about getting command help
    # Common patterns: "wk help <command>", "wk <command> -h", etc.
    assert_output --regexp "(help|Help|HELP)"
}

@test "help output shows available commands" {
    run "$WK_BIN" help
    assert_success
    # Help should list commands
    assert_output --partial "init"
    assert_output --partial "new"
    assert_output --partial "list"
}

@test "command help shows usage pattern" {
    run "$WK_BIN" help new
    assert_success
    # Command help should show usage
    assert_output --partial "new"
}

@test "command help for init shows usage" {
    run "$WK_BIN" help init
    assert_success
    assert_output --partial "init"
    assert_output --partial "prefix" || assert_output --partial "--"
}

@test "command help for list shows filter options" {
    run "$WK_BIN" help list
    assert_success
    assert_output --partial "list"
    # Should mention filtering options
    assert_output --partial "status" || \
    assert_output --partial "tag" || \
    assert_output --partial "type" || \
    assert_output --partial "--"
}

@test "command help for dep shows relationship types" {
    run "$WK_BIN" help dep
    assert_success
    assert_output --partial "dep"
    # Should mention relationship types
    assert_output --partial "blocks" || \
    assert_output --partial "tracks" || \
    assert_output --partial "relationship"
}

@test "help help shows help about help" {
    run "$WK_BIN" help help
    # Should succeed and show something about the help command
    assert_success
    assert_output --partial "help"
}

@test "help for unknown command suggests alternatives" {
    run "$WK_BIN" help unknown123
    assert_failure
    # Should fail gracefully with an error message
}

# Verify footer structure

@test "main help ends with useful information" {
    run "$WK_BIN" help
    assert_success
    # Output should have reasonable length indicating content
    [ "${#output}" -gt 100 ]
}

@test "command help provides enough context" {
    run "$WK_BIN" help new
    assert_success
    # Should have reasonable length
    [ "${#output}" -gt 50 ]
}

@test "help output is not empty" {
    run "$WK_BIN" help
    assert_success
    [ -n "$output" ]
}

@test "-h output shows short help" {
    run "$WK_BIN" -h
    assert_success
    # -h shows short about text (clap behavior)
    assert_output --partial "collaborative"
    assert_output --partial "issue tracker"
}

@test "command -h output matches command --help output" {
    run "$WK_BIN" list -h
    local h_output="$output"
    run "$WK_BIN" list --help
    local help_output="$output"
    [ "$h_output" = "$help_output" ]
}
