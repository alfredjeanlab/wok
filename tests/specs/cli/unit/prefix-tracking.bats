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

@test "config prefixes shows all prefixes with counts" {
    mkdir -p prefixes_list && cd prefixes_list
    run "$WK_BIN" init --prefix proj --private
    assert_success

    create_issue task "Proj task 1"
    create_issue task "Proj task 2"
    "$WK_BIN" new "Other task" --prefix other

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "proj: 2 issues"
    assert_output --partial "other: 1 issue"
    assert_output --partial "(default)"
}

@test "config prefixes with empty database shows no prefixes" {
    mkdir -p prefixes_empty && cd prefixes_empty
    run "$WK_BIN" init --prefix test --private
    assert_success

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "No prefixes found"
}

@test "config prefixes -o json outputs valid JSON" {
    mkdir -p prefixes_json && cd prefixes_json
    run "$WK_BIN" init --prefix main --private
    assert_success

    create_issue task "Test task"

    run "$WK_BIN" config prefixes -o json
    assert_success

    # Should be valid JSON
    echo "$output" | jq -e . >/dev/null

    # Check structure
    echo "$output" | jq -e '.default == "main"' >/dev/null
    echo "$output" | jq -e '.prefixes | length > 0' >/dev/null
    echo "$output" | jq -e '.prefixes[0].prefix == "main"' >/dev/null
    echo "$output" | jq -e '.prefixes[0].issue_count == 1' >/dev/null
    echo "$output" | jq -e '.prefixes[0].is_default == true' >/dev/null
}

@test "config prefixes -o id outputs only prefix names" {
    mkdir -p prefixes_id && cd prefixes_id
    run "$WK_BIN" init --prefix main --private
    assert_success

    create_issue task "Main task"
    "$WK_BIN" new "Other task" --prefix api

    run "$WK_BIN" config prefixes -o id
    assert_success
    assert_output --partial "main"
    assert_output --partial "api"
    # Should not include counts or markers
    refute_output --partial "issue"
    refute_output --partial "(default)"
}

@test "new --prefix creates issue with different prefix" {
    mkdir -p prefix_new && cd prefix_new
    run "$WK_BIN" init --prefix main --private
    assert_success

    run "$WK_BIN" new "Other project task" --prefix other -o id
    assert_success
    [[ "$output" == other-* ]]

    # Issue should be visible
    run "$WK_BIN" show "$output"
    assert_success
    assert_output --partial "Other project task"
}

@test "new -p short form works for prefix" {
    mkdir -p prefix_short && cd prefix_short
    run "$WK_BIN" init --prefix main --private
    assert_success

    run "$WK_BIN" new "Short prefix" -p short -o id
    assert_success
    [[ "$output" == short-* ]]
}

@test "new --prefix rejects invalid prefix" {
    mkdir -p prefix_invalid && cd prefix_invalid
    run "$WK_BIN" init --prefix main --private
    assert_success

    # Too short
    run "$WK_BIN" new "Invalid" --prefix a
    assert_failure

    # Contains dash
    run "$WK_BIN" new "Invalid" --prefix my-proj
    assert_failure

    # Uppercase
    run "$WK_BIN" new "Invalid" --prefix ABC
    assert_failure
}

@test "new --prefix updates prefix table" {
    mkdir -p prefix_table && cd prefix_table
    run "$WK_BIN" init --prefix main --private
    assert_success

    create_issue task "Main task"
    "$WK_BIN" new "API task" --prefix api

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "main: 1 issue"
    assert_output --partial "api: 1 issue"
}

@test "new --prefix works with all flags" {
    mkdir -p prefix_flags && cd prefix_flags
    run "$WK_BIN" init --prefix main --private
    assert_success

    # Combine with --label, --note, type
    run "$WK_BIN" new bug "Flagged issue" --prefix other --label urgent --note "Description"
    assert_success
    assert_output --partial "other-"
    assert_output --partial "[bug]"

    # Get the ID
    id=$("$WK_BIN" new task "Another" --prefix other -o id)

    # Verify dependencies work
    "$WK_BIN" new task "Blocked" --prefix other --blocked-by "$id"

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "other: 3 issues"
}

@test "config rename updates prefixes table" {
    mkdir -p prefix_rename && cd prefix_rename
    run "$WK_BIN" init --prefix old --private
    assert_success

    create_issue task "Test"

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "old: 1 issue"

    run "$WK_BIN" config rename old new
    assert_success

    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "new: 1 issue"
    refute_output --partial "old:"
}

@test "existing databases backfill prefixes table" {
    mkdir -p prefix_backfill && cd prefix_backfill
    run "$WK_BIN" init --prefix proj --private
    assert_success

    # Create issues first
    create_issue task "Task 1"
    create_issue task "Task 2"

    # The prefixes should be auto-populated from existing issues
    run "$WK_BIN" config prefixes
    assert_success
    assert_output --partial "proj: 2 issues"
}
