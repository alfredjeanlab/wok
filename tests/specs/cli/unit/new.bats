#!/usr/bin/env bats
load '../../helpers/common'

@test "new creates issues with correct type (default task, feature, bug, chore, idea)" {
    # Default creates task
    run "$WK_BIN" new "NewType My task"
    assert_success
    assert_output --partial "[task]"

    # Explicit task
    run "$WK_BIN" new task "NewType My explicit task"
    assert_success
    assert_output --partial "[task]"

    # Feature
    run "$WK_BIN" new feature "NewType My feature"
    assert_success
    assert_output --partial "[feature]"

    # Bug
    run "$WK_BIN" new bug "NewType My bug"
    assert_success
    assert_output --partial "[bug]"

    # Chore
    run "$WK_BIN" new chore "NewType My chore"
    assert_success
    assert_output --partial "[chore]"

    # Idea
    run "$WK_BIN" new idea "NewType My idea"
    assert_success
    assert_output --partial "[idea]"

    # Starts in todo status
    run "$WK_BIN" new "NewType Status task"
    assert_success
    assert_output --partial "(todo)"
}

@test "new with --label and --note adds metadata" {
    # --label adds label
    id=$(create_issue task "NewMeta Labeled task" --label "project:auth")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"

    # Multiple --label adds all labels
    id=$(create_issue task "NewMeta Multi-labeled task" --label "project:auth" --label "priority:high")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "project:auth"
    assert_output --partial "priority:high"

    # --note adds note
    id=$(create_issue task "NewMeta Noted task" --note "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"
}

@test "new generates unique ID with prefix and validates inputs" {
    # Generate ID with prefix (use subdirectory for different prefix)
    mkdir -p subproj && cd subproj
    "$WK_BIN" init --prefix myproj
    run "$WK_BIN" new "Test task"
    assert_success
    assert_output --partial "myproj-"
    cd ..

    # Requires title
    run "$WK_BIN" new
    assert_failure

    # Empty title fails
    run "$WK_BIN" new ""
    assert_failure

    # Epic
    run "$WK_BIN" new epic "My epic"
    assert_success
    assert_output --partial "[epic]"

    # Invalid type fails
    run "$WK_BIN" new bogus "My bogus"
    assert_failure
}

@test "new with --priority adds priority label" {
    # --priority=0 adds priority:0 label
    id=$(create_issue task "NewPrio Critical task" --priority 0)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:0"

    # --priority=4 adds priority:4 label
    id=$(create_issue task "NewPrio Low priority" --priority 4)
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:4"

    # --priority and --label combine
    id=$(create_issue task "NewPrio Labeled priority" --priority 1 --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "priority:1"
    assert_output --partial "backend"

    # Invalid priority values fail
    run "$WK_BIN" new task "Invalid" --priority 5
    assert_failure

    run "$WK_BIN" new task "Invalid" --priority -1
    assert_failure

    run "$WK_BIN" new task "Invalid" --priority high
    assert_failure
}

@test "new with --description adds note as Description" {
    # --description adds note
    id=$(create_issue task "NewDesc Described task" --description "Initial context")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Initial context"

    # Shows as Description in output
    id=$(create_issue task "NewDesc Described task 2" --description "My description")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "My description"

    # --note also works (documented happy path)
    id=$(create_issue task "NewDesc Noted task" --note "Initial note")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "Initial note"

    # --description and --label combine
    id=$(create_issue task "NewDesc Labeled described" --description "Context" --label "backend")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Context"
    assert_output --partial "backend"
}

@test "new with comma-separated labels adds all labels" {
    # Comma-separated labels
    id=$(create_issue task "NewComma Comma labels" --label "a,b,c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: a, b, c"

    # Comma-separated and multiple --label combine
    id=$(create_issue task "NewComma Mixed labels" --label "a,b" --label "c")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels:"
    assert_output --partial "a"
    assert_output --partial "b"
    assert_output --partial "c"

    # Whitespace around comma-separated labels trims correctly
    id=$(create_issue task "NewComma Whitespace labels" --label "  x  ,  y  ")
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Labels: x, y"
}

@test "new -o id outputs only the issue ID" {
    # Default text output includes full message
    run "$WK_BIN" new task "NewOutput Text task"
    assert_success
    assert_output --partial "Created [task]"

    # -o id outputs only the ID
    run "$WK_BIN" new task "NewOutput ID task" -o id
    assert_success
    # Output should be just the ID (prefix-xxxx format)
    [[ "$output" =~ ^[a-z]+-[a-f0-9]+$ ]]
    # Should NOT contain the verbose message
    refute_output --partial "Created"
    refute_output --partial "[task]"

    # The ID should be valid (can be shown)
    run "$WK_BIN" show "$output"
    assert_success
    assert_output --partial "NewOutput ID task"
}

@test "new -o ids alias works for backward compatibility" {
    # -o ids should work the same as -o id
    run "$WK_BIN" new task "NewOutput IDs alias" -o ids
    assert_success
    # Output should be just the ID
    [[ "$output" =~ ^[a-z]+-[a-f0-9]+$ ]]
    refute_output --partial "Created"
}

@test "new -o json outputs valid JSON with expected fields" {
    run "$WK_BIN" new task "NewOutput JSON task" --label "test:json" -o json
    assert_success

    # Should be valid JSON
    echo "$output" | jq -e . >/dev/null

    # Check required fields
    echo "$output" | jq -e '.id' >/dev/null
    echo "$output" | jq -e '.type == "task"' >/dev/null
    echo "$output" | jq -e '.title == "NewOutput JSON task"' >/dev/null
    echo "$output" | jq -e '.status == "todo"' >/dev/null
    echo "$output" | jq -e '.labels | index("test:json")' >/dev/null
}

@test "new -o id enables scripting workflows" {
    # Capture ID and use in subsequent commands
    id=$("$WK_BIN" new task "NewOutput Scripted task" -o id)
    [[ -n "$id" ]]

    # Add label using captured ID
    run "$WK_BIN" label "$id" "scripted"
    assert_success

    # Verify label was added
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "scripted"
}

@test "new --prefix creates issue with different prefix" {
    # Use subdirectory to avoid conflicts
    mkdir -p newprefix && cd newprefix
    "$WK_BIN" init --prefix main

    # Create issue with different prefix using --prefix
    run "$WK_BIN" new "NewPrefix Task" --prefix other -o id
    assert_success
    [[ "$output" == other-* ]]

    # -p short form works
    run "$WK_BIN" new "NewPrefix Short" -p short -o id
    assert_success
    [[ "$output" == short-* ]]

    # Works with other flags
    run "$WK_BIN" new bug "NewPrefix Bug" --prefix api --label urgent -o id
    assert_success
    [[ "$output" == api-* ]]

    run "$WK_BIN" show "$output"
    assert_success
    assert_output --partial "urgent"
}
