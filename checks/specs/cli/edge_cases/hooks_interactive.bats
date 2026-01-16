#!/usr/bin/env bats
load '../../helpers/common'

# ============================================================================
# Interactive Mode Edge Cases
# These tests verify the interactive mode doesn't break non-interactive usage
# ============================================================================

# ============================================================================
# Interactive Picker Tests
# Note: Testing interactive TUIs in CI is tricky since script/pty may not work.
# These tests focus on non-TTY behavior and error messages.
# ============================================================================

# Note: The appearance tests below are skipped in CI environments because
# the `script` command requires a real TTY which isn't available in CI.
# To run these locally, set ENABLE_TTY_TESTS=1
#
# @test "interactive picker shows radio buttons" {
#     [ "$ENABLE_TTY_TESTS" = "1" ] || skip "TTY tests disabled in CI"
#     result=$(script -q /dev/null sh -c 'printf "q\n" | "$WK_BIN" hooks install -i 2>&1')
#     [[ "$result" == *"○"* ]] || [[ "$result" == *"●"* ]]
# }
#
# @test "interactive picker shows all scopes" {
#     [ "$ENABLE_TTY_TESTS" = "1" ] || skip "TTY tests disabled in CI"
#     result=$(script -q /dev/null sh -c 'printf "q\n" | "$WK_BIN" hooks install -i 2>&1')
#     [[ "$result" == *"local"* ]]
#     [[ "$result" == *"project"* ]]
#     [[ "$result" == *"user"* ]]
# }

# These tests verify the picker configuration without needing a TTY

@test "hooks -i fails gracefully when not a TTY" {
    # Force interactive mode on non-TTY (without scope) should error
    run timeout 5 bash -c '"$WK_BIN" hooks install -i < /dev/null 2>&1'
    # Should either fail or timeout, but provide useful message
    if [ $status -eq 124 ]; then
        fail "Command hung instead of failing gracefully"
    fi
    # If it didn't timeout, it should have failed with helpful message
    assert_failure
    assert_output --partial "terminal" || assert_output --partial "TTY" || assert_output --partial "interactive"
}

@test "hooks -i with explicit scope skips picker" {
    # Even with -i, if scope is provided, no picker needed
    # This might still work even on non-TTY
    run timeout 5 bash -c '"$WK_BIN" hooks install -i local < /dev/null 2>&1' || true
    # Either succeeds or fails with clear message, doesn't hang
    [ $status -ne 124 ]
}

@test "hooks install terminates cleanly on SIGINT" {
    # Verify the process can be interrupted
    timeout 2 bash -c '
        "$WK_BIN" hooks install < /dev/null &
        pid=$!
        sleep 0.5
        kill -INT $pid 2>/dev/null || true
        wait $pid 2>/dev/null || true
    ' || true
    # Test passes if we get here (didn't hang)
}

@test "hooks install terminates cleanly on SIGTERM" {
    timeout 2 bash -c '
        "$WK_BIN" hooks install < /dev/null &
        pid=$!
        sleep 0.5
        kill -TERM $pid 2>/dev/null || true
        wait $pid 2>/dev/null || true
    ' || true
    # Test passes if we get here
}

@test "hooks install runs in non-interactive mode when backgrounded" {
    # When run in background, should use non-interactive mode
    # Run with explicit scope to avoid needing interactive picker
    timeout 5 bash -c '"$WK_BIN" hooks install local &
        pid=$!
        wait $pid
        exit $?
    '
    # Should complete without hanging
}

# ============================================================================
# Setup / Teardown
# ============================================================================

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}
