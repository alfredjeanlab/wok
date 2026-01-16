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

@test "list shows issues" {
    create_issue task "Task 1"
    create_issue task "Task 2"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Task 1"
    assert_output --partial "Task 2"
}

@test "list default shows both todo and in_progress items" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "Active task")
    id3=$(create_issue task "Done task")
    "$WK_BIN" start "$id2"
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Todo task"
    assert_output --partial "Active task"
    refute_output --partial "Done task"
}

@test "list with no issues shows empty or message" {
    run "$WK_BIN" list
    assert_success
}

@test "list --status todo shows only todo items" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "In progress task")
    "$WK_BIN" start "$id2"
    run "$WK_BIN" list --status todo
    assert_success
    assert_output --partial "Todo task"
    refute_output --partial "In progress task"
}

@test "list --status in_progress shows only active items" {
    create_issue task "Todo task"
    id=$(create_issue task "Active task")
    "$WK_BIN" start "$id"
    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "Active task"
    refute_output --partial "Todo task"
}

@test "list --status done shows completed items" {
    id=$(create_issue task "Done task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" list --status done
    assert_success
    assert_output --partial "Done task"
}

@test "list --type feature shows only features" {
    create_issue feature "My feature"
    create_issue task "My task"
    run "$WK_BIN" list --type feature
    assert_success
    assert_output --partial "My feature"
    refute_output --partial "My task"
}

@test "list --type bug shows only bugs" {
    create_issue bug "My bug"
    create_issue task "My task"
    run "$WK_BIN" list --type bug
    assert_success
    assert_output --partial "My bug"
    refute_output --partial "My task"
}

@test "list --type chore shows only chores" {
    create_issue chore "My chore"
    create_issue task "My task"
    run "$WK_BIN" list --type chore
    assert_success
    assert_output --partial "My chore"
    refute_output --partial "My task"
}

@test "list -t feature shows only features (short form)" {
    create_issue feature "My feature"
    create_issue task "My task"
    run "$WK_BIN" list -t feature
    assert_success
    assert_output --partial "My feature"
    refute_output --partial "My task"
}

@test "list --label filters by label" {
    create_issue task "Labeled task" --label "project:auth"
    create_issue task "Other task"
    run "$WK_BIN" list --label "project:auth"
    assert_success
    assert_output --partial "Labeled task"
    refute_output --partial "Other task"
}

@test "list default shows all open issues (blocked and unblocked)" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Blocker"
    assert_output --partial "Blocked"
}

@test "list --blocked shows only blocked issues" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" list --blocked
    assert_success
    refute_output --partial "Blocker"
    assert_output --partial "Blocked"
}

@test "list does not show blocked count footer" {
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" list
    assert_success
    # list no longer shows blocked count footer - use ready to find unblocked items
    refute_output --partial "blocked issues"
}

@test "list combines filters" {
    create_issue feature "Feature with label" --label "team:alpha"
    create_issue task "Task with label" --label "team:alpha"
    run "$WK_BIN" list --type feature --label "team:alpha"
    assert_success
    assert_output --partial "Feature with label"
    refute_output --partial "Task with label"
}

# JSON format tests

@test "list --format json outputs valid JSON" {
    create_issue task "JSON test task"
    run "$WK_BIN" list --format json
    assert_success
    echo "$output" | jq . >/dev/null  # Validates JSON
}

@test "list --format json contains issue fields" {
    create_issue task "JSON test task"
    run "$WK_BIN" list --format json
    assert_success
    echo "$output" | jq -e '.issues[0].id' >/dev/null
    echo "$output" | jq -e '.issues[0].issue_type' >/dev/null
    echo "$output" | jq -e '.issues[0].status' >/dev/null
    echo "$output" | jq -e '.issues[0].title' >/dev/null
    echo "$output" | jq -e '.issues[0].labels' >/dev/null
}

@test "list --format json does not include blocked_count" {
    a=$(create_issue task "Blocker task")
    b=$(create_issue task "Blocked task")
    "$WK_BIN" dep "$a" blocks "$b"
    run "$WK_BIN" list --format json
    assert_success
    # blocked_count is no longer in the output
    result=$(echo "$output" | jq '.blocked_count')
    [ "$result" = "null" ]
}

