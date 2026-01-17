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

@test "search requires query argument" {
    run "$WK_BIN" search
    assert_failure
    assert_output --partial "Usage"
}

@test "search finds issues by title" {
    create_issue task "Authentication login"
    create_issue task "Dashboard widget"
    run "$WK_BIN" search "login"
    assert_success
    assert_output --partial "Authentication login"
    refute_output --partial "Dashboard widget"
}

@test "search finds issues by description" {
    id=$(create_issue task "Generic task")
    "$WK_BIN" edit "$id" description "Implement OAuth2 flow"
    run "$WK_BIN" search "OAuth2"
    assert_success
    assert_output --partial "Generic task"
}

@test "search finds issues by note content" {
    id=$(create_issue task "Setup task")
    "$WK_BIN" note "$id" "Configure SSL certificates"
    run "$WK_BIN" search "SSL"
    assert_success
    assert_output --partial "Setup task"
}

@test "search finds issues by label" {
    id=$(create_issue task "Important task")
    "$WK_BIN" label "$id" "priority:high"
    run "$WK_BIN" search "priority:high"
    assert_success
    assert_output --partial "Important task"
}

@test "search finds issues by external link URL" {
    id=$(create_issue task "Linked task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    run "$WK_BIN" search "github.com"
    assert_success
    assert_output --partial "Linked task"
}

@test "search finds issues by external ID" {
    id=$(create_issue task "Jira linked")
    "$WK_BIN" link "$id" "jira://PE-5555"
    run "$WK_BIN" search "PE-5555"
    assert_success
    assert_output --partial "Jira linked"
}

@test "search is case-insensitive" {
    create_issue task "Authentication Module"
    run "$WK_BIN" search "authentication"
    assert_success
    assert_output --partial "Authentication Module"
}

@test "search with no matches shows empty output" {
    create_issue task "Some task"
    run "$WK_BIN" search "nonexistent"
    assert_success
    refute_output --partial "Some task"
}

@test "search respects --status filter" {
    id1=$(create_issue task "Todo task")
    id2=$(create_issue task "Done task")
    "$WK_BIN" start "$id2"
    "$WK_BIN" done "$id2"
    run "$WK_BIN" search "task" --status todo
    assert_success
    assert_output --partial "Todo task"
    refute_output --partial "Done task"
}

@test "search respects --type filter" {
    create_issue bug "Bug with auth"
    create_issue task "Task with auth"
    run "$WK_BIN" search "auth" --type bug
    assert_success
    assert_output --partial "Bug with auth"
    refute_output --partial "Task with auth"
}

@test "search respects --label filter" {
    id1=$(create_issue task "Task A")
    id2=$(create_issue task "Task B")
    "$WK_BIN" label "$id1" "urgent"
    run "$WK_BIN" search "Task" --label urgent
    assert_success
    assert_output --partial "Task A"
    refute_output --partial "Task B"
}

@test "search --format json outputs valid JSON" {
    create_issue task "JSON test task"
    run "$WK_BIN" search "JSON" --format json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "search -f json short flag works" {
    create_issue task "Short flag test"
    run "$WK_BIN" search "Short" -f json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "search help shows examples" {
    run "$WK_BIN" search --help
    assert_success
    assert_output --partial "Examples"
}

@test "search limits results to 25 and shows N more" {
    # Create 30 matching issues
    for i in $(seq 1 30); do
        create_issue task "Limit test item $i"
    done
    run "$WK_BIN" search "Limit test"
    assert_success
    # Should show exactly 25 issues and a "more" message
    local count=$(echo "$output" | grep -c "Limit test item")
    [ "$count" -eq 25 ]
    assert_output --partial "... 5 more"
}

@test "search does not show N more when under limit" {
    # Create only 10 matching issues
    for i in $(seq 1 10); do
        create_issue task "Under limit $i"
    done
    run "$WK_BIN" search "Under limit"
    assert_success
    refute_output --partial "more"
}

@test "search JSON includes more field when results exceed limit" {
    # Create 30 matching issues
    for i in $(seq 1 30); do
        create_issue task "JSON limit test $i"
    done
    run "$WK_BIN" search "JSON limit test" --format json
    assert_success
    # Verify JSON is valid and contains 'more' field
    echo "$output" | jq . >/dev/null
    local more=$(echo "$output" | jq '.more')
    [ "$more" = "5" ]
}

@test "search JSON omits more field when under limit" {
    create_issue task "JSON no more"
    run "$WK_BIN" search "JSON no more" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    # 'more' should be null (not present)
    local more=$(echo "$output" | jq '.more')
    [ "$more" = "null" ]
}

# Time-based filter tests

@test "search --filter with age filters by creation time" {
    old_id=$(create_issue task "SearchFilterTest Old")
    sleep 0.2
    new_id=$(create_issue task "SearchFilterTest New")
    run "$WK_BIN" search "SearchFilterTest" --filter "age < 100ms"
    assert_success
    assert_output --partial "SearchFilterTest New"
    refute_output --partial "SearchFilterTest Old"
}

@test "search -q short flag works" {
    create_issue task "SearchShortFlagTest Issue"
    run "$WK_BIN" search "SearchShortFlagTest" -q "age < 1h"
    assert_success
    assert_output --partial "SearchShortFlagTest Issue"
}

@test "search --filter invalid expression shows error" {
    run "$WK_BIN" search "test" --filter "invalid < 3d"
    assert_failure
    assert_output --partial "unknown field"
}

@test "search --limit overrides default limit" {
    for i in $(seq 1 10); do
        create_issue task "SearchLimitTest $i"
    done
    run "$WK_BIN" search "SearchLimitTest" --limit 3
    assert_success
    local count=$(echo "$output" | grep -c "SearchLimitTest")
    [ "$count" -eq 3 ]
}

@test "search -n short flag for limit works" {
    for i in $(seq 1 5); do
        create_issue task "SearchLimitShortTest $i"
    done
    run "$WK_BIN" search "SearchLimitShortTest" -n 2
    assert_success
    local count=$(echo "$output" | grep -c "SearchLimitShortTest")
    [ "$count" -eq 2 ]
}

@test "search --filter and --limit work together" {
    for i in $(seq 1 5); do
        create_issue task "SearchComboTest $i"
    done
    run "$WK_BIN" search "SearchComboTest" --filter "age < 1h" --limit 2
    assert_success
    local count=$(echo "$output" | grep -c "SearchComboTest")
    [ "$count" -eq 2 ]
}

@test "search --format json includes filters_applied" {
    create_issue task "SearchJSONFilterTest Issue"
    run "$WK_BIN" search "SearchJSONFilterTest" --filter "age < 1d" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local filters=$(echo "$output" | jq '.filters_applied')
    [ "$filters" != "null" ]
}

@test "search --format json includes limit when specified" {
    create_issue task "SearchJSONLimitTest Issue"
    run "$WK_BIN" search "SearchJSONLimitTest" --limit 5 --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local limit=$(echo "$output" | jq '.limit')
    [ "$limit" = "5" ]
}

# Closed filter tests

@test "search --filter closed works with query" {
    id=$(create_issue task "SearchClosedTest Auth feature")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" search "Auth" --filter "closed < 1d"
    assert_success
    assert_output --partial "SearchClosedTest Auth feature"
}

@test "search --filter closed excludes open issues" {
    closed_id=$(create_issue task "SearchClosedExclude Closed")
    "$WK_BIN" start "$closed_id"
    "$WK_BIN" done "$closed_id"
    open_id=$(create_issue task "SearchClosedExclude Open")
    run "$WK_BIN" search "SearchClosedExclude" --filter "closed < 1d"
    assert_success
    assert_output --partial "SearchClosedExclude Closed"
    refute_output --partial "SearchClosedExclude Open"
}
