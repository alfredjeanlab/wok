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

@test "list shows issues and handles empty database" {
    # Empty database
    run "$WK_BIN" list
    assert_success

    # Shows created issues
    create_issue task "Task 1"
    create_issue task "Task 2"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "Task 1"
    assert_output --partial "Task 2"
}

@test "list default shows todo and in_progress, excludes done" {
    id1=$(create_issue task "ListDefault Todo task")
    id2=$(create_issue task "ListDefault Active task")
    id3=$(create_issue task "ListDefault Done task")
    "$WK_BIN" start "$id2"
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    run "$WK_BIN" list
    assert_success
    assert_output --partial "ListDefault Todo task"
    assert_output --partial "ListDefault Active task"
    refute_output --partial "ListDefault Done task"
}

@test "list --status filters by status" {
    id1=$(create_issue task "StatusFilter Todo")
    id2=$(create_issue task "StatusFilter InProgress")
    id3=$(create_issue task "StatusFilter Done")
    "$WK_BIN" start "$id2"
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"

    run "$WK_BIN" list --status todo
    assert_success
    assert_output --partial "StatusFilter Todo"
    refute_output --partial "StatusFilter InProgress"
    refute_output --partial "StatusFilter Done"

    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "StatusFilter InProgress"
    refute_output --partial "StatusFilter Todo"

    run "$WK_BIN" list --status done
    assert_success
    assert_output --partial "StatusFilter Done"
}

@test "list --type filters by issue type" {
    create_issue feature "TypeFilter MyFeature"
    create_issue bug "TypeFilter MyBug"
    create_issue chore "TypeFilter MyChore"
    create_issue task "TypeFilter MyTask"

    run "$WK_BIN" list --type feature
    assert_success
    assert_output --partial "TypeFilter MyFeature"
    refute_output --partial "TypeFilter MyTask"

    run "$WK_BIN" list --type bug
    assert_success
    assert_output --partial "TypeFilter MyBug"

    run "$WK_BIN" list --type chore
    assert_success
    assert_output --partial "TypeFilter MyChore"

    # Short flag -t works
    run "$WK_BIN" list -t task
    assert_success
    assert_output --partial "TypeFilter MyTask"
}

@test "list --label and --blocked filters" {
    create_issue task "LabelFilter Labeled" --label "project:auth"
    create_issue task "LabelFilter Other"
    a=$(create_issue task "BlockFilter Blocker")
    b=$(create_issue task "BlockFilter Blocked")
    "$WK_BIN" dep "$a" blocks "$b"

    # Label filter
    run "$WK_BIN" list --label "project:auth"
    assert_success
    assert_output --partial "LabelFilter Labeled"
    refute_output --partial "LabelFilter Other"

    # Default shows both blocked and unblocked
    run "$WK_BIN" list
    assert_success
    assert_output --partial "BlockFilter Blocker"
    assert_output --partial "BlockFilter Blocked"

    # --blocked shows only blocked
    run "$WK_BIN" list --blocked
    assert_success
    refute_output --partial "BlockFilter Blocker"
    assert_output --partial "BlockFilter Blocked"

    # No blocked count footer
    run "$WK_BIN" list
    refute_output --partial "blocked issues"
}

@test "list combines filters" {
    create_issue feature "Combined Feature" --label "team:alpha"
    create_issue task "Combined Task" --label "team:alpha"
    run "$WK_BIN" list --type feature --label "team:alpha"
    assert_success
    assert_output --partial "Combined Feature"
    refute_output --partial "Combined Task"
}

@test "list --format json outputs valid data with fields" {
    id=$(create_issue task "JSONList Task")
    "$WK_BIN" label "$id" "priority:high"

    run "$WK_BIN" list --format json
    assert_success
    echo "$output" | jq . >/dev/null
    echo "$output" | jq -e '.issues[0].id' >/dev/null
    echo "$output" | jq -e '.issues[0].issue_type' >/dev/null
    echo "$output" | jq -e '.issues[0].status' >/dev/null
    echo "$output" | jq -e '.issues[0].title' >/dev/null
    echo "$output" | jq -e '.issues[0].labels' >/dev/null

    # Labels included
    label=$(echo "$output" | jq -r '.issues[] | select(.title == "JSONList Task") | .labels[0]')
    [ "$label" = "priority:high" ]

    # Short flag -f works
    run "$WK_BIN" list -f json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "list --format json respects filters and has no blocked_count" {
    create_issue task "JSONFilter Task"
    create_issue bug "JSONFilter Bug"
    a=$(create_issue task "JSONBlock Blocker")
    b=$(create_issue task "JSONBlock Blocked")
    "$WK_BIN" dep "$a" blocks "$b"

    # Type filter
    run "$WK_BIN" list --type bug --format json
    assert_success
    all_bugs=$(echo "$output" | jq '[.issues[].issue_type] | all(. == "bug")')
    [ "$all_bugs" = "true" ]

    # No blocked_count
    run "$WK_BIN" list --format json
    result=$(echo "$output" | jq '.blocked_count')
    [ "$result" = "null" ]
}

@test "list sorts by priority ASC then created_at DESC" {
    id1=$(create_issue task "SortList P3 task")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "SortList P1 task")
    "$WK_BIN" label "$id2" "priority:1"

    run "$WK_BIN" list
    assert_success
    first_issue=$(echo "$output" | grep -E '^\- \[' | head -1)
    [[ "$first_issue" == *"SortList P1 task"* ]]

    # Same priority: newer first
    id3=$(create_issue task "SortList Older")
    sleep 0.1
    id4=$(create_issue task "SortList Newer")
    run "$WK_BIN" list
    newer_line=$(echo "$output" | grep -n "SortList Newer" | cut -d: -f1)
    older_line=$(echo "$output" | grep -n "SortList Older" | cut -d: -f1)
    [ "$newer_line" -lt "$older_line" ]
}

