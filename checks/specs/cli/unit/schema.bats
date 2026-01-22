#!/usr/bin/env bats
load '../../helpers/common'

# Schema commands don't need an initialized project
setup_file() {
    file_setup
    # No init needed - schema command works without a project
}

@test "schema requires subcommand" {
    run "$WK_BIN" schema
    assert_failure
    assert_output --partial "Usage"
}

@test "schema list outputs valid JSON" {
    run "$WK_BIN" schema list
    assert_success

    # Validate it's JSON by parsing with jq
    echo "$output" | jq . > /dev/null
}

@test "schema list contains expected structure" {
    run "$WK_BIN" schema list
    assert_success

    # Check for key schema elements
    assert_output --partial '"$schema"'
    assert_output --partial '"issues"'
    assert_output --partial '"ListOutputJson"'
}

@test "schema show outputs valid JSON" {
    run "$WK_BIN" schema show
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema show includes nested types" {
    run "$WK_BIN" schema show
    assert_success

    assert_output --partial '"Note"'
    assert_output --partial '"Link"'
    assert_output --partial '"Event"'
}

@test "schema ready outputs valid JSON" {
    run "$WK_BIN" schema ready
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema search outputs valid JSON" {
    run "$WK_BIN" schema search
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema search includes 'more' field" {
    run "$WK_BIN" schema search
    assert_success
    assert_output --partial '"more"'
}

@test "all schemas have \$schema field" {
    for cmd in list show ready search; do
        run "$WK_BIN" schema "$cmd"
        assert_success
        assert_output --partial '"$schema"'
    done
}

@test "schema -h shows help" {
    run "$WK_BIN" schema -h
    assert_success
    assert_output --partial "list"
    assert_output --partial "show"
    assert_output --partial "ready"
    assert_output --partial "search"
}

@test "schema help shows examples" {
    run "$WK_BIN" schema --help
    assert_success
    assert_output --partial "wk schema list"
    assert_output --partial "wk schema show"
}

@test "schema list includes issue type enum values" {
    run "$WK_BIN" schema list
    assert_success

    # All issue types should be in schema
    assert_output --partial '"feature"'
    assert_output --partial '"task"'
    assert_output --partial '"bug"'
    assert_output --partial '"chore"'
    assert_output --partial '"idea"'
}

@test "schema list includes status enum values" {
    run "$WK_BIN" schema list
    assert_success

    # All statuses should be in schema
    assert_output --partial '"todo"'
    assert_output --partial '"in_progress"'
    assert_output --partial '"done"'
    assert_output --partial '"closed"'
}

@test "schema show includes datetime format" {
    run "$WK_BIN" schema show
    assert_success

    # DateTime fields should have date-time format
    assert_output --partial '"date-time"'
}