@test "list --format json respects filters" {
    create_issue task "Task unique filter 123"
    create_issue bug "Bug unique filter 123"
    run "$WK_BIN" list --type bug --format json
    assert_success
    # All issues in the result should be bugs
    all_bugs=$(echo "$output" | jq '[.issues[].issue_type] | all(. == "bug")')
    [ "$all_bugs" = "true" ]
    # And should contain our bug
    echo "$output" | jq -e '.issues[] | select(.title == "Bug unique filter 123")' >/dev/null
}

@test "list -f json short flag works" {
    create_issue task "Short flag test"
    run "$WK_BIN" list -f json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "list --format json with labels" {
    id=$(create_issue task "Labeled JSON task")
    "$WK_BIN" label "$id" "priority:high"
    run "$WK_BIN" list --format json
    assert_success
    label=$(echo "$output" | jq -r '.issues[0].labels[0]')
    [ "$label" = "priority:high" ]
}

# Sort order tests

@test "list sorts by priority ASC" {
    id1=$(create_issue task "P3 task")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "P1 task")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" list
    assert_success
    first_issue=$(echo "$output" | grep -E '^\- \[' | head -1)
    [[ "$first_issue" == *"P1 task"* ]]
}

@test "list same priority sorts by created_at DESC (newest first)" {
    id1=$(create_issue task "SortTest Older task")
    sleep 1
    id2=$(create_issue task "SortTest Newer task")
    # Both have default priority 2
    run "$WK_BIN" list
    assert_success
    # Find SortTest issues and verify newer appears before older
    newer_line=$(echo "$output" | grep -n "SortTest Newer task" | cut -d: -f1)
    older_line=$(echo "$output" | grep -n "SortTest Older task" | cut -d: -f1)
    [ "$newer_line" -lt "$older_line" ]
}

@test "list treats missing priority as 2 (medium)" {
    # Create high priority issue
    id1=$(create_issue task "PrioTest High priority task")
    "$WK_BIN" label "$id1" "priority:1"
    # Create default priority issue (no tag = 2)
    id2=$(create_issue task "PrioTest Default priority task")
    # Create low priority issue
    id3=$(create_issue task "PrioTest Low priority task")
    "$WK_BIN" label "$id3" "priority:3"
    run "$WK_BIN" list
    assert_success
    # Find line numbers for each PrioTest issue
    high_line=$(echo "$output" | grep -n "PrioTest High priority task" | cut -d: -f1)
    default_line=$(echo "$output" | grep -n "PrioTest Default priority task" | cut -d: -f1)
    low_line=$(echo "$output" | grep -n "PrioTest Low priority task" | cut -d: -f1)
    # Order should be: high (1), default (2), low (3)
    [ "$high_line" -lt "$default_line" ]
    [ "$default_line" -lt "$low_line" ]
}

@test "list prefers priority: over p: tag" {
    id=$(create_issue task "PrefTest Dual tagged")
    "$WK_BIN" label "$id" "p:0"
    "$WK_BIN" label "$id" "priority:4"
    id2=$(create_issue task "PrefTest Default priority issue")
    run "$WK_BIN" list
    assert_success
    # Dual tagged (priority:4) should appear after default (priority:2)
    dual_line=$(echo "$output" | grep -n "PrefTest Dual tagged" | cut -d: -f1)
    default_line=$(echo "$output" | grep -n "PrefTest Default priority issue" | cut -d: -f1)
    [ "$default_line" -lt "$dual_line" ]
}

# Time-based filter tests

@test "list --filter with age filters by creation time" {
    old_id=$(create_issue task "FilterTest Old issue")
    sleep 2
    new_id=$(create_issue task "FilterTest New issue")
    # Filter for issues created less than 1 second ago (only new issue)
    run "$WK_BIN" list --filter "age < 1s"
    assert_success
    assert_output --partial "FilterTest New issue"
    refute_output --partial "FilterTest Old issue"
}

@test "list --filter with age >= shows older issues" {
    old_id=$(create_issue task "AgeTest Old issue")
    sleep 2
    new_id=$(create_issue task "AgeTest New issue")
    # Filter for issues at least 1 second old
    run "$WK_BIN" list --filter "age >= 1s"
    assert_success
    assert_output --partial "AgeTest Old issue"
    refute_output --partial "AgeTest New issue"
}

@test "list -q short flag works" {
    create_issue task "ShortFlagTest Task"
    run "$WK_BIN" list -q "age < 1h"
    assert_success
    assert_output --partial "ShortFlagTest Task"
}

