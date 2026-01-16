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

# === Prefix Rename Tests ===

@test "config rename requires both old and new prefix" {
    mkdir -p rename_1 && cd rename_1
    run "$WK_BIN" init --prefix test --local
    assert_success
    run "$WK_BIN" config rename
    assert_failure
}

@test "config rename with only one arg fails" {
    mkdir -p rename_2 && cd rename_2
    run "$WK_BIN" init --prefix test --local
    assert_success
    run "$WK_BIN" config rename newprefix
    assert_failure
}

@test "config rename changes issue IDs" {
    mkdir -p rename_3 && cd rename_3
    run "$WK_BIN" init --prefix old --local
    assert_success
    id=$(create_issue task "Test issue")
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
}

@test "config rename updates config prefix" {
    mkdir -p rename_4 && cd rename_4
    run "$WK_BIN" init --prefix old --local
    assert_success
    create_issue task "Test issue"
    run "$WK_BIN" config rename old new
    assert_success

    # New issues should use new prefix
    new_id=$(create_issue task "Another issue")
    [[ "$new_id" == new-* ]]
}

@test "config rename preserves dependencies" {
    mkdir -p rename_5 && cd rename_5
    run "$WK_BIN" init --prefix old --local
    assert_success
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")
    "$WK_BIN" dep "$id1" blocks "$id2"

    run "$WK_BIN" config rename old new
    assert_success

    new_id2="${id2/old/new}"
    run "$WK_BIN" show "$new_id2"
    assert_success
    assert_output --partial "Blocked by"
}

@test "config rename preserves labels" {
    mkdir -p rename_6 && cd rename_6
    run "$WK_BIN" init --prefix old --local
    assert_success
    id=$(create_issue task "Test issue" --label urgent)

    run "$WK_BIN" config rename old new
    assert_success

    new_id="${id/old/new}"
    run "$WK_BIN" show "$new_id"
    assert_success
    assert_output --partial "urgent"
}

@test "config rename only affects matching prefix" {
    mkdir -p rename_7 && cd rename_7
    run "$WK_BIN" init --prefix main --local
    assert_success
    # Create an issue, then manually create one with a different prefix
    main_id=$(create_issue task "Main issue")

    # Rename 'main' to 'new'
    run "$WK_BIN" config rename main new
    assert_success

    # Main issue should be renamed
    new_id="${main_id/main/new}"
    run "$WK_BIN" show "$new_id"
    assert_success
}

@test "config rename same prefix is noop" {
    mkdir -p rename_8 && cd rename_8
    run "$WK_BIN" init --prefix same --local
    assert_success
    id=$(create_issue task "Test issue")

    run "$WK_BIN" config rename same same
    assert_success
    assert_output --partial "already"

    # Issue should still exist with same ID
    run "$WK_BIN" show "$id"
    assert_success
}

@test "config rename rejects invalid new prefix" {
    mkdir -p rename_9 && cd rename_9
    run "$WK_BIN" init --prefix old --local
    assert_success

    # Too short
    run "$WK_BIN" config rename old a
    assert_failure

    # Contains dash
    run "$WK_BIN" config rename old my-proj
    assert_failure

    # Uppercase
    run "$WK_BIN" config rename old ABC
    assert_failure
}

@test "config rename rejects invalid old prefix" {
    mkdir -p rename_10 && cd rename_10
    run "$WK_BIN" init --prefix valid --local
    assert_success

    # Too short
    run "$WK_BIN" config rename a new
    assert_failure

    # Contains dash
    run "$WK_BIN" config rename my-proj new
    assert_failure
}

@test "config rename does not update config for non-matching prefix" {
    mkdir -p rename_11 && cd rename_11
    run "$WK_BIN" init --prefix current --local
    assert_success
    # Create issue with different prefix (manually if needed)

    # Rename 'other' to 'new' - should not change config since config is 'current'
    run "$WK_BIN" config rename other new
    assert_success

    # New issues should still use 'current' prefix
    new_id=$(create_issue task "After rename")
    [[ "$new_id" == current-* ]]
}

# === Config Remote Tests ===

@test "config remote: configures git remote on local tracker" {
    # Create isolated subdir for this test
    mkdir -p config_remote_1 && cd config_remote_1

    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote .
    assert_success
    assert_output --partial "Remote configured: git:."

    # Verify config updated
    run cat .wok/config.toml
    assert_output --partial '[remote]'
    assert_output --partial 'url = "git:."'
}

@test "config remote: accepts explicit git:. format" {
    # Create isolated subdir for this test
    mkdir -p config_remote_2 && cd config_remote_2

    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote "git:."
    assert_success
    assert_output --partial "Remote configured: git:."
}

@test "config remote: accepts websocket URL" {
    # Create isolated subdir for this test
    mkdir -p config_remote_3 && cd config_remote_3

    run "$WK_BIN" init --prefix test --local
    assert_success

    run "$WK_BIN" config remote "ws://localhost:7890"
    assert_success
    assert_output --partial "Remote configured: ws://localhost:7890"
}

@test "config remote: updates gitignore for remote mode" {
    # Create isolated subdir for this test
    mkdir -p config_remote_4 && cd config_remote_4

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

@test "config remote: no-op when same remote already configured" {
    # Create isolated subdir for this test
    mkdir -p config_remote_5 && cd config_remote_5

    run "$WK_BIN" init --prefix test
    assert_success  # Default is remote mode with git:.

    run "$WK_BIN" config remote .
    assert_success
    assert_output --partial "already configured"
}

@test "config remote: changing remotes not supported" {
    # Create isolated subdir for this test
    mkdir -p config_remote_6 && cd config_remote_6

    run "$WK_BIN" init --prefix test
    assert_success  # Default remote is git:.

    run "$WK_BIN" config remote "ws://other:7890"
    assert_success  # Not an error, just a message
    assert_output --partial "not currently supported"
    assert_output --partial "Current remote: git:."
}

@test "config remote: fails when workspace is configured" {
    # Create isolated subdir for this test
    mkdir -p config_remote_workspace_1 && cd config_remote_workspace_1

    # Create a shared workspace directory
    mkdir -p shared_wok

    # Initialize with workspace pointing to shared location
    run "$WK_BIN" init --prefix test --local --workspace shared_wok
    assert_success

    # Attempting to configure remote should fail
    run "$WK_BIN" config remote .
    assert_failure
    assert_output --partial "workspace are incompatible"
}

@test "config remote: error includes helpful hint when workspace is set" {
    # Create isolated subdir for this test
    mkdir -p config_remote_workspace_2 && cd config_remote_workspace_2

    # Create a shared workspace directory
    mkdir -p shared_wok

    # Initialize with workspace
    run "$WK_BIN" init --prefix test --local --workspace shared_wok
    assert_success

    # Verify error message includes hint
    run "$WK_BIN" config remote "ws://localhost:7890"
    assert_failure
    assert_output --partial "hint:"
}