@test "list treats missing priority as 2 and prefers priority: over p:" {
    id1=$(create_issue task "PrioList High")
    "$WK_BIN" label "$id1" "priority:1"
    id2=$(create_issue task "PrioList Default")
    id3=$(create_issue task "PrioList Low")
    "$WK_BIN" label "$id3" "priority:3"

    run "$WK_BIN" list
    assert_success
    high_line=$(echo "$output" | grep -n "PrioList High" | cut -d: -f1)
    default_line=$(echo "$output" | grep -n "PrioList Default" | cut -d: -f1)
    low_line=$(echo "$output" | grep -n "PrioList Low" | cut -d: -f1)
    [ "$high_line" -lt "$default_line" ]
    [ "$default_line" -lt "$low_line" ]

    # Prefers priority: over p:
    id4=$(create_issue task "PrefList Dual")
    "$WK_BIN" label "$id4" "p:0"
    "$WK_BIN" label "$id4" "priority:4"
    id5=$(create_issue task "PrefList Default2")
    run "$WK_BIN" list
    dual_line=$(echo "$output" | grep -n "PrefList Dual" | cut -d: -f1)
    default2_line=$(echo "$output" | grep -n "PrefList Default2" | cut -d: -f1)
    [ "$default2_line" -lt "$dual_line" ]
}

@test "list --filter with age filters by time" {
    old_id=$(create_issue task "AgeFilter Old")
    sleep 0.2
    new_id=$(create_issue task "AgeFilter New")

    run "$WK_BIN" list --filter "age < 100ms"
    assert_success
    assert_output --partial "AgeFilter New"
    refute_output --partial "AgeFilter Old"

    run "$WK_BIN" list --filter "age >= 100ms"
    assert_success
    assert_output --partial "AgeFilter Old"
    refute_output --partial "AgeFilter New"

    # Short flag -q works
    run "$WK_BIN" list -q "age < 1h"
    assert_success
}

@test "list --filter validates expressions" {
    run "$WK_BIN" list --filter "invalid < 3d"
    assert_failure
    assert_output --partial "unknown field"

    run "$WK_BIN" list --filter "age << 3d"
    assert_failure
    assert_output --partial "unknown operator"

    run "$WK_BIN" list --filter "age < 3x"
    assert_failure
    assert_output --partial "unknown duration unit"
}

@test "list --filter multiple filters and combined with flags" {
    create_issue task "MultiFilter Task" --label "team:alpha"
    create_issue bug "MultiFilter Bug" --label "team:alpha"

    run "$WK_BIN" list --filter "age < 1h" --filter "updated < 1h"
    assert_success
    assert_output --partial "MultiFilter"

    run "$WK_BIN" list --filter "age < 1h" --type task --label "team:alpha"
    assert_success
    assert_output --partial "MultiFilter Task"
    refute_output --partial "MultiFilter Bug"
}

@test "list --limit truncates results" {
    create_issue task "Limit 1" --label "limit-tag"
    create_issue task "Limit 2" --label "limit-tag"
    create_issue task "Limit 3" --label "limit-tag"

    run "$WK_BIN" list --label "limit-tag" --limit 2
    assert_success
    local count=$(echo "$output" | grep -c "Limit")
    [ "$count" -eq 2 ]

    # Short flag -n works
    run "$WK_BIN" list --label "limit-tag" -n 1
    assert_success
    count=$(echo "$output" | grep -c "Limit")
    [ "$count" -eq 1 ]
}

@test "list --format json includes metadata when filters/limit used" {
    create_issue task "JSONMeta Issue"

    run "$WK_BIN" list --filter "age < 1d" --format json
    assert_success
    local filters=$(echo "$output" | jq '.filters_applied')
    [ "$filters" != "null" ]

    run "$WK_BIN" list --limit 10 --format json
    assert_success
    local limit=$(echo "$output" | jq '.limit')
    [ "$limit" = "10" ]
}

@test "list --filter closed shows closed issues" {
    id=$(create_issue task "ClosedFilter Issue")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"

    # Without filter, done hidden
    run "$WK_BIN" list
    refute_output --partial "ClosedFilter Issue"

    # With closed filter, shown
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedFilter Issue"
}

@test "list --filter closed includes done and closed statuses" {
    done_id=$(create_issue task "ClosedStatus Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    closed_id=$(create_issue task "ClosedStatus Closed")
    "$WK_BIN" close "$closed_id" --reason "duplicate"
    open_id=$(create_issue task "ClosedStatus Open")

    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedStatus Done"
    assert_output --partial "ClosedStatus Closed"
    refute_output --partial "ClosedStatus Open"
}

@test "list --filter closed synonyms work" {
    id=$(create_issue task "ClosedSyn Issue")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"

    for field in "closed" "completed" "done"; do
        run "$WK_BIN" list --filter "$field < 1d"
        assert_success
        assert_output --partial "ClosedSyn Issue"
    done
}
