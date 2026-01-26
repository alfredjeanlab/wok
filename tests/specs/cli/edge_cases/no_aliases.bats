#!/usr/bin/env bats
load '../../helpers/common'


# Tests verifying only canonical command names work - no aliases or unsupported commands.
@test "common aliases do not exist" {
    local aliases=(
        "create:Test task"    # use: new
        "ls"                  # use: list
        "add:Test task"       # use: new
        "view:test-x"         # use: show
        "get:test-x"          # use: show
        "begin:test-x"        # use: start
        "finish:test-x"       # use: done
        "complete:test-x"     # use: done
        "modify:test-x"       # use: edit
        "update:test-x"       # use: edit
        "comment:test-x:note" # use: note
        "history"             # use: log
        "events"              # use: log
        "backup:file.jsonl"   # use: export
        "dump:file.jsonl"     # use: export
    )
    for alias_args in "${aliases[@]}"; do
        IFS=':' read -r cmd args <<< "$alias_args"
        run $WK_BIN $cmd $args
        assert_failure
    done
}

@test "aliases requiring issues do not exist" {
    id=$(create_issue task "Test")
    local aliases=(rm del link unlink)
    for cmd in "${aliases[@]}"; do
        run "$WK_BIN" "$cmd" "$id"
        assert_failure
    done
}

@test "unsupported commands fail" {
    local commands=(
        "delete:test-abc"
        "rm:test-abc"
        "remove:test-abc"
        "status"
        "add:Test task"
        "create:Test task"
        "open:test-abc"
        "update:test-abc"
        "get:test-abc"
        "push"
        "pull"
        "version"
        "info"
        "find:test"
        "archive:test-abc"
    )
    for cmd_args in "${commands[@]}"; do
        IFS=':' read -r cmd args <<< "$cmd_args"
        run $WK_BIN $cmd $args
        assert_failure
    done
}

@test "help does not mention unsupported commands" {
    run "$WK_BIN" help
    assert_success
    refute_output --partial "delete"
    refute_output --partial "create"
    refute_output --partial "update"
    refute_output --partial "push"
    refute_output --partial "pull"
    refute_output --partial "archive"
    # These need regex to avoid matching flags/substrings
    refute_output --regexp "^\s+add\s"
    refute_output --regexp "^\s+remove\s"  # "remove" appears in descriptions, only check as command
    refute_output --regexp "^\s+status\s"
    refute_output --regexp " rm "
    # "version" can appear in --version flag but not as a subcommand
    refute_output --regexp "^\s+version\s"
}

@test "help <unsupported> fails" {
    local commands=(delete add create status version)
    for cmd in "${commands[@]}"; do
        run "$WK_BIN" help "$cmd"
        assert_failure
    done
}

@test "canonical commands work" {
    # new
    run "$WK_BIN" new "Test task"
    assert_success

    # list
    run "$WK_BIN" list
    assert_success

    # show, start, done, edit, tree
    id=$(create_issue task "Lifecycle test")
    run "$WK_BIN" show "$id"
    assert_success
    run "$WK_BIN" tree "$id"
    assert_success
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" done "$id"
    assert_success

    # reopen (from done)
    run "$WK_BIN" reopen "$id" --reason "reopened"
    assert_success

    # close
    id2=$(create_issue task "Close test")
    run "$WK_BIN" close "$id2" --reason "closed"
    assert_success

    # edit
    run "$WK_BIN" edit "$id" title "New title"
    assert_success

    # dep, undep
    id3=$(create_issue task "A")
    id4=$(create_issue task "B")
    run "$WK_BIN" dep "$id3" blocks "$id4"
    assert_success
    run "$WK_BIN" undep "$id3" blocks "$id4"
    assert_success

    # label, unlabel
    run "$WK_BIN" label "$id" "mylabel"
    assert_success
    run "$WK_BIN" unlabel "$id" "mylabel"
    assert_success

    # note
    run "$WK_BIN" note "$id" "My note"
    assert_success

    # log
    run "$WK_BIN" log
    assert_success

    # export
    run "$WK_BIN" export "$BATS_FILE_TMPDIR/export.jsonl"
    assert_success
}
