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

@test "export creates file with valid JSONL format" {
    # Creates file
    create_issue task "ExportBasic Test task"
    run "$WK_BIN" export "export.jsonl"
    assert_success
    [ -f "export.jsonl" ]

    # Produces valid JSONL format
    create_issue task "ExportBasic JSONL task"
    rm -f "export.jsonl"
    "$WK_BIN" export "export.jsonl"
    while IFS= read -r line; do
        echo "$line" | jq . > /dev/null 2>&1 || \
        { echo "Invalid JSON: $line"; return 1; }
    done < "export.jsonl"

    # Empty database produces empty file or no output
    run "$WK_BIN" export "export.jsonl"
    assert_success
}

@test "export includes issue data (title, type, status, labels)" {
    # Includes issue data
    create_issue task "ExportData My test task"
    "$WK_BIN" export "export.jsonl"
    grep -q "My test task" "export.jsonl"

    # Includes all issues
    create_issue task "ExportData Task 1"
    create_issue bug "ExportData Bug 1"
    create_issue feature "ExportData Feature 1"
    "$WK_BIN" export "export.jsonl"
    grep -q "Task 1" "export.jsonl"
    grep -q "Bug 1" "export.jsonl"
    grep -q "Feature 1" "export.jsonl"

    # Includes issue type
    create_issue bug "ExportData Test bug"
    "$WK_BIN" export "export.jsonl"
    grep -q "bug" "export.jsonl"

    # Includes issue status
    id=$(create_issue task "ExportData Status task")
    "$WK_BIN" start "$id"
    "$WK_BIN" export "export.jsonl"
    grep -q "in_progress" "export.jsonl"

    # Includes labels
    id=$(create_issue task "ExportData Labeled task" --label "mylabel")
    "$WK_BIN" export "export.jsonl"
    grep -q "mylabel" "export.jsonl"

    # Overwrites existing file
    echo "old content" > "export.jsonl"
    create_issue task "ExportData New task"
    "$WK_BIN" export "export.jsonl"
    refute grep -q "old content" "export.jsonl"
}

@test "export requires filepath" {
    run "$WK_BIN" export
    assert_failure
}

@test "export accepts various path formats" {
    create_issue task "ExportPath Test task"

    # Absolute path
    run "$WK_BIN" export "/tmp/wk_export_test.jsonl"
    assert_success
    [ -f "/tmp/wk_export_test.jsonl" ]
    rm -f "/tmp/wk_export_test.jsonl"

    # Relative path
    run "$WK_BIN" export "export.jsonl"
    assert_success
    [ -f "export.jsonl" ]

    # Relative path with subdirectory
    mkdir -p subdir
    run "$WK_BIN" export "subdir/export.jsonl"
    assert_success
    [ -f "subdir/export.jsonl" ]

    # Path with ..
    run "$WK_BIN" export "subdir/../export2.jsonl"
    assert_success
    [ -f "export2.jsonl" ]
}
