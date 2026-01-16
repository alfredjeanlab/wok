#!/usr/bin/env bats
load '../../helpers/common'

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

@test "export creates file" {
    create_issue task "Test task"
    run "$WK_BIN" export "export.jsonl"
    assert_success
    [ -f "export.jsonl" ]
}

@test "export produces JSONL format" {
    create_issue task "JSONLTest task"
    rm -f "export.jsonl"
    "$WK_BIN" export "export.jsonl"
    # Each line should be valid JSON
    while IFS= read -r line; do
        echo "$line" | jq . > /dev/null 2>&1 || \
        { echo "Invalid JSON: $line"; return 1; }
    done < "export.jsonl"
}

@test "export includes issue data" {
    create_issue task "My test task"
    "$WK_BIN" export "export.jsonl"
    grep -q "My test task" "export.jsonl"
}

@test "export includes all issues" {
    create_issue task "Task 1"
    create_issue bug "Bug 1"
    create_issue feature "Feature 1"
    "$WK_BIN" export "export.jsonl"
    grep -q "Task 1" "export.jsonl"
    grep -q "Bug 1" "export.jsonl"
    grep -q "Feature 1" "export.jsonl"
}

@test "export includes issue type" {
    create_issue bug "Test bug"
    "$WK_BIN" export "export.jsonl"
    grep -q "bug" "export.jsonl"
}

@test "export includes issue status" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" export "export.jsonl"
    grep -q "in_progress" "export.jsonl"
}

@test "export empty database produces empty file or no output" {
    run "$WK_BIN" export "export.jsonl"
    assert_success
}

@test "export requires filepath" {
    run "$WK_BIN" export
    assert_failure
}

@test "export overwrites existing file" {
    echo "old content" > "export.jsonl"
    create_issue task "New task"
    "$WK_BIN" export "export.jsonl"
    refute grep -q "old content" "export.jsonl"
}

@test "export includes labels" {
    id=$(create_issue task "Labeled task" --label "mylabel")
    "$WK_BIN" export "export.jsonl"
    grep -q "mylabel" "export.jsonl"
}

# Export path validation: any valid filesystem path works

@test "export accepts absolute path" {
    create_issue task "Test task"
    run "$WK_BIN" export "/tmp/wk_export_test.jsonl"
    assert_success
    [ -f "/tmp/wk_export_test.jsonl" ]
    rm -f "/tmp/wk_export_test.jsonl"
}

@test "export accepts relative path" {
    create_issue task "Test task"
    run "$WK_BIN" export "export.jsonl"
    assert_success
    [ -f "export.jsonl" ]
}

@test "export accepts relative path with subdirectory" {
    mkdir -p subdir
    create_issue task "Test task"
    run "$WK_BIN" export "subdir/export.jsonl"
    assert_success
    [ -f "subdir/export.jsonl" ]
}

@test "export accepts path with .." {
    mkdir -p subdir
    create_issue task "Test task"
    run "$WK_BIN" export "subdir/../export.jsonl"
    assert_success
    [ -f "export.jsonl" ]
}
