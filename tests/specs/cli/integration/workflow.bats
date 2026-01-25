#!/usr/bin/env bats
load '../../helpers/common'

# Per-test isolation - each test calls init_project with different prefixes
setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

# Full workflow test from DESIGN.md section 5 verification plan

@test "complete workflow from DESIGN.md" {
    init_project "auth"

    # 2. Create issues
    feature=$("$WK_BIN" new feature "Build auth system" --label project:auth | grep -oE 'auth-[a-z0-9]+(-[0-9]+)?' | head -1)
    schema=$("$WK_BIN" new task "Design database schema" --label project:auth | grep -oE 'auth-[a-z0-9]+(-[0-9]+)?' | head -1)
    login=$("$WK_BIN" new task "Implement login endpoint" --label project:auth | grep -oE 'auth-[a-z0-9]+(-[0-9]+)?' | head -1)
    hash=$("$WK_BIN" new bug "Fix password hashing" --label priority:high | grep -oE 'auth-[a-z0-9]+(-[0-9]+)?' | head -1)

    # Verify all created
    [ -n "$feature" ]
    [ -n "$schema" ]
    [ -n "$login" ]
    [ -n "$hash" ]

    # 3. Set up dependencies
    run "$WK_BIN" dep "$feature" tracks "$schema" "$login"
    assert_success
    run "$WK_BIN" dep "$schema" blocks "$login"
    assert_success

    # 4. Verify blocking - login should be blocked
    # list shows all open issues (blocked and unblocked)
    run "$WK_BIN" list
    assert_success
    assert_output --partial "$login"

    # ready shows only unblocked issues
    run "$WK_BIN" ready
    assert_success
    refute_output --partial "$login"

    run "$WK_BIN" list --blocked
    assert_success
    assert_output --partial "$login"

    # 5. Start work and add notes
    run "$WK_BIN" start "$schema"
    assert_success
    run "$WK_BIN" note "$schema" "Using PostgreSQL with normalized schema"
    assert_success

    # 6. Test reopen (return to backlog)
    run "$WK_BIN" reopen "$schema"
    assert_success
    run "$WK_BIN" list
    assert_output --partial "$schema"

    run "$WK_BIN" start "$schema"
    assert_success

    # 7. Complete work
    run "$WK_BIN" done "$schema"
    assert_success

    # Login should now be unblocked
    run "$WK_BIN" list
    assert_output --partial "$login"

    run "$WK_BIN" start "$login"
    assert_success

    # 8. Test close/reopen
    run "$WK_BIN" close "$hash" --reason "not a bug, works as designed"
    assert_success
    run "$WK_BIN" reopen "$hash" --reason "actually is a bug, reproduced"
    assert_success

    # 9. Verify log
    run "$WK_BIN" log "$schema"
    assert_success
    assert_output --partial "created"
    assert_output --partial "started"

    run "$WK_BIN" log "$hash"
    assert_success
    assert_output --partial "closed"
    assert_output --partial "reopened"

    # 10. Test filtering
    run "$WK_BIN" list --label project:auth
    assert_success

    run "$WK_BIN" list --status done
    assert_success
    assert_output --partial "$schema"

    # 11. Test edit
    run "$WK_BIN" edit "$hash" title "Fix password hashing (revised)"
    assert_success
    run "$WK_BIN" edit "$hash" type task
    assert_success

    # 12. View issue with deps
    run "$WK_BIN" show "$feature"
    assert_success
    assert_output --partial "Tracks"

    run "$WK_BIN" show "$login"
    assert_success
    assert_output --partial "Tracked by"

    run "$WK_BIN" tree "$feature"
    assert_success

    # 13. Test export
    run "$WK_BIN" export "$TEST_DIR/issues.jsonl"
    assert_success
    [ -f "$TEST_DIR/issues.jsonl" ]
}

@test "workflow handles concurrent work" {
    init_project
    t1=$(create_issue task "Task 1")
    t2=$(create_issue task "Task 2")
    t3=$(create_issue task "Task 3")

    # Start multiple tasks
    "$WK_BIN" start "$t1"
    "$WK_BIN" start "$t2"

    run "$WK_BIN" list --status in_progress
    assert_success
    assert_output --partial "$t1"
    assert_output --partial "$t2"
    refute_output --partial "$t3"
}

@test "workflow with feature hierarchy" {
    init_project
    feature=$(create_issue feature "Main Feature")
    t1=$(create_issue task "Task 1")
    t2=$(create_issue task "Task 2")
    t3=$(create_issue task "Task 3")

    # Feature tracks all tasks
    "$WK_BIN" dep "$feature" tracks "$t1" "$t2" "$t3"

    # T1 blocks T2, T2 blocks T3
    "$WK_BIN" dep "$t1" blocks "$t2"
    "$WK_BIN" dep "$t2" blocks "$t3"

    # Only T1 should be ready (unblocked)
    run "$WK_BIN" ready
    assert_output --partial "$t1"
    refute_output --partial "$t2"
    refute_output --partial "$t3"

    # Complete T1
    "$WK_BIN" start "$t1"
    "$WK_BIN" done "$t1"

    # Now T2 should be ready
    run "$WK_BIN" ready
    assert_output --partial "$t2"
    refute_output --partial "$t3"

    # Complete T2
    "$WK_BIN" start "$t2"
    "$WK_BIN" done "$t2"

    # Now T3 should be ready
    run "$WK_BIN" ready
    assert_output --partial "$t3"
}

@test "chore full lifecycle" {
    init_project
    id=$(create_issue chore "Refactor auth module")

    # Verify chore type
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "[chore]"
    assert_output --partial "Status: todo"

    # Start work
    "$WK_BIN" start "$id"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Status: in_progress"

    # Complete work
    "$WK_BIN" done "$id"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Status: done"
}

# Prime command integration tests

@test "prime works alongside initialized project" {
    init_project
    run "$WK_BIN" prime
    assert_success
    # Should still output template even with .wok/ present
    assert_output --partial "## Core Rules"
    assert_output --partial "## Finding Work"
}

@test "prime output can be piped to file" {
    run bash -c "$WK_BIN prime > $TEST_DIR/template.md"
    assert_success
    [ -f "$TEST_DIR/template.md" ]
    [ -s "$TEST_DIR/template.md" ]
    # Verify content
    grep -q "## Core Rules" "$TEST_DIR/template.md"
}

@test "prime output is consistent across calls" {
    run "$WK_BIN" prime
    first_output="$output"
    run "$WK_BIN" prime
    assert_equal "$output" "$first_output"
}
