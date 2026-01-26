#!/usr/bin/env bats
load '../../helpers/common'

# Per-test isolation - each test calls init_project
setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

# Tests for remote command in local mode (no remote configured)

@test "remote status in local mode shows not applicable" {
    init_project
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "not"
}

@test "remote sync in local mode is silent" {
    run "$WK_BIN" init --prefix test --local
    assert_success
    run "$WK_BIN" remote sync
    assert_success
    assert_output ""
}

@test "remote status provides configuration hint" {
    run "$WK_BIN" init --prefix test --local
    assert_success
    run "$WK_BIN" remote status
    assert_success
    assert_output --partial "[remote]"
    assert_output --partial "url"
}

@test "remote help shows subcommands" {
    run "$WK_BIN" help remote
    assert_success
    assert_output --partial "status"
    assert_output --partial "sync"
}

@test "remote --help shows subcommands" {
    run "$WK_BIN" remote --help
    assert_success
    assert_output --partial "status"
    assert_output --partial "sync"
}

@test "remote appears in main help" {
    run "$WK_BIN" help
    assert_success
    assert_output --partial "remote"
}
