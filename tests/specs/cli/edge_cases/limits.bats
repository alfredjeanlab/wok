#!/usr/bin/env bats
load '../../helpers/common'

# Input limit tests based on REQUIREMENTS.md:
# - Issue titles: auto-truncated at 120 characters (with ellipsis)
# - Note content: max 10,000 characters
# - Label names: max 100 characters
# - Reason text: max 500 characters

# Title limits (auto-truncation at 120 characters)

@test "new accepts title at 120 character limit without truncation" {
    local title=$(printf 'x%.0s' {1..120})
    id=$(create_issue task "$title")
    # Title should be preserved exactly as given
    run "$WK_BIN" show "$id" --output json
    refute_output --partial '..."'
}

@test "new truncates title exceeding 120 characters" {
    local title=$(printf 'x%.0s' {1..501})
    id=$("$WK_BIN" new "$title" --output id)
    # Title should be truncated with ellipsis, full content in description
    run "$WK_BIN" show "$id" --output json
    assert_output --partial '..."'
}

@test "edit accepts title at 120 character limit without truncation" {
    id=$(create_issue task "Original title")
    local title=$(printf 'x%.0s' {1..120})
    run "$WK_BIN" edit "$id" title "$title"
    assert_success
    # Title should be preserved exactly as given
    run "$WK_BIN" show "$id" --output json
    refute_output --partial '..."'
}

@test "edit rejects title exceeding 120 characters" {
    # Edit rejects long titles because truncation would implicitly change the description
    id=$(create_issue task "Original title")
    local title=$(printf 'x%.0s' {1..121})
    run "$WK_BIN" edit "$id" title "$title"
    assert_failure
    assert_output --partial "double-newline"
}

# Note limits (10,000 characters)

@test "note accepts content at 10000 character limit" {
    id=$(create_issue task "Test task")
    local content=$(printf 'a%.0s' {1..10000})
    run "$WK_BIN" note "$id" "$content"
    assert_success
}

@test "note rejects content exceeding 10000 characters" {
    id=$(create_issue task "Test task")
    local content=$(printf 'a%.0s' {1..10001})
    run "$WK_BIN" note "$id" "$content"
    assert_failure
    assert_output --partial "10000" || assert_output --partial "10,000"
}

@test "new --note accepts content at 10000 character limit" {
    local content=$(printf 'a%.0s' {1..10000})
    run "$WK_BIN" new "Test task" --note "$content"
    assert_success
}

@test "new --note rejects content exceeding 10000 characters" {
    local content=$(printf 'a%.0s' {1..10001})
    run "$WK_BIN" new "Test task" --note "$content"
    assert_failure
    assert_output --partial "10000" || assert_output --partial "10,000"
}

# Label limits (100 characters)

@test "label accepts name at 100 character limit" {
    id=$(create_issue task "Test task")
    local label=$(printf 'a%.0s' {1..100})
    run "$WK_BIN" label "$id" "$label"
    assert_success
}

@test "label rejects name exceeding 100 characters" {
    id=$(create_issue task "Test task")
    local label=$(printf 'a%.0s' {1..101})
    run "$WK_BIN" label "$id" "$label"
    assert_failure
    assert_output --partial "100"
}

@test "new --label accepts name at 100 character limit" {
    local label=$(printf 'a%.0s' {1..100})
    run "$WK_BIN" new "Test task" --label "$label"
    assert_success
}

@test "new --label rejects name exceeding 100 characters" {
    local label=$(printf 'a%.0s' {1..101})
    run "$WK_BIN" new "Test task" --label "$label"
    assert_failure
    assert_output --partial "100"
}

# Reason limits (500 characters)

@test "close accepts reason at 500 character limit" {
    id=$(create_issue task "Test task")
    local reason=$(printf 'a%.0s' {1..500})
    run "$WK_BIN" close "$id" --reason "$reason"
    assert_success
}

@test "close rejects reason exceeding 500 characters" {
    id=$(create_issue task "Test task")
    local reason=$(printf 'a%.0s' {1..501})
    run "$WK_BIN" close "$id" --reason "$reason"
    assert_failure
    assert_output --partial "500"
}

@test "reopen accepts reason at 500 character limit" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    local reason=$(printf 'a%.0s' {1..500})
    run "$WK_BIN" reopen "$id" --reason "$reason"
    assert_success
}

@test "reopen rejects reason exceeding 500 characters" {
    id=$(create_issue task "Test task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    local reason=$(printf 'a%.0s' {1..501})
    run "$WK_BIN" reopen "$id" --reason "$reason"
    assert_failure
    assert_output --partial "500"
}
