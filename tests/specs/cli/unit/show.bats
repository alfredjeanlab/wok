#!/usr/bin/env bats
load '../../helpers/common'

@test "show displays issue details and labels" {
    # Displays issue details
    id=$(create_issue task "ShowBasic Test task")
    run "$WK_BIN" show "$id"
    assert_success
    assert_line --index 0 --partial "[task] $id"
    assert_output --partial "Title: ShowBasic Test task"
    assert_output --partial "Status: todo"
    assert_output --partial "Created:"
    assert_output --partial "Updated:"

    # Displays labels
    id=$(create_issue task "ShowBasic Labeled task" --label "project:auth")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"

    # Displays multiple labels
    id=$(create_issue task "ShowBasic Multi-labeled" --label "label1" --label "label2")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"
}

@test "show displays notes grouped by status" {
    # Displays notes
    id=$(create_issue task "ShowNotes Test task")
    "$WK_BIN" note "$id" "My note content"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "My note content"

    # Groups notes by status
    id=$(create_issue task "ShowNotes Grouped task")
    "$WK_BIN" note "$id" "Todo note"
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "In progress note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Todo note"
    assert_output --partial "In progress note"
}

@test "show displays dependency relationships" {
    # Displays blockers
    a=$(create_issue task "ShowDep Blocker")
    b=$(create_issue task "ShowDep Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" show "$b"
    assert_success
    assert_output --partial "Blocked by"
    assert_output --partial "$a"

    # Displays blocking relationships
    run "$WK_BIN" show "$a"
    assert_success
    assert_output --partial "Blocks"
    assert_output --partial "$b"

    # Displays parent relationship
    feature=$(create_issue feature "ShowDep Parent feature")
    task=$(create_issue task "ShowDep Child task")
    "$WK_BIN" dep "$feature" tracks "$task"
    run "$WK_BIN" show "$task"
    assert_success
    assert_output --partial "Tracked by"
    assert_output --partial "$feature"

    # Displays children
    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"
    assert_output --partial "$task"

    # Displays log
    id=$(create_issue task "ShowDep Log task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Log:"
    assert_output --partial "started"
}

@test "show error handling" {
    # Nonexistent issue fails
    run "$WK_BIN" show "test-nonexistent"
    assert_failure

    # Requires issue ID
    run "$WK_BIN" show
    assert_failure
}

@test "show: multiple issues in text mode separated by ---" {
    id1=$(create_issue task "First issue")
    id2=$(create_issue task "Second issue")
    run "$WK_BIN" show "$id1" "$id2"
    assert_success
    assert_output --partial "First issue"
    assert_output --partial "---"
    assert_output --partial "Second issue"
}

@test "show: multiple issues in json mode outputs JSONL" {
    id1=$(create_issue task "First issue")
    id2=$(create_issue task "Second issue")
    run "$WK_BIN" show "$id1" "$id2" -o json
    assert_success
    # Count lines - should be 2 (one per issue)
    line_count=$(echo "$output" | wc -l | tr -d ' ')
    assert_equal "$line_count" "2"
    # Each line should be valid JSON
    echo "$output" | head -1 | jq . >/dev/null
    echo "$output" | tail -1 | jq . >/dev/null
}

@test "show: single issue json format is compact (JSONL)" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" show "$id" -o json
    assert_success
    # Single line of JSON (compact format)
    line_count=$(echo "$output" | wc -l | tr -d ' ')
    assert_equal "$line_count" "1"
    # Should be valid JSON
    echo "$output" | jq . >/dev/null
}

@test "show: fails if any ID is invalid" {
    id=$(create_issue task "Test issue")
    run "$WK_BIN" show "$id" nonexistent
    assert_failure
    assert_output --partial "not found"
}
