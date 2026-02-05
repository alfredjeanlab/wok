#!/usr/bin/env bats
load '../../helpers/common'

@test "list shows issues with status filtering" {
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

    # Default shows todo and in_progress, excludes done
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

    # --status filters by status
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

@test "list filters by type, label, and blocked" {
    # --type filters by issue type
    create_issue feature "TypeFilter MyFeature"
    create_issue bug "TypeFilter MyBug"
    create_issue chore "TypeFilter MyChore"
    create_issue task "TypeFilter MyTask"
    create_issue idea "TypeFilter MyIdea"

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

    run "$WK_BIN" list --type idea
    assert_success
    assert_output --partial "TypeFilter MyIdea"
    refute_output --partial "TypeFilter MyTask"

    # Short flag -t works
    run "$WK_BIN" list -t task
    assert_success
    assert_output --partial "TypeFilter MyTask"

    # --label and --blocked filters
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

    # Combines filters
    create_issue feature "Combined Feature" --label "team:alpha"
    create_issue task "Combined Task" --label "team:alpha"
    run "$WK_BIN" list --type feature --label "team:alpha"
    assert_success
    assert_output --partial "Combined Feature"
    refute_output --partial "Combined Task"
}

@test "list --output json outputs valid data" {
    id=$(create_issue task "JSONList Task")
    "$WK_BIN" label "$id" "priority:high"

    run "$WK_BIN" list --output json
    assert_success
    echo "$output" | jq . >/dev/null
    echo "$output" | jq -e '.[0].id' >/dev/null
    echo "$output" | jq -e '.[0].issue_type' >/dev/null
    echo "$output" | jq -e '.[0].status' >/dev/null
    echo "$output" | jq -e '.[0].title' >/dev/null
    echo "$output" | jq -e '.[0].labels' >/dev/null

    # Labels included
    label=$(echo "$output" | jq -r '.[] | select(.title == "JSONList Task") | .labels[0]')
    [ "$label" = "priority:high" ]

    # Short flag -f works
    run "$WK_BIN" list -o json
    assert_success
    echo "$output" | jq . >/dev/null

    # Respects filters
    create_issue task "JSONFilter Task"
    create_issue bug "JSONFilter Bug"
    a=$(create_issue task "JSONBlock Blocker")
    b=$(create_issue task "JSONBlock Blocked")
    "$WK_BIN" dep "$a" blocks "$b"

    # Type filter
    run "$WK_BIN" list --type bug --output json
    assert_success
    all_bugs=$(echo "$output" | jq '[.[].issue_type] | all(. == "bug")')
    [ "$all_bugs" = "true" ]

    # Output is a plain array, no wrapper object
    run "$WK_BIN" list --output json
    echo "$output" | jq -e 'type == "array"' >/dev/null
}

@test "list sorts by priority" {
    # Sorts by priority ASC then created_at DESC
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

    # Treats missing priority as 2
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

@test "list --filter expressions" {
    # Age filter
    # Use generous timing margins to avoid flakiness under high load
    old_id=$(create_issue task "AgeFilter Old")
    sleep 0.5
    new_id=$(create_issue task "AgeFilter New")

    run "$WK_BIN" list --filter "age < 400ms"
    assert_success
    assert_output --partial "AgeFilter New"
    refute_output --partial "AgeFilter Old"

    run "$WK_BIN" list --filter "age >= 400ms"
    assert_success
    assert_output --partial "AgeFilter Old"
    refute_output --partial "AgeFilter New"

    # Short flag -q works
    run "$WK_BIN" list -q "age < 1h"
    assert_success

    # Validates expressions
    run "$WK_BIN" list --filter "invalid < 3d"
    assert_failure
    assert_output --partial "unknown filter field"

    run "$WK_BIN" list --filter "age << 3d"
    assert_failure
    assert_output --partial "invalid filter operator"

    run "$WK_BIN" list --filter "age < 3x"
    assert_failure
    assert_output --partial "invalid duration"

    # Multiple filters and combined with flags
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

@test "list --limit and --filter closed" {
    # --limit truncates results
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

    # JSON output is a plain array even with filters/limit
    create_issue task "JSONMeta Issue"

    run "$WK_BIN" list --filter "age < 1d" --output json
    assert_success
    echo "$output" | jq -e 'type == "array"' >/dev/null

    run "$WK_BIN" list --limit 10 --output json
    assert_success
    echo "$output" | jq -e 'type == "array"' >/dev/null

    # --filter closed shows closed issues
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

    # Closed includes done and closed statuses
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

@test "list --filter completed only shows done status" {
    # Create issues with different terminal states
    done_id=$(create_issue task "CompletedFilter Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    closed_id=$(create_issue task "CompletedFilter Cancelled")
    "$WK_BIN" close "$closed_id" --reason "wontfix"
    open_id=$(create_issue task "CompletedFilter Open")

    # completed filter should only match Status::Done
    run "$WK_BIN" list --filter "completed < 1d"
    assert_success
    assert_output --partial "CompletedFilter Done"
    refute_output --partial "CompletedFilter Cancelled"
    refute_output --partial "CompletedFilter Open"

    # done is a synonym for completed
    run "$WK_BIN" list --filter "done < 1d"
    assert_success
    assert_output --partial "CompletedFilter Done"
    refute_output --partial "CompletedFilter Cancelled"
}

@test "list --filter skipped only shows closed status" {
    # Create issues with different terminal states
    done_id=$(create_issue task "SkippedFilter Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    closed_id=$(create_issue task "SkippedFilter Cancelled")
    "$WK_BIN" close "$closed_id" --reason "wontfix"
    open_id=$(create_issue task "SkippedFilter Open")

    # skipped filter should only match Status::Closed
    run "$WK_BIN" list --filter "skipped < 1d"
    assert_success
    assert_output --partial "SkippedFilter Cancelled"
    refute_output --partial "SkippedFilter Done"
    refute_output --partial "SkippedFilter Open"

    # cancelled is a synonym for skipped
    run "$WK_BIN" list --filter "cancelled < 1d"
    assert_success
    assert_output --partial "SkippedFilter Cancelled"
    refute_output --partial "SkippedFilter Done"
}

@test "list --filter closed shows both done and closed status" {
    # Create issues with different terminal states
    done_id=$(create_issue task "ClosedFilter Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    closed_id=$(create_issue task "ClosedFilter Cancelled")
    "$WK_BIN" close "$closed_id" --reason "wontfix"
    open_id=$(create_issue task "ClosedFilter Open")

    # closed filter should match both Status::Done and Status::Closed
    run "$WK_BIN" list --filter "closed < 1d"
    assert_success
    assert_output --partial "ClosedFilter Done"
    assert_output --partial "ClosedFilter Cancelled"
    refute_output --partial "ClosedFilter Open"
}

@test "list --filter with word operators (shell-friendly)" {
    # Word operators are shell-friendly alternatives to < > = etc.
    # Use generous timing margins to avoid flakiness under high load
    old_id=$(create_issue task "WordOp Old")
    sleep 0.5
    new_id=$(create_issue task "WordOp New")

    # lt = less than (<)
    run "$WK_BIN" list --filter "age lt 400ms"
    assert_success
    assert_output --partial "WordOp New"
    refute_output --partial "WordOp Old"

    # gte = greater than or equal (>=)
    run "$WK_BIN" list --filter "age gte 400ms"
    assert_success
    assert_output --partial "WordOp Old"
    refute_output --partial "WordOp New"

    # gt = greater than (>)
    run "$WK_BIN" list --filter "age gt 300ms"
    assert_success
    assert_output --partial "WordOp Old"

    # lte = less than or equal (<=)
    run "$WK_BIN" list --filter "age lte 1d"
    assert_success
    assert_output --partial "WordOp New"
    assert_output --partial "WordOp Old"

    # Case insensitive
    run "$WK_BIN" list --filter "age LT 1d"
    assert_success

    run "$WK_BIN" list --filter "age GT 0ms"
    assert_success
}

@test "list defaults to 100 results" {
    # Create more than 100 issues
    for i in {1..105}; do
        create_issue task "DefaultLimit Issue $i"
    done

    # Default list should return at most 100
    run "$WK_BIN" list
    assert_success
    local count=$(echo "$output" | grep -c "^\- \[")
    [ "$count" -le 100 ]
}

@test "list --limit 0 shows all results (unlimited)" {
    # Create 15 issues with a unique label (enough to prove unlimited works)
    for i in {1..15}; do
        create_issue task "UnlimitedTest Issue $i" --label "test:unlimited"
    done

    # --limit 0 should show all issues
    run "$WK_BIN" list --label "test:unlimited" --limit 0
    assert_success
    local count=$(echo "$output" | grep -c "^\- \[")
    [ "$count" -eq 15 ]
}

@test "list explicit limit overrides default" {
    # Create 50 issues with a unique label
    for i in {1..50}; do
        create_issue task "ExplicitLimit Issue $i" --label "test:explicit"
    done

    # --limit 20 should return exactly 20
    run "$WK_BIN" list --label "test:explicit" --limit 20
    assert_success
    local count=$(echo "$output" | grep -c "^\- \[")
    [ "$count" -eq 20 ]
}

@test "list --output ids outputs space-separated IDs" {
    id1=$(create_issue task "IDFormat Issue 1")
    id2=$(create_issue task "IDFormat Issue 2")
    run "$WK_BIN" list --output ids
    assert_success
    assert_output --partial "$id1"
    assert_output --partial "$id2"
    # Verify no other content (no type, status, or title)
    [[ ! "$output" =~ "task" ]]
    [[ ! "$output" =~ "todo" ]]
    [[ ! "$output" =~ "IDFormat" ]]
    # Verify output is a single line with space-separated IDs
    local line_count=$(echo "$output" | wc -l | tr -d ' ')
    [ "$line_count" -eq 1 ]
}

@test "list --output ids works with filters" {
    id=$(create_issue task "FilterID Task")
    create_issue bug "FilterID Bug"
    run "$WK_BIN" list --type task --output ids
    assert_success
    assert_output --partial "$id"
    [[ ! "$output" =~ "FilterID Bug" ]]
}

@test "list --output ids respects limit" {
    for i in {1..15}; do
        create_issue task "LimitID Issue $i" --label "test:limit-ids"
    done
    run "$WK_BIN" list --label "test:limit-ids" --output ids --limit 10
    assert_success
    # Count space-separated words (IDs)
    local count=$(echo "$output" | wc -w | tr -d ' ')
    [ "$count" -eq 10 ]
}

@test "list -o ids works as short flag" {
    id=$(create_issue task "ShortFlagID Issue")
    run "$WK_BIN" list -o ids
    assert_success
    assert_output --partial "$id"
}

@test "list --output ids can be piped to other commands" {
    id=$(create_issue task "Pipe Test Issue")
    # Verify output is clean for command substitution
    run "$WK_BIN" list --output ids
    assert_success
    # Output should be space-separated IDs (alphanumeric with hyphens)
    for word in $output; do
        [[ "$word" =~ ^[a-z0-9-]+$ ]]
    done
}

@test "list --output ids composes with batch commands" {
    # Create issues with a unique label for isolation
    id1=$(create_issue task "BatchClose Issue 1" --label "test:batch-close")
    id2=$(create_issue task "BatchClose Issue 2" --label "test:batch-close")

    # Verify we can use command substitution to close multiple issues
    # shellcheck disable=SC2046
    run "$WK_BIN" close $("$WK_BIN" list --label "test:batch-close" --output ids) --reason "batch closed"
    assert_success
    assert_output --partial "Closed $id1"
    assert_output --partial "Closed $id2"

    # Verify both issues are now closed
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: closed"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: closed"
}

@test "list --filter accepts 'now' as value" {
    # Create and close an issue
    id=$(create_issue task "NowFilter Issue")
    "$WK_BIN" close "$id" --reason "test"

    # closed < now should match (closed before current time)
    run "$WK_BIN" list --filter "closed < now"
    assert_success
    assert_output --partial "NowFilter Issue"

    # closed > now should not match (nothing closed in the future)
    run "$WK_BIN" list --filter "closed > now"
    assert_success
    refute_output --partial "NowFilter Issue"
}

@test "list --filter accepts bare status fields" {
    # Create issues with different states
    open_id=$(create_issue task "BareFilter Open")
    done_id=$(create_issue task "BareFilter Done")
    "$WK_BIN" start "$done_id"
    "$WK_BIN" done "$done_id"
    skipped_id=$(create_issue task "BareFilter Skipped")
    "$WK_BIN" close "$skipped_id" --reason "wontfix"

    # Bare "closed" matches any terminal state
    run "$WK_BIN" list --filter "closed"
    assert_success
    assert_output --partial "BareFilter Done"
    assert_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Open"

    # Bare "completed" matches only Status::Done
    run "$WK_BIN" list --filter "completed"
    assert_success
    assert_output --partial "BareFilter Done"
    refute_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Open"

    # Bare "skipped" matches only Status::Closed
    run "$WK_BIN" list --filter "skipped"
    assert_success
    assert_output --partial "BareFilter Skipped"
    refute_output --partial "BareFilter Done"
    refute_output --partial "BareFilter Open"
}

@test "list --filter bare fields work with aliases" {
    id=$(create_issue task "AliasFilter Done")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"

    # "done" is alias for "completed"
    run "$WK_BIN" list --filter "done"
    assert_success
    assert_output --partial "AliasFilter Done"

    # "cancelled" is alias for "skipped"
    skipped_id=$(create_issue task "AliasFilter Skipped")
    "$WK_BIN" close "$skipped_id" --reason "test"

    run "$WK_BIN" list --filter "cancelled"
    assert_success
    assert_output --partial "AliasFilter Skipped"
    refute_output --partial "AliasFilter Done"
}

@test "list --filter rejects bare non-status fields" {
    # Bare "age" without operator should fail
    run "$WK_BIN" list --filter "age"
    assert_failure
    assert_output --partial "requires operator"

    # Bare "updated" without operator should fail
    run "$WK_BIN" list --filter "updated"
    assert_failure
    assert_output --partial "requires operator"
}

@test "list filters by prefix" {
    id1=$(create_issue task "PrefixFilter Alpha task")
    # Create an issue with a different prefix
    id2=$("$WK_BIN" new task "PrefixFilter Beta task" --prefix beta | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    prefix1="${id1%%-*}"  # extract prefix from first issue

    run "$WK_BIN" list -p "$prefix1"
    assert_success
    assert_output --partial "PrefixFilter Alpha task"
    refute_output --partial "PrefixFilter Beta task"

    run "$WK_BIN" list -p beta
    assert_success
    assert_output --partial "PrefixFilter Beta task"
    refute_output --partial "PrefixFilter Alpha task"

    run "$WK_BIN" list --prefix "$prefix1"
    assert_success
    assert_output --partial "PrefixFilter Alpha task"
}

@test "list auto-filters by configured project prefix" {
    id1=$(create_issue task "AutoPrefix Own task")
    # Create an issue with a different prefix in the same DB
    id2=$("$WK_BIN" new task "AutoPrefix Other task" --prefix beta | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1)

    # Without -p flag, should only show issues matching configured prefix
    run "$WK_BIN" list
    assert_success
    assert_output --partial "AutoPrefix Own task"
    refute_output --partial "AutoPrefix Other task"

    # Explicit -p should override: show only beta issues
    run "$WK_BIN" list -p beta
    assert_success
    assert_output --partial "AutoPrefix Other task"
    refute_output --partial "AutoPrefix Own task"
}

@test "list --label supports negation with ! prefix" {
    # Create issues with different labels
    create_issue task "NegLabel Has wontfix" --label "wontfix"
    create_issue task "NegLabel Has bug" --label "bug"
    create_issue task "NegLabel No labels"

    # Positive filter: only issues with wontfix
    run "$WK_BIN" list --label "wontfix"
    assert_success
    assert_output --partial "NegLabel Has wontfix"
    refute_output --partial "NegLabel Has bug"
    refute_output --partial "NegLabel No labels"

    # Negative filter: exclude issues with wontfix
    run "$WK_BIN" list --label '!wontfix'
    assert_success
    refute_output --partial "NegLabel Has wontfix"
    assert_output --partial "NegLabel Has bug"
    assert_output --partial "NegLabel No labels"

    # Mixed: has bug OR lacks wontfix (comma = OR)
    run "$WK_BIN" list --label 'bug,!wontfix'
    assert_success
    assert_output --partial "NegLabel Has bug"
    assert_output --partial "NegLabel No labels"
    # Has wontfix but no bug, but matches '!wontfix' is false, 'bug' is false → no match... wait
    # Actually: has wontfix has the label 'wontfix', so '!wontfix' is false, 'bug' is false → no match
    refute_output --partial "NegLabel Has wontfix"

    # Multiple flags: (lacks wontfix) AND (lacks bug)
    run "$WK_BIN" list --label '!wontfix' --label '!bug'
    assert_success
    refute_output --partial "NegLabel Has wontfix"
    refute_output --partial "NegLabel Has bug"
    assert_output --partial "NegLabel No labels"
}

@test "list --label negation works with namespaced labels" {
    # Create issues with namespaced labels
    create_issue task "NSLabel Plan needed" --label "plan:needed"
    create_issue task "NSLabel Plan ready" --label "plan:ready"
    create_issue task "NSLabel No plan"

    # Exclude issues needing planning
    run "$WK_BIN" list --label '!plan:needed'
    assert_success
    refute_output --partial "NSLabel Plan needed"
    assert_output --partial "NSLabel Plan ready"
    assert_output --partial "NSLabel No plan"

    # Combine: has plan:ready AND lacks plan:needed
    run "$WK_BIN" list --label 'plan:ready' --label '!plan:needed'
    assert_success
    refute_output --partial "NSLabel Plan needed"
    assert_output --partial "NSLabel Plan ready"
    refute_output --partial "NSLabel No plan"
}

@test "list --label empty label after ! is an error" {
    run "$WK_BIN" list --label '!'
    assert_failure
    assert_output --partial "cannot be empty"
}
