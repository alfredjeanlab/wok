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

@test "ready shows unblocked todo issues" {
    create_issue task "Ready task"
    run "$WK_BIN" ready
    assert_success
    assert_output --partial "Ready task"
}

@test "ready excludes blocked issues" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked issue")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" ready
    assert_success
    assert_output --partial "Blocker"
    refute_output --partial "Blocked issue"
}

@test "ready shows no ready issues message when no label matches" {
    # Use a label that doesn't exist to get "no ready issues"
    run "$WK_BIN" ready --label "nonexistent-label-xyz123"
    assert_success
    assert_output --partial "No ready issues"
}

@test "ready with label filter" {
    id=$(create_issue task "Labeled task")
    "$WK_BIN" label "$id" "priority:high"
    create_issue task "Unlabeled task"
    run "$WK_BIN" ready --label "priority:high"
    assert_success
    assert_output --partial "Labeled task"
    refute_output --partial "Unlabeled task"
}

# JSON format tests

@test "ready --format json outputs valid JSON" {
    create_issue task "JSON ready task"
    run "$WK_BIN" ready --format json
    assert_success
    echo "$output" | jq . >/dev/null  # Validates JSON
}

@test "ready --format json contains issue fields" {
    create_issue task "JSON ready task"
    run "$WK_BIN" ready --format json
    assert_success
    echo "$output" | jq -e '.issues[0].id' >/dev/null
    echo "$output" | jq -e '.issues[0].issue_type' >/dev/null
    echo "$output" | jq -e '.issues[0].status' >/dev/null
    echo "$output" | jq -e '.issues[0].title' >/dev/null
    echo "$output" | jq -e '.issues[0].labels' >/dev/null
}

@test "ready --format json excludes blocked issues" {
    a=$(create_issue task "Blocker JSON ready")
    b=$(create_issue task "Blocked JSON ready")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" ready --format json
    assert_success
    # The blocked issue should not appear in the results
    blocked_present=$(echo "$output" | jq --arg b "$b" '[.issues[].id] | contains([$b])')
    [ "$blocked_present" = "false" ]
    # The blocker should appear
    blocker_present=$(echo "$output" | jq --arg a "$a" '[.issues[].id] | contains([$a])')
    [ "$blocker_present" = "true" ]
}

@test "ready --format json returns empty array when no label matches" {
    # Use a label that doesn't exist to get empty array
    run "$WK_BIN" ready --label "nonexistent-label-abc456" --format json
    assert_success
    count=$(echo "$output" | jq '.issues | length')
    [ "$count" -eq 0 ]
}

@test "ready --format json with label filter" {
    id=$(create_issue task "Labeled ready")
    "$WK_BIN" label "$id" "team:backend"
    create_issue task "Unlabeled ready"
    run "$WK_BIN" ready --label "team:backend" --format json
    assert_success
    count=$(echo "$output" | jq '.issues | length')
    [ "$count" -eq 1 ]
    title=$(echo "$output" | jq -r '.issues[0].title')
    [ "$title" = "Labeled ready" ]
}

@test "ready -f json short flag works" {
    create_issue task "Short flag ready"
    run "$WK_BIN" ready -f json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "ready --format json with labels" {
    id=$(create_issue task "JsonLabelReady task")
    "$WK_BIN" label "$id" "module:api"
    run "$WK_BIN" ready --format json
    assert_success
    # Find our specific issue and check its label
    label=$(echo "$output" | jq -r '.issues[] | select(.title == "JsonLabelReady task") | .labels[0]')
    [ "$label" = "module:api" ]
}

# Sort order tests

@test "ready sorts recent high-priority before recent low-priority" {
    id1=$(create_issue task "ReadySort Low priority recent")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "ReadySort High priority recent")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" ready
    assert_success
    # High priority should appear before low priority
    high_line=$(echo "$output" | grep -n "ReadySort High priority recent" | cut -d: -f1)
    low_line=$(echo "$output" | grep -n "ReadySort Low priority recent" | cut -d: -f1)
    [ "$high_line" -lt "$low_line" ]
}

@test "ready uses priority:N tag for priority" {
    id=$(create_issue task "P0 task")
    "$WK_BIN" label "$id" "priority:0"
    id2=$(create_issue task "Default task")
    run "$WK_BIN" ready
    assert_success
    # P0 should appear before default (priority 2)
    first_issue=$(echo "$output" | grep -E '^\- \[' | head -1)
    [[ "$first_issue" == *"P0 task"* ]]
}

@test "ready prefers priority: over p: tag" {
    id=$(create_issue task "ReadyPref Dual tagged")
    "$WK_BIN" label "$id" "p:0"
    "$WK_BIN" label "$id" "priority:4"
    id2=$(create_issue task "ReadyPref Default priority issue")
    run "$WK_BIN" ready
    assert_success
    # Dual tagged (priority:4) should appear after default (priority:2)
    dual_line=$(echo "$output" | grep -n "ReadyPref Dual tagged" | cut -d: -f1)
    default_line=$(echo "$output" | grep -n "ReadyPref Default priority issue" | cut -d: -f1)
    [ "$default_line" -lt "$dual_line" ]
}

@test "ready treats missing priority as 2 (medium)" {
    # Create high priority issue
    id1=$(create_issue task "ReadyMiss High priority task")
    "$WK_BIN" label "$id1" "priority:1"
    # Create default priority issue (no tag = 2)
    id2=$(create_issue task "ReadyMiss Default priority task")
    # Create low priority issue
    id3=$(create_issue task "ReadyMiss Low priority task")
    "$WK_BIN" label "$id3" "priority:3"
    run "$WK_BIN" ready
    assert_success
    # Find line numbers for each ReadyMiss issue
    high_line=$(echo "$output" | grep -n "ReadyMiss High priority task" | cut -d: -f1)
    default_line=$(echo "$output" | grep -n "ReadyMiss Default priority task" | cut -d: -f1)
    low_line=$(echo "$output" | grep -n "ReadyMiss Low priority task" | cut -d: -f1)
    # Order should be: high (1), default (2), low (3)
    [ "$high_line" -lt "$default_line" ]
    [ "$default_line" -lt "$low_line" ]
}

@test "ready named priority values work" {
    id1=$(create_issue task "ReadyNamed Highest priority")
    "$WK_BIN" label "$id1" "priority:highest"
    id2=$(create_issue task "ReadyNamed Lowest priority")
    "$WK_BIN" label "$id2" "priority:lowest"
    run "$WK_BIN" ready
    assert_success
    # Highest should appear before lowest
    highest_line=$(echo "$output" | grep -n "ReadyNamed Highest priority" | cut -d: -f1)
    lowest_line=$(echo "$output" | grep -n "ReadyNamed Lowest priority" | cut -d: -f1)
    [ "$highest_line" -lt "$lowest_line" ]
}
