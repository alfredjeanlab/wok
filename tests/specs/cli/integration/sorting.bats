#!/usr/bin/env bats
load '../../helpers/common'

# Per-test isolation - each test gets its own project to avoid parallel interference
setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
    init_project "test"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

# Ready command sort tests

@test "ready: recent high-priority before recent low-priority" {
    id1=$(create_issue task "Low priority recent")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "High priority recent")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" ready
    assert_success
    # High priority should appear first
    first_line=$(echo "$output" | grep -E '^\- \[' | head -1)
    [[ "$first_line" == *"High priority recent"* ]]
}

@test "ready: priority:N preferred over p:N" {
    id=$(create_issue task "Dual tagged issue")
    "$WK_BIN" label "$id" "p:0"
    "$WK_BIN" label "$id" "priority:4"
    # Create reference issue with default priority
    id2=$(create_issue task "Default priority issue")
    run "$WK_BIN" ready
    assert_success
    # Dual tagged (priority:4) should appear after default (priority:2)
    # Check relative ordering - default should appear before dual-tagged
    pos_default=$(echo "$output" | grep -n "Default priority issue" | head -1 | cut -d: -f1)
    pos_dual=$(echo "$output" | grep -n "Dual tagged issue" | head -1 | cut -d: -f1)
    [ "$pos_default" -lt "$pos_dual" ]
}

@test "ready: named priority values work" {
    id1=$(create_issue task "Highest priority")
    "$WK_BIN" label "$id1" "priority:highest"
    id2=$(create_issue task "Lowest priority")
    "$WK_BIN" label "$id2" "priority:lowest"
    run "$WK_BIN" ready
    assert_success
    first_line=$(echo "$output" | grep -E '^\- \[' | head -1)
    [[ "$first_line" == *"Highest priority"* ]]
}

@test "ready: default priority is 2 (medium)" {
    # Create high priority issue
    id1=$(create_issue task "High priority task")
    "$WK_BIN" label "$id1" "priority:1"
    # Create default priority issue (no tag = 2)
    id2=$(create_issue task "Default priority task")
    # Create low priority issue
    id3=$(create_issue task "Low priority task")
    "$WK_BIN" label "$id3" "priority:3"
    run "$WK_BIN" ready
    assert_success
    # Check relative ordering: high (1) < default (2) < low (3)
    pos_high=$(echo "$output" | grep -n "High priority task" | head -1 | cut -d: -f1)
    pos_default=$(echo "$output" | grep -n "Default priority task" | head -1 | cut -d: -f1)
    pos_low=$(echo "$output" | grep -n "Low priority task" | head -1 | cut -d: -f1)
    [ "$pos_high" -lt "$pos_default" ]
    [ "$pos_default" -lt "$pos_low" ]
}

# List command sort tests

@test "list: sorts by priority ASC" {
    id1=$(create_issue task "P3 list task")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "P1 list task")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" list
    assert_success
    # Check relative ordering - P1 should appear before P3
    pos_p1=$(echo "$output" | grep -n "P1 list task" | head -1 | cut -d: -f1)
    pos_p3=$(echo "$output" | grep -n "P3 list task" | head -1 | cut -d: -f1)
    [ "$pos_p1" -lt "$pos_p3" ]
}

@test "list: same priority sorts by created_at DESC (newest first)" {
    id1=$(create_issue task "Older list task")
    sleep 0.1
    id2=$(create_issue task "Newer list task")
    # Both have default priority 2
    run "$WK_BIN" list
    assert_success
    # Check relative ordering - newer should appear before older
    pos_newer=$(echo "$output" | grep -n "Newer list task" | head -1 | cut -d: -f1)
    pos_older=$(echo "$output" | grep -n "Older list task" | head -1 | cut -d: -f1)
    [ "$pos_newer" -lt "$pos_older" ]
}

@test "list: priority:N preferred over p:N" {
    id=$(create_issue task "Dual tagged list")
    "$WK_BIN" label "$id" "p:0"
    "$WK_BIN" label "$id" "priority:4"
    id2=$(create_issue task "Default list issue")
    run "$WK_BIN" list
    assert_success
    # Dual tagged (priority:4) should appear after default (priority:2)
    pos_default=$(echo "$output" | grep -n "Default list issue" | head -1 | cut -d: -f1)
    pos_dual=$(echo "$output" | grep -n "Dual tagged list" | head -1 | cut -d: -f1)
    [ "$pos_default" -lt "$pos_dual" ]
}

# JSON format tests

@test "ready: json format respects priority sorting" {
    id1=$(create_issue task "Low priority json")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "High priority json")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" ready --output json
    assert_success
    # High priority should appear before low priority in JSON array
    pos_high=$(echo "$output" | jq -r '.issues | to_entries | .[] | select(.value.id == "'"$id2"'") | .key')
    pos_low=$(echo "$output" | jq -r '.issues | to_entries | .[] | select(.value.id == "'"$id1"'") | .key')
    [ "$pos_high" -lt "$pos_low" ]
}

@test "list: json format respects priority sorting" {
    id1=$(create_issue task "Low priority list json")
    "$WK_BIN" label "$id1" "priority:3"
    id2=$(create_issue task "High priority list json")
    "$WK_BIN" label "$id2" "priority:1"
    run "$WK_BIN" list --output json
    assert_success
    # High priority should appear before low priority in JSON array
    pos_high=$(echo "$output" | jq -r '.issues | to_entries | .[] | select(.value.id == "'"$id2"'") | .key')
    pos_low=$(echo "$output" | jq -r '.issues | to_entries | .[] | select(.value.id == "'"$id1"'") | .key')
    [ "$pos_high" -lt "$pos_low" ]
}
