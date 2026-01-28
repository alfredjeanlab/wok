#!/usr/bin/env bats
load '../../helpers/common'

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

@test "BATS_FILE_TMPDIR exists and is writable" {
    [ -d "$BATS_FILE_TMPDIR" ]
    touch "$BATS_FILE_TMPDIR/test_file"
    [ -f "$BATS_FILE_TMPDIR/test_file" ]
}

@test "HOME is isolated to test directory" {
    [[ "$HOME" == "$BATS_FILE_TMPDIR" ]]
}

@test "test_setup returns to BATS_FILE_TMPDIR" {
    cd /tmp
    test_setup
    [ "$PWD" = "$BATS_FILE_TMPDIR" ]
}

@test "WK_BIN is set and executable" {
    [ -n "$WK_BIN" ]
    [ -x "$WK_BIN" ] || command -v "$WK_BIN" >/dev/null
}

@test "WK_BIN runs successfully" {
    run "$WK_BIN" --version
    assert_success
}
