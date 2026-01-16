#!/usr/bin/env bats
load '../../helpers/common'

# Tests verifying consistent short flags across commands
# - --type / -t consistency across list, edit, import
# - --status / -s consistency across list, import
# - --label / -l consistency across new, list, import
# - --format / -f consistency across show, import

setup_file() {
    file_setup
    init_project_once test
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

# --type / -t consistency
# All commands with --type should also accept -t

@test "list accepts -t as short form for --type" {
    create_issue bug "My bug"
    create_issue task "My task"
    run "$WK_BIN" list -t bug
    assert_success
    assert_output --partial "My bug"
    refute_output --partial "My task"
}

@test "edit accepts -t as short form for --type" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" edit "$id" -t bug
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[bug]"
}

@test "import accepts -t as short form for --type" {
    create_issue bug "Imported bug"
    "$WK_BIN" export "test_export.jsonl"

    # Re-init in clean directory
    local tmpdir
    tmpdir="$(mktemp -d)"
    cd "$tmpdir" || exit 1
    export HOME="$tmpdir"
    "$WK_BIN" init --prefix imp

    run "$WK_BIN" import -t bug "$BATS_FILE_TMPDIR/test_export.jsonl"
    assert_success
    rm -rf "$tmpdir"
}

# --status / -s consistency

@test "list accepts -s as short form for --status" {
    id=$(create_issue task "Started task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" list -s in_progress
    assert_success
    assert_output --partial "Started task"
}

@test "import accepts -s as short form for --status" {
    create_issue task "Test"
    "$WK_BIN" export "test.jsonl"
    run "$WK_BIN" import --dry-run -s todo "test.jsonl"
    assert_success
}

# --label / -l consistency

@test "new accepts -l as short form for --label" {
    id=$("$WK_BIN" new task "Labeled task" -l mylabel | grep -oE '[a-z]+-[a-z0-9]+' | head -1)
    run "$WK_BIN" show "$id"
    assert_output --partial "mylabel"
}

@test "list accepts -l as short form for --label" {
    create_issue task "Labeled" --label findme
    run "$WK_BIN" list -l findme
    assert_success
    assert_output --partial "Labeled"
}

@test "import accepts -l as short form for --label" {
    create_issue task "Test" --label importlabel
    "$WK_BIN" export "test.jsonl"
    run "$WK_BIN" import --dry-run -l importlabel "test.jsonl"
    assert_success
}

# --format / -f consistency

@test "show accepts -f as short form for --format" {
    id=$(create_issue task "Test")
    run "$WK_BIN" show "$id" -f json
    assert_success
    assert_output --partial "{"
}

@test "import accepts -f as short form for --format" {
    create_issue task "Test"
    "$WK_BIN" export "test.jsonl"
    run "$WK_BIN" import --dry-run -f wk "test.jsonl"
    assert_success
}