@test "list --filter multiple filters combine with AND" {
    id=$(create_issue task "MultiFilterTest Issue")
    run "$WK_BIN" list --filter "age < 1h" --filter "updated < 1h"
    assert_success
    assert_output --partial "MultiFilterTest Issue"
}

@test "list --filter invalid expression shows error" {
    run "$WK_BIN" list --filter "invalid < 3d"
    assert_failure
    assert_output --partial "unknown field"
}

@test "list --filter invalid operator shows error" {
    run "$WK_BIN" list --filter "age << 3d"
    assert_failure
    assert_output --partial "unknown operator"
}

@test "list --filter invalid duration unit shows error" {
    run "$WK_BIN" list --filter "age < 3x"
    assert_failure
    assert_output --partial "unknown duration unit"
}

@test "list --limit truncates results" {
    create_issue task "LimitTest 1" --label "limit-test-tag"
    create_issue task "LimitTest 2" --label "limit-test-tag"
    create_issue task "LimitTest 3" --label "limit-test-tag"
    run "$WK_BIN" list --label "limit-test-tag" --limit 2
    assert_success
    local count=$(echo "$output" | grep -c "LimitTest")
    [ "$count" -eq 2 ]
}

@test "list -n short flag for limit works" {
    create_issue task "LimitShortTest 1" --label "limit-short-tag"
    create_issue task "LimitShortTest 2" --label "limit-short-tag"
    create_issue task "LimitShortTest 3" --label "limit-short-tag"
    run "$WK_BIN" list --label "limit-short-tag" -n 1
    assert_success
    local count=$(echo "$output" | grep -c "LimitShortTest")
    [ "$count" -eq 1 ]
}

@test "list --format json includes filters_applied when filter used" {
    create_issue task "JSONFilterTest Issue"
    run "$WK_BIN" list --filter "age < 1d" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local filters=$(echo "$output" | jq '.filters_applied')
    [ "$filters" != "null" ]
}

@test "list --format json includes limit when used" {
    create_issue task "JSONLimitTest Issue"
    run "$WK_BIN" list --limit 10 --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local limit=$(echo "$output" | jq '.limit')
    [ "$limit" = "10" ]
}

@test "list --filter combined with other flags" {
    create_issue task "CombinedTest Task" --label "team:alpha"
    create_issue bug "CombinedTest Bug" --label "team:alpha"
    run "$WK_BIN" list --filter "age < 1h" --type task --label "team:alpha"
    assert_success
    assert_output --partial "CombinedTest Task"
    refute_output --partial "CombinedTest Bug"
}

# Closed filter tests

@test "list --filter closed shows recently closed issues" {
    id=$(create_issue task "ClosedFilterTest Closeable issue")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedFilterTest Closeable issue"
}

@test "list --filter closed excludes open issues" {
    closed_id=$(create_issue task "ClosedFilterTest Closed issue")
    "$WK_BIN" start "$closed_id"
    "$WK_BIN" done "$closed_id"
    open_id=$(create_issue task "ClosedFilterTest Open issue")
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedFilterTest Closed issue"
    refute_output --partial "ClosedFilterTest Open issue"
}

@test "list --filter closed includes done and closed statuses" {
    done_id=$(create_issue task "ClosedFilterTest Done issue")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    closed_id=$(create_issue task "ClosedFilterTest Closed issue")
    "$WK_BIN" close "$closed_id" --reason "duplicate"
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedFilterTest Done issue"
    assert_output --partial "ClosedFilterTest Closed issue"
}

@test "list --filter closed synonyms work" {
    id=$(create_issue task "ClosedSynonymTest Issue")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    # Test all synonyms
    for field in "closed" "completed" "done"; do
        run "$WK_BIN" list --filter "$field < 1d"
        assert_success
        assert_output --partial "ClosedSynonymTest Issue"
    done
}

@test "list --filter closed without --all shows closed issues" {
    # By default, list excludes done/closed without --all
    # But closed filter should include them automatically
    id=$(create_issue task "ClosedAutoIncludeTest Issue")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    # Without closed filter, done issues are hidden
    run "$WK_BIN" list
    assert_success
    refute_output --partial "ClosedAutoIncludeTest Issue"
    # With closed filter, done issues are included
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedAutoIncludeTest Issue"
}
