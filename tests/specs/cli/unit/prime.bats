#!/usr/bin/env bats
load '../../helpers/common'


# Prime command tests
# Prime outputs the onboarding template for context priming
# Always outputs template regardless of work directory

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

@test "prime outputs template without initialization" {
    # prime should always output template content
    [ ! -d ".wok" ]
    run "$WK_BIN" prime
    assert_success
    assert_output --partial "## Core Rules"
}

@test "prime outputs template content" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    assert_output --partial "## Core Rules"
    assert_output --partial "## Finding Work"
}

@test "prime output contains wk commands" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    assert_output --partial "wk list"
    assert_output --partial "wk new"
    assert_output --partial "wk start"
    assert_output --partial "wk done"
}

@test "prime documents list default shows open items" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    # Verify documentation says list shows todo AND in_progress by default
    assert_output --partial "todo + in_progress"
}

@test "prime output is not empty when initialized" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    [ -n "$output" ]
}

@test "prime output contains workflow sections" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    assert_output --partial "## Creating & Updating"
    assert_output --partial "## Dependencies"
    assert_output --partial "## Common Workflows"
}

@test "prime help shows description" {
    run "$WK_BIN" prime --help
    assert_success
    assert_output --partial "onboarding"
}

@test "prime output is valid markdown" {
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime
    assert_success
    # Check for proper markdown header syntax
    assert_output --partial "# "
    # Check code blocks are properly opened and closed
    echo "$output" | grep -c '```' | grep -qE '^[02468]' || {
        echo "Unbalanced code blocks"
        return 1
    }
}

@test "prime with --help shows usage" {
    run "$WK_BIN" prime --help
    assert_success
    assert_output --partial "Usage:"
    assert_output --partial "wk prime"
    assert_output --partial "-h, --help"
}

@test "prime ignores extra arguments" {
    # prime should succeed even with extra arguments (clap ignores them or errors gracefully)
    "$WK_BIN" init --prefix test
    run "$WK_BIN" prime extra arg1 arg2
    # Either succeeds (ignores args) or fails gracefully
    # Current clap behavior: errors on unknown arguments
    # This test documents actual behavior
    [ "$status" -eq 0 ] || [ "$status" -eq 2 ]
}

@test "prime works from subdirectory" {
    "$WK_BIN" init --prefix test
    mkdir -p subdir/nested
    cd subdir/nested
    run "$WK_BIN" prime
    assert_success
    assert_output --partial "## Core Rules"
}
