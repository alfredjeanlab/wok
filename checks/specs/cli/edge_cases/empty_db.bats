#!/usr/bin/env bats
load '../../helpers/common'


# Tests verifying commands handle empty database gracefully.
@test "read commands succeed on empty database" {
    local commands=(
        "list"
        "list --blocked"
        "list --all"
        "list --status in_progress"
        "list --type bug"
        "list --label nonexistent"
        "log"
    )
    for cmd in "${commands[@]}"; do
        run $WK_BIN $cmd
        assert_success
    done
}

@test "export succeeds on empty database" {
    run "$WK_BIN" export "$BATS_FILE_TMPDIR/empty.jsonl"
    assert_success
}

@test "commands on nonexistent issues fail" {
    local commands=(
        "show test-nonexistent"
        "start test-nonexistent"
    )
    for cmd in "${commands[@]}"; do
        run $WK_BIN $cmd
        assert_failure
    done
}
