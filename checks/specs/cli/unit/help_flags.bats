#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying all commands support -h, --help, and help <cmd>
#
# NOTE: These tests only check help output and don't need wk init.
# Using file-level setup to avoid per-test temp directory overhead.

setup_file() {
    file_setup
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}
# Supported commands per REQUIREMENTS.md:
# help, init, new, start, done, close, reopen, edit, list, show,
# tree, dep, undep, label, unlabel, note, log, export

# Test -h flag works for all commands

@test "init -h shows help" {
    run "$WK_BIN" init -h
    assert_success
    assert_output --partial "init"
}

@test "new -h shows help" {
    run "$WK_BIN" new -h
    assert_success
    assert_output --partial "new"
}

@test "start -h shows help" {
    run "$WK_BIN" start -h
    assert_success
    assert_output --partial "start"
}

@test "done -h shows help" {
    run "$WK_BIN" done -h
    assert_success
    assert_output --partial "done"
}

@test "close -h shows help" {
    run "$WK_BIN" close -h
    assert_success
    assert_output --partial "close"
}

@test "reopen -h shows help" {
    run "$WK_BIN" reopen -h
    assert_success
    assert_output --partial "reopen"
}

@test "edit -h shows help" {
    run "$WK_BIN" edit -h
    assert_success
    assert_output --partial "edit"
}

@test "list -h shows help" {
    run "$WK_BIN" list -h
    assert_success
    assert_output --partial "list"
}

@test "show -h shows help" {
    run "$WK_BIN" show -h
    assert_success
    assert_output --partial "show"
}

@test "tree -h shows help" {
    run "$WK_BIN" tree -h
    assert_success
    assert_output --partial "tree"
}

@test "dep -h shows help" {
    run "$WK_BIN" dep -h
    assert_success
    assert_output --partial "dep"
}

@test "undep -h shows help" {
    run "$WK_BIN" undep -h
    assert_success
    assert_output --partial "undep"
}

@test "label -h shows help" {
    run "$WK_BIN" label -h
    assert_success
    assert_output --partial "label"
}

@test "unlabel -h shows help" {
    run "$WK_BIN" unlabel -h
    assert_success
    assert_output --partial "unlabel"
}

@test "note -h shows help" {
    run "$WK_BIN" note -h
    assert_success
    assert_output --partial "note"
}

@test "log -h shows help" {
    run "$WK_BIN" log -h
    assert_success
    assert_output --partial "log"
}

@test "export -h shows help" {
    run "$WK_BIN" export -h
    assert_success
    assert_output --partial "export"
}

# Test --help flag works for all commands

@test "init --help shows help" {
    run "$WK_BIN" init --help
    assert_success
    assert_output --partial "init"
}

@test "new --help shows help" {
    run "$WK_BIN" new --help
    assert_success
    assert_output --partial "new"
}

@test "start --help shows help" {
    run "$WK_BIN" start --help
    assert_success
    assert_output --partial "start"
}

@test "done --help shows help" {
    run "$WK_BIN" done --help
    assert_success
    assert_output --partial "done"
}

@test "close --help shows help" {
    run "$WK_BIN" close --help
    assert_success
    assert_output --partial "close"
}

@test "reopen --help shows help" {
    run "$WK_BIN" reopen --help
    assert_success
    assert_output --partial "reopen"
}

@test "edit --help shows help" {
    run "$WK_BIN" edit --help
    assert_success
    assert_output --partial "edit"
}

@test "list --help shows help" {
    run "$WK_BIN" list --help
    assert_success
    assert_output --partial "list"
}

@test "show --help shows help" {
    run "$WK_BIN" show --help
    assert_success
    assert_output --partial "show"
}

@test "tree --help shows help" {
    run "$WK_BIN" tree --help
    assert_success
    assert_output --partial "tree"
}

@test "dep --help shows help" {
    run "$WK_BIN" dep --help
    assert_success
    assert_output --partial "dep"
}

@test "undep --help shows help" {
    run "$WK_BIN" undep --help
    assert_success
    assert_output --partial "undep"
}

@test "label --help shows help" {
    run "$WK_BIN" label --help
    assert_success
    assert_output --partial "label"
}

@test "unlabel --help shows help" {
    run "$WK_BIN" unlabel --help
    assert_success
    assert_output --partial "unlabel"
}

@test "note --help shows help" {
    run "$WK_BIN" note --help
    assert_success
    assert_output --partial "note"
}

@test "log --help shows help" {
    run "$WK_BIN" log --help
    assert_success
    assert_output --partial "log"
}

@test "export --help shows help" {
    run "$WK_BIN" export --help
    assert_success
    assert_output --partial "export"
}

# Test help <cmd> works for all commands

@test "help init shows help" {
    run "$WK_BIN" help init
    assert_success
    assert_output --partial "init"
}

@test "help new shows help" {
    run "$WK_BIN" help new
    assert_success
    assert_output --partial "new"
}

@test "help start shows help" {
    run "$WK_BIN" help start
    assert_success
    assert_output --partial "start"
}

@test "help done shows help" {
    run "$WK_BIN" help done
    assert_success
    assert_output --partial "done"
}

@test "help close shows help" {
    run "$WK_BIN" help close
    assert_success
    assert_output --partial "close"
}

@test "help reopen shows help" {
    run "$WK_BIN" help reopen
    assert_success
    assert_output --partial "reopen"
}

@test "help edit shows help" {
    run "$WK_BIN" help edit
    assert_success
    assert_output --partial "edit"
}

@test "help list shows help" {
    run "$WK_BIN" help list
    assert_success
    assert_output --partial "list"
}

@test "help show shows help" {
    run "$WK_BIN" help show
    assert_success
    assert_output --partial "show"
}

@test "help tree shows help" {
    run "$WK_BIN" help tree
    assert_success
    assert_output --partial "tree"
}

@test "help dep shows help" {
    run "$WK_BIN" help dep
    assert_success
    assert_output --partial "dep"
}

@test "help undep shows help" {
    run "$WK_BIN" help undep
    assert_success
    assert_output --partial "undep"
}

@test "help label shows help" {
    run "$WK_BIN" help label
    assert_success
    assert_output --partial "label"
}

@test "help unlabel shows help" {
    run "$WK_BIN" help unlabel
    assert_success
    assert_output --partial "unlabel"
}

@test "help note shows help" {
    run "$WK_BIN" help note
    assert_success
    assert_output --partial "note"
}

@test "help log shows help" {
    run "$WK_BIN" help log
    assert_success
    assert_output --partial "log"
}

@test "help export shows help" {
    run "$WK_BIN" help export
    assert_success
    assert_output --partial "export"
}

@test "help help shows help" {
    run "$WK_BIN" help help
    assert_success
    assert_output --partial "help"
}
