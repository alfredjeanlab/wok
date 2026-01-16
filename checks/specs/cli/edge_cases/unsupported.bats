#!/usr/bin/env bats
load '../../helpers/common'

# Tests to verify unsupported commands don't exist in help or when called
# Supported commands per REQUIREMENTS.md:
# help, init, new, start, stop, done, close, reopen, edit, list, show,
# tree, dep, undep, label, unlabel, note, log, export

# Test unsupported commands fail when called

@test "unsupported command 'delete' fails" {
    init_project
    run "$WK_BIN" delete test-abc
    assert_failure
}

@test "unsupported command 'rm' fails" {
    init_project
    run "$WK_BIN" rm test-abc
    assert_failure
}

@test "unsupported command 'remove' fails" {
    init_project
    run "$WK_BIN" remove test-abc
    assert_failure
}

@test "unsupported command 'status' fails" {
    init_project
    run "$WK_BIN" status
    assert_failure
}

@test "unsupported command 'add' fails" {
    init_project
    run "$WK_BIN" add "Test task"
    assert_failure
}

@test "unsupported command 'create' fails" {
    init_project
    run "$WK_BIN" create "Test task"
    assert_failure
}

@test "unsupported command 'open' fails" {
    init_project
    run "$WK_BIN" open test-abc
    assert_failure
}

@test "unsupported command 'update' fails" {
    init_project
    run "$WK_BIN" update test-abc
    assert_failure
}

@test "unsupported command 'get' fails" {
    init_project
    run "$WK_BIN" get test-abc
    assert_failure
}

@test "unsupported command 'config' fails" {
    run "$WK_BIN" config
    assert_failure
}

@test "unsupported command 'push' fails" {
    init_project
    run "$WK_BIN" push
    assert_failure
}

@test "unsupported command 'pull' fails" {
    init_project
    run "$WK_BIN" pull
    assert_failure
}

@test "unsupported command 'version' fails" {
    run "$WK_BIN" version
    assert_failure
}

@test "unsupported command 'info' fails" {
    init_project
    run "$WK_BIN" info
    assert_failure
}

@test "unsupported command 'find' fails" {
    init_project
    run "$WK_BIN" find "test"
    assert_failure
}

@test "unsupported command 'archive' fails" {
    init_project
    run "$WK_BIN" archive test-abc
    assert_failure
}

@test "unsupported command 'import' fails" {
    init_project
    run "$WK_BIN" import file.json
    assert_failure
}

# Test unsupported commands don't appear in help output

@test "help does not mention 'delete'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "delete"
}

@test "help does not mention 'rm'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial " rm "
    refute_output --regexp "^rm "
}

@test "help does not mention 'remove'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "remove"
}

@test "help does not mention 'status'" {
    run "$WK_BIN" help
    assert_success
    # status might appear as --status flag, but not as a command
    refute_output --regexp "^\s+status\s"
}

@test "help does not mention 'add' as command" {
    run "$WK_BIN" help
    assert_success
    refute_output --regexp "^\s+add\s"
}

@test "help does not mention 'create'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "create"
}

@test "help does not mention 'update'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "update"
}

@test "help does not mention 'push'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "push"
}

@test "help does not mention 'pull'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "pull"
}

@test "help does not mention 'version'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "version"
}

@test "help does not mention 'archive'" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "archive"
}

# Note: 'import' IS a supported command now - see REQUIREMENTS.md section 3

# Verify that help <unsupported> fails properly

@test "help delete fails" {
    run "$WK_BIN" help delete
    assert_failure
}

@test "help add fails" {
    run "$WK_BIN" help add
    assert_failure
}

@test "help create fails" {
    run "$WK_BIN" help create
    assert_failure
}

@test "help status fails" {
    run "$WK_BIN" help status
    assert_failure
}

# Note: 'config' IS a supported command - manages configuration settings

@test "help version fails" {
    run "$WK_BIN" help version
    assert_failure
}
