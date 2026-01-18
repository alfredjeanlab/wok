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

@test "config rename requires both old and new prefix" {
    mkdir -p rename_args && cd rename_args
    run "$WK_BIN" init --prefix test --local
    assert_success

    # Requires both prefixes
    run "$WK_BIN" config rename
    assert_failure

    # Only one arg fails
    run "$WK_BIN" config rename newprefix
    assert_failure
}

@test "config rename changes issue IDs and updates config prefix" {
    mkdir -p rename_main && cd rename_main
    run "$WK_BIN" init --prefix old --local
    assert_success
    id=$(create_issue task "ConfigRename Test issue")

    run "$WK_BIN" config rename old new
    assert_success

    # Old ID should not exist
    run "$WK_BIN" show "$id"
    assert_failure

    # New ID should exist
    new_id="${id/old/new}"
    run "$WK_BIN" show "$new_id"
    assert_success
    assert_output --partial "Test issue"

    # New issues should use new prefix
    new_id=$(create_issue task "ConfigRename Another issue")
    [[ "$new_id" == new-* ]]
}

@test "config rename preserves dependencies and labels" {
    # Preserves dependencies
    mkdir -p rename_deps && cd rename_deps
    run "$WK_BIN" init --prefix old --local
    assert_success
    id1=$(create_issue task "ConfigRenameDeps Blocker")
    id2=$(create_issue task "ConfigRenameDeps Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"

    run "$WK_BIN" config rename old new
    assert_success

    new_id2="${id2/old/new}"
    run "$WK_BIN" show "$new_id2"
    assert_success
    assert_output --partial "Blocked by"

    # Preserves labels
    cd ..
    mkdir -p rename_labels && cd rename_labels
    run "$WK_BIN" init --prefix old --local
    assert_success
    id=$(create_issue task "ConfigRenameLabels Test issue" --label urgent)

    run "$WK_BIN" config rename old new
    assert_success

    new_id="${id/old/new}"
    run "$WK_BIN" show "$new_id"
    assert_success
    assert_output --partial "urgent"
}

@test "config rename only affects matching prefix and same prefix is noop" {
    # Only affects matching prefix
    mkdir -p rename_match && cd rename_match
    run "$WK_BIN" init --prefix main --local
    assert_success
    main_id=$(create_issue task "ConfigRenameMatch Main issue")

    run "$WK_BIN" config rename main new
    assert_success

    new_id="${main_id/main/new}"
    run "$WK_BIN" show "$new_id"
    assert_success

    # Same prefix is noop
    cd ..
    mkdir -p rename_same && cd rename_same
    run "$WK_BIN" init --prefix same --local
    assert_success
    id=$(create_issue task "ConfigRenameSame Test issue")

    run "$WK_BIN" config rename same same
    assert_success
    assert_output --partial "already"

    run "$WK_BIN" show "$id"
    assert_success

    # Does not update config for non-matching prefix
    cd ..
    mkdir -p rename_nomatch && cd rename_nomatch
    run "$WK_BIN" init --prefix current --local
    assert_success

    run "$WK_BIN" config rename other new
    assert_success

    new_id=$(create_issue task "ConfigRenameNoMatch After rename")
    [[ "$new_id" == current-* ]]
}

@test "config rename rejects invalid prefixes" {
    mkdir -p rename_invalid && cd rename_invalid
    run "$WK_BIN" init --prefix old --local
    assert_success

    # New prefix too short
    run "$WK_BIN" config rename old a
    assert_failure

    # New prefix contains dash
    run "$WK_BIN" config rename old my-proj
    assert_failure

    # New prefix uppercase
    run "$WK_BIN" config rename old ABC
    assert_failure

    # Old prefix too short
    run "$WK_BIN" config rename a new
    assert_failure

    # Old prefix contains dash
    run "$WK_BIN" config rename my-proj new
    assert_failure
}

@test "config remote configures git and websocket remotes" {
    # Git remote on local tracker
    mkdir -p remote_git && cd remote_git
    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote .
    assert_success
    assert_output --partial "Remote configured: git:."

    run cat .wok/config.toml
    assert_output --partial '[remote]'
    assert_output --partial 'url = "git:."'

    # Explicit git:. format
    cd ..
    mkdir -p remote_gitexplicit && cd remote_gitexplicit
    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote "git:."
    assert_success
    assert_output --partial "Remote configured: git:."

    # Websocket URL
    cd ..
    mkdir -p remote_ws && cd remote_ws
    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote "ws://localhost:7890"
    assert_success
    assert_output --partial "Remote configured: ws://localhost:7890"

    # Updates gitignore for remote mode
    cd ..
    mkdir -p remote_gitignore && cd remote_gitignore
    run "$WK_BIN" init --prefix test --local
    assert_success

    # Verify local mode gitignore includes config.toml
    run grep "config.toml" .wok/.gitignore
    assert_success

    "$WK_BIN" config remote .

    # Verify remote mode gitignore does NOT include config.toml
    run grep "config.toml" .wok/.gitignore
    assert_failure
}

@test "config remote handles already-configured and changing remotes" {
    # No-op when same remote already configured
    mkdir -p remote_noop && cd remote_noop
    run "$WK_BIN" init --prefix test
    assert_success  # Default is remote mode with git:.

    run "$WK_BIN" config remote .
    assert_success
    assert_output --partial "already configured"

    # Changing remotes not supported
    run "$WK_BIN" config remote "ws://other:7890"
    assert_success  # Not an error, just a message
    assert_output --partial "not currently supported"
    assert_output --partial "Current remote: git:."
}

@test "config remote fails when workspace is configured" {
    # Fails with workspace
    mkdir -p remote_workspace && cd remote_workspace
    mkdir -p shared_wok
    run "$WK_BIN" init --prefix test --local --workspace shared_wok
    assert_success

    run "$WK_BIN" config remote .
    assert_failure
    assert_output --partial "workspace are incompatible"

    # Error includes helpful hint
    mkdir -p shared_wok2
    cd ..
    mkdir -p remote_workspace2 && cd remote_workspace2
    mkdir -p shared_wok
    run "$WK_BIN" init --prefix test --local --workspace shared_wok
    assert_success

    run "$WK_BIN" config remote "ws://localhost:7890"
    assert_failure
    assert_output --partial "hint:"
}
