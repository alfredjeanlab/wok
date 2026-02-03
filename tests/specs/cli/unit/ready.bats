#!/usr/bin/env bats
load '../../helpers/common'

@test "ready shows unblocked todo issues and excludes blocked" {
    create_issue task "Ready task"
    a=$(create_issue task "Blocker")
    b=$(create_issue task "Blocked issue")
    "$WK_BIN" dep "$a" blocks "$b"

    run "$WK_BIN" ready
    assert_success
    assert_output --partial "Ready task"
    assert_output --partial "Blocker"
    refute_output --partial "Blocked issue"
}

@test "ready with label filter" {
    id=$(create_issue task "Labeled task")
    "$WK_BIN" label "$id" "priority:high"
    create_issue task "Unlabeled task"

    # With matching label
    run "$WK_BIN" ready --label "priority:high"
    assert_success
    assert_output --partial "Labeled task"
    refute_output --partial "Unlabeled task"

    # With non-matching label
    run "$WK_BIN" ready --label "nonexistent-label-xyz123"
    assert_success
    assert_output --partial "No ready issues"
}

@test "ready --output json outputs valid data with expected fields" {
    id=$(create_issue task "JSON ready task")
    "$WK_BIN" label "$id" "module:api"

    # Test --output json
    run "$WK_BIN" ready --output json
    assert_success
    echo "$output" | jq . >/dev/null
    echo "$output" | jq -e '.issues[0].id' >/dev/null
    echo "$output" | jq -e '.issues[0].issue_type' >/dev/null
    echo "$output" | jq -e '.issues[0].status' >/dev/null
    echo "$output" | jq -e '.issues[0].title' >/dev/null
    echo "$output" | jq -e '.issues[0].labels' >/dev/null

    # Verify labels included
    label=$(echo "$output" | jq -r '.issues[] | select(.title == "JSON ready task") | .labels[0]')
    [ "$label" = "module:api" ]

    # Test -o json short flag
    run "$WK_BIN" ready -o json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "ready --output json excludes blocked and respects filters" {
    a=$(create_issue task "Blocker JSON ready")
    "$WK_BIN" label "$a" "test:json-blocked"
    b=$(create_issue task "Blocked JSON ready")
    "$WK_BIN" label "$b" "test:json-blocked"
    "$WK_BIN" dep "$a" blocks "$b"
    id=$(create_issue task "Labeled ready")
    "$WK_BIN" label "$id" "team:backend"

    # Blocked issues excluded (filter by test label to avoid 5-issue limit interference)
    run "$WK_BIN" ready --label "test:json-blocked" --output json
    assert_success
    blocked_present=$(echo "$output" | jq --arg b "$b" '[.issues[].id] | contains([$b])')
    [ "$blocked_present" = "false" ]
    blocker_present=$(echo "$output" | jq --arg a "$a" '[.issues[].id] | contains([$a])')
    [ "$blocker_present" = "true" ]

    # Label filter works
    run "$WK_BIN" ready --label "team:backend" --output json
    assert_success
    count=$(echo "$output" | jq '.issues | length')
    [ "$count" -eq 1 ]
    title=$(echo "$output" | jq -r '.issues[0].title')
    [ "$title" = "Labeled ready" ]

    # Non-matching label returns empty
    run "$WK_BIN" ready --label "nonexistent-label-abc456" --output json
    assert_success
    count=$(echo "$output" | jq '.issues | length')
    [ "$count" -eq 0 ]
}

# Sort order tests

@test "ready sorts recent high-priority before recent low-priority" {
    id1=$(create_issue task "ReadySort Low priority recent")
    "$WK_BIN" label "$id1" "priority:3"
    "$WK_BIN" label "$id1" "test:sort-recent"
    id2=$(create_issue task "ReadySort High priority recent")
    "$WK_BIN" label "$id2" "priority:1"
    "$WK_BIN" label "$id2" "test:sort-recent"
    run "$WK_BIN" ready --label "test:sort-recent"
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
    "$WK_BIN" label "$id" "test:pref-priority"
    id2=$(create_issue task "ReadyPref Default priority issue")
    "$WK_BIN" label "$id2" "test:pref-priority"
    run "$WK_BIN" ready --label "test:pref-priority"
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
    "$WK_BIN" label "$id1" "test:miss-priority"
    # Create default priority issue (no tag = 2)
    id2=$(create_issue task "ReadyMiss Default priority task")
    "$WK_BIN" label "$id2" "test:miss-priority"
    # Create low priority issue
    id3=$(create_issue task "ReadyMiss Low priority task")
    "$WK_BIN" label "$id3" "priority:3"
    "$WK_BIN" label "$id3" "test:miss-priority"
    run "$WK_BIN" ready --label "test:miss-priority"
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
    "$WK_BIN" label "$id1" "test:named-priority"
    id2=$(create_issue task "ReadyNamed Lowest priority")
    "$WK_BIN" label "$id2" "priority:lowest"
    "$WK_BIN" label "$id2" "test:named-priority"
    run "$WK_BIN" ready --label "test:named-priority"
    assert_success
    # Highest should appear before lowest
    highest_line=$(echo "$output" | grep -n "ReadyNamed Highest priority" | cut -d: -f1)
    lowest_line=$(echo "$output" | grep -n "ReadyNamed Lowest priority" | cut -d: -f1)
    [ "$highest_line" -lt "$lowest_line" ]
}

@test "ready returns at most 5 issues" {
    # Create 8 ready issues
    for i in {1..8}; do
        create_issue task "ReadyLimit Issue $i"
    done

    run "$WK_BIN" ready
    assert_success
    # Count issues shown (lines starting with "- [")
    local count=$(echo "$output" | grep -c "^\- \[")
    [ "$count" -le 5 ]

    # JSON also respects limit
    run "$WK_BIN" ready --output json
    assert_success
    count=$(echo "$output" | jq '.issues | length')
    [ "$count" -le 5 ]
}

@test "ready shows hint when more issues exist" {
    # Create 8 ready issues (more than the 5 limit)
    for i in {1..8}; do
        id=$(create_issue task "ReadyHint Issue $i")
        "$WK_BIN" label "$id" "test:hint-more"
    done

    run "$WK_BIN" ready --label "test:hint-more"
    assert_success
    assert_output --partial "3 more"
    assert_output --partial "wk list"
}

@test "ready does not show hint when all issues fit" {
    # Create 3 ready issues (fewer than 5 limit)
    for i in {1..3}; do
        id=$(create_issue task "ReadyNoHint Issue $i")
        "$WK_BIN" label "$id" "test:hint-none"
    done

    run "$WK_BIN" ready --label "test:hint-none"
    assert_success
    refute_output --partial "more"
}

@test "ready filters by prefix" {
    id1=$(create_issue task "PrefixReady Alpha task")
    id2=$("$WK_BIN" new task "PrefixReady Beta task" --prefix beta | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    prefix1="${id1%%-*}"

    run "$WK_BIN" ready -p "$prefix1"
    assert_success
    assert_output --partial "PrefixReady Alpha task"
    refute_output --partial "PrefixReady Beta task"

    run "$WK_BIN" ready --prefix beta
    assert_success
    assert_output --partial "PrefixReady Beta task"
    refute_output --partial "PrefixReady Alpha task"
}
