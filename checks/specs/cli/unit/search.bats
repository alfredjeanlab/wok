#!/usr/bin/env bats
load '../../helpers/common'

@test "search requires query and finds by various fields" {
    # Requires query argument
    run "$WK_BIN" search
    assert_failure
    assert_output --partial "Usage"

    # Setup test data
    create_issue task "SearchBasic Authentication login"
    create_issue task "SearchBasic Dashboard widget"
    id=$(create_issue task "SearchBasic Generic task")
    "$WK_BIN" edit "$id" description "Implement OAuth2 flow"
    id2=$(create_issue task "SearchBasic Setup task")
    "$WK_BIN" note "$id2" "Configure SSL certificates"
    id3=$(create_issue task "SearchBasic Important task")
    "$WK_BIN" label "$id3" "priority:high"
    id4=$(create_issue task "SearchBasic Linked task")
    "$WK_BIN" link "$id4" "https://github.com/org/repo/issues/123"
    id5=$(create_issue task "SearchBasic Jira linked")
    "$WK_BIN" link "$id5" "jira://PE-5555"

    # Finds issues by title
    run "$WK_BIN" search "login"
    assert_success
    assert_output --partial "SearchBasic Authentication login"
    refute_output --partial "SearchBasic Dashboard widget"

    # Finds issues by description
    run "$WK_BIN" search "OAuth2"
    assert_success
    assert_output --partial "SearchBasic Generic task"

    # Finds issues by note content
    run "$WK_BIN" search "SSL"
    assert_success
    assert_output --partial "SearchBasic Setup task"

    # Finds issues by label
    run "$WK_BIN" search "priority:high"
    assert_success
    assert_output --partial "SearchBasic Important task"

    # Finds issues by external link URL
    run "$WK_BIN" search "github.com"
    assert_success
    assert_output --partial "SearchBasic Linked task"

    # Finds issues by external ID
    run "$WK_BIN" search "PE-5555"
    assert_success
    assert_output --partial "SearchBasic Jira linked"

    # Search is case-insensitive
    run "$WK_BIN" search "authentication"
    assert_success
    assert_output --partial "SearchBasic Authentication login"
}

@test "search with no matches shows empty output" {
    create_issue task "SearchEmpty Some task"
    run "$WK_BIN" search "nonexistent"
    assert_success
    refute_output --partial "SearchEmpty Some task"
}

@test "search respects --status, --type, --label filters" {
    id1=$(create_issue task "SearchFilter Todo task")
    id2=$(create_issue task "SearchFilter Done task")
    "$WK_BIN" start "$id2"
    "$WK_BIN" done "$id2"
    create_issue bug "SearchFilter Bug with auth"
    create_issue task "SearchFilter Task with auth"
    id3=$(create_issue task "SearchFilter Task A")
    id4=$(create_issue task "SearchFilter Task B")
    "$WK_BIN" label "$id3" "urgent"

    # --status filters
    run "$WK_BIN" search "SearchFilter" --status todo
    assert_success
    assert_output --partial "SearchFilter Todo task"
    refute_output --partial "SearchFilter Done task"

    # --type filters
    run "$WK_BIN" search "auth" --type bug
    assert_success
    assert_output --partial "SearchFilter Bug with auth"
    refute_output --partial "SearchFilter Task with auth"

    # --label filters
    run "$WK_BIN" search "Task" --label urgent
    assert_success
    assert_output --partial "SearchFilter Task A"
    refute_output --partial "SearchFilter Task B"
}

@test "search --format json outputs valid JSON (including short flag)" {
    create_issue task "SearchJSON test task"
    create_issue task "SearchJSON Short flag test"

    run "$WK_BIN" search "SearchJSON" --format json
    assert_success
    echo "$output" | jq . >/dev/null

    # -f short flag works
    run "$WK_BIN" search "SearchJSON Short" -f json
    assert_success
    echo "$output" | jq . >/dev/null
}

@test "search help shows examples" {
    run "$WK_BIN" search --help
    assert_success
    assert_output --partial "Examples"
}

@test "search limits results to 25 and shows N more" {
    for i in $(seq 1 30); do
        create_issue task "SearchLimit test item $i"
    done

    run "$WK_BIN" search "SearchLimit test"
    assert_success
    local count=$(echo "$output" | grep -c "SearchLimit test item")
    [ "$count" -eq 25 ]
    assert_output --partial "... 5 more"

    # JSON includes more field
    run "$WK_BIN" search "SearchLimit test" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local more=$(echo "$output" | jq '.more')
    [ "$more" = "5" ]
}

@test "search does not show N more when under limit" {
    for i in $(seq 1 10); do
        create_issue task "SearchUnderLimit $i"
    done

    run "$WK_BIN" search "SearchUnderLimit"
    assert_success
    refute_output --partial "more"

    # JSON omits more field when under limit
    run "$WK_BIN" search "SearchUnderLimit" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local more=$(echo "$output" | jq '.more')
    [ "$more" = "null" ]
}

@test "search --filter with age and validation" {
    old_id=$(create_issue task "SearchAge Old")
    sleep 0.2
    new_id=$(create_issue task "SearchAge New")

    run "$WK_BIN" search "SearchAge" --filter "age < 100ms"
    assert_success
    assert_output --partial "SearchAge New"
    refute_output --partial "SearchAge Old"

    # -q short flag works
    run "$WK_BIN" search "SearchAge" -q "age < 1h"
    assert_success
    assert_output --partial "SearchAge"

    # Invalid expression shows error
    run "$WK_BIN" search "test" --filter "invalid < 3d"
    assert_failure
    assert_output --partial "unknown field"
}

@test "search --limit overrides default limit" {
    for i in $(seq 1 10); do
        create_issue task "SearchLimitOverride $i"
    done

    run "$WK_BIN" search "SearchLimitOverride" --limit 3
    assert_success
    local count=$(echo "$output" | grep -c "SearchLimitOverride")
    [ "$count" -eq 3 ]

    # -n short flag works
    run "$WK_BIN" search "SearchLimitOverride" -n 2
    assert_success
    count=$(echo "$output" | grep -c "SearchLimitOverride")
    [ "$count" -eq 2 ]
}

@test "search --filter and --limit work together with JSON metadata" {
    for i in $(seq 1 5); do
        create_issue task "SearchCombo $i"
    done

    run "$WK_BIN" search "SearchCombo" --filter "age < 1h" --limit 2
    assert_success
    local count=$(echo "$output" | grep -c "SearchCombo")
    [ "$count" -eq 2 ]

    # JSON includes filters_applied
    run "$WK_BIN" search "SearchCombo" --filter "age < 1d" --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local filters=$(echo "$output" | jq '.filters_applied')
    [ "$filters" != "null" ]

    # JSON includes limit when specified
    run "$WK_BIN" search "SearchCombo" --limit 5 --format json
    assert_success
    echo "$output" | jq . >/dev/null
    local limit=$(echo "$output" | jq '.limit')
    [ "$limit" = "5" ]
}

@test "search --filter closed works with query and excludes open" {
    closed_id=$(create_issue task "SearchClosed Closed")
    "$WK_BIN" start "$closed_id"
    "$WK_BIN" done "$closed_id"
    open_id=$(create_issue task "SearchClosed Open")

    run "$WK_BIN" search "SearchClosed" --filter "closed < 1d"
    assert_success
    assert_output --partial "SearchClosed Closed"
    refute_output --partial "SearchClosed Open"
}
