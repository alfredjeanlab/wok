#!/usr/bin/env bats
load '../../helpers/common'

# Version flag tests - verifies -v, --version, and -V behavior.
# NOTE: These tests only check version output and don't need wk init.

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

# Positive tests - flags work correctly

@test "--version outputs version" {
    run "$WK_BIN" --version
    assert_success
    assert_output --partial "wok"
    # Version should be semver-like
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-v outputs version" {
    run "$WK_BIN" -v
    assert_success
    assert_output --partial "wok"
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-V outputs version (silent alias)" {
    run "$WK_BIN" -V
    assert_success
    assert_output --partial "wok"
    assert_output --regexp "[0-9]+\.[0-9]+\.[0-9]+"
}

@test "-v and --version produce identical output" {
    run "$WK_BIN" -v
    local v_output="$output"
    run "$WK_BIN" --version
    [ "$v_output" = "$output" ]
}

@test "-V produces same output as -v" {
    run "$WK_BIN" -v
    local v_output="$output"
    run "$WK_BIN" -V
    [ "$v_output" = "$output" ]
}

# Negative tests - help output

@test "-v is documented in help" {
    run "$WK_BIN" --help
    assert_success
    assert_output --partial "-v"
    assert_output --partial "--version"
}

@test "-V is NOT documented in help" {
    run "$WK_BIN" --help
    assert_success
    # -V should be hidden
    refute_output --regexp "\s-V[,\s]"
    refute_output --partial "[-V"
}

@test "version subcommand does not exist" {
    # This test exists in no_aliases.bats but we reinforce it here
    run "$WK_BIN" version
    assert_failure
}
