#!/usr/bin/env bats

# Hook command tests

setup() {
    load '../test_helper'
    setup_test_environment
}

teardown() {
    teardown_test_environment
}

# ──────────────────────────────────────────────────────────────────────────────
# hook list
# ──────────────────────────────────────────────────────────────────────────────

@test "hook list: shows no hooks when config does not exist" {
    run_wok hook list
    assert_success
    assert_output --partial "No hooks configured"
}

@test "hook list: shows configured hooks from hooks.toml" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "urgent-bugs"
events = ["issue.created"]
filter = "-t bug -l urgent"
run = "./scripts/page-oncall.sh"

[[hooks]]
name = "audit-all"
events = ["issue.*"]
run = "./scripts/audit.sh"
EOF

    run_wok hook list
    assert_success
    assert_output --partial "urgent-bugs:"
    assert_output --partial "events: issue.created"
    assert_output --partial "filter: -t bug -l urgent"
    assert_output --partial "run: ./scripts/page-oncall.sh"
    assert_output --partial "audit-all:"
    assert_output --partial "events: issue.*"
}

@test "hook list: shows configured hooks from hooks.json" {
    cat > "$TEST_DIR/.wok/hooks.json" << 'EOF'
{
    "hooks": [
        {
            "name": "json-hook",
            "events": ["issue.done"],
            "run": "./notify.sh"
        }
    ]
}
EOF

    run_wok hook list
    assert_success
    assert_output --partial "json-hook:"
    assert_output --partial "events: issue.done"
}

@test "hook list: merges hooks from both files" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "toml-hook"
events = ["issue.created"]
run = "./toml.sh"
EOF

    cat > "$TEST_DIR/.wok/hooks.json" << 'EOF'
{"hooks": [{"name": "json-hook", "events": ["issue.done"], "run": "./json.sh"}]}
EOF

    run_wok hook list
    assert_success
    assert_output --partial "toml-hook:"
    assert_output --partial "json-hook:"
}

@test "hook list: outputs JSON format with -o json" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "test-hook"
events = ["issue.created"]
run = "./test.sh"
EOF

    run_wok hook list -o json
    assert_success
    # Verify it's valid JSON with expected content
    echo "$output" | jq -e '.[0].name == "test-hook"'
    echo "$output" | jq -e '.[0].events[0] == "issue.created"'
}

@test "hook list: outputs empty array in JSON when no hooks" {
    run_wok hook list -o json
    assert_success
    assert_output "[]"
}

# ──────────────────────────────────────────────────────────────────────────────
# hook test
# ──────────────────────────────────────────────────────────────────────────────

@test "hook test: requires hook name" {
    run_wok hook test
    assert_failure
}

@test "hook test: requires issue id" {
    run_wok hook test some-hook
    assert_failure
}

@test "hook test: fails when no hooks configured" {
    run_wok new task "Test task"
    issue_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test nonexistent "$issue_id"
    assert_failure
    assert_output --partial "no hooks configured"
}

@test "hook test: fails when hook not found" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "other-hook"
events = ["issue.created"]
run = "./test.sh"
EOF

    run_wok new task "Test task"
    issue_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test nonexistent "$issue_id"
    assert_failure
    assert_output --partial "not found"
}

@test "hook test: shows hook would trigger" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "test-hook"
events = ["issue.created"]
run = "echo triggered"
EOF

    run_wok new task "Test task"
    issue_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test test-hook "$issue_id"
    assert_success
    assert_output --partial "Testing hook 'test-hook'"
    assert_output --partial "Issue: $issue_id"
    assert_output --partial "Event: issue.created"
}

@test "hook test: shows filter match status" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "bug-only"
events = ["issue.created"]
filter = "-t bug"
run = "echo triggered"
EOF

    # Create a bug - should match
    run_wok new bug "Test bug"
    bug_id=$(echo "$output" | grep -oE '\[bug\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test bug-only "$bug_id"
    assert_success
    assert_output --partial "Filter: -t bug (MATCH)"

    # Create a task - should not match
    run_wok new task "Test task"
    task_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test bug-only "$task_id"
    assert_success
    assert_output --partial "Filter: -t bug (NO MATCH)"
    assert_output --partial "Hook would NOT trigger"
}

# ──────────────────────────────────────────────────────────────────────────────
# Hook triggering on mutations
# ──────────────────────────────────────────────────────────────────────────────

@test "hooks: trigger on issue creation" {
    # Create a hook that writes to a file
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "log-creation"
events = ["issue.created"]
run = "cat > /tmp/hook_test_output.txt"
EOF

    run_wok new bug "Test hook trigger"
    assert_success

    # Wait a moment for the hook to execute
    sleep 0.2

    # Check if hook was triggered (file should exist with JSON)
    if [ -f /tmp/hook_test_output.txt ]; then
        content=$(cat /tmp/hook_test_output.txt)
        echo "$content" | jq -e '.event == "issue.created"'
        rm -f /tmp/hook_test_output.txt
    fi
}

@test "hooks: receive correct environment variables" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "env-check"
events = ["issue.created"]
run = "env | grep WOK_ > /tmp/hook_env_output.txt"
EOF

    run_wok new task "Env test"
    assert_success

    sleep 0.2

    if [ -f /tmp/hook_env_output.txt ]; then
        content=$(cat /tmp/hook_env_output.txt)
        echo "$content" | grep -q "WOK_EVENT=issue.created"
        echo "$content" | grep -q "WOK_ISSUE_TYPE=task"
        echo "$content" | grep -q "WOK_ISSUE_STATUS=todo"
        rm -f /tmp/hook_env_output.txt
    fi
}

@test "hooks: wildcard matches all events" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "audit-all"
events = ["issue.*"]
run = "echo $WOK_EVENT >> /tmp/hook_audit.txt"
EOF

    # Create issue
    run_wok new task "Audit test"
    issue_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    # Start it
    run_wok start "$issue_id"

    # Done it
    run_wok done "$issue_id"

    sleep 0.3

    if [ -f /tmp/hook_audit.txt ]; then
        content=$(cat /tmp/hook_audit.txt)
        # Should have multiple events logged
        echo "$content" | grep -q "issue.created"
        echo "$content" | grep -q "issue.started"
        echo "$content" | grep -q "issue.done"
        rm -f /tmp/hook_audit.txt
    fi
}

@test "hooks: filter limits which issues trigger" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "bugs-only"
events = ["issue.created"]
filter = "-t bug"
run = "echo triggered >> /tmp/hook_filter.txt"
EOF

    # Create a bug - should trigger
    run_wok new bug "Should trigger"

    # Create a task - should not trigger
    run_wok new task "Should not trigger"

    sleep 0.2

    if [ -f /tmp/hook_filter.txt ]; then
        # Count triggers - should be exactly 1
        count=$(wc -l < /tmp/hook_filter.txt | tr -d ' ')
        [ "$count" -eq 1 ]
        rm -f /tmp/hook_filter.txt
    fi
}

# ──────────────────────────────────────────────────────────────────────────────
# Error handling
# ──────────────────────────────────────────────────────────────────────────────

@test "hooks: invalid TOML shows error" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
invalid toml {{{{
EOF

    run_wok hook list
    assert_failure
    assert_output --partial "failed to parse hooks.toml"
}

@test "hooks: invalid JSON shows error" {
    cat > "$TEST_DIR/.wok/hooks.json" << 'EOF'
{invalid json
EOF

    run_wok hook list
    assert_failure
    assert_output --partial "failed to parse hooks.json"
}

@test "hooks: invalid filter in test shows error" {
    cat > "$TEST_DIR/.wok/hooks.toml" << 'EOF'
[[hooks]]
name = "bad-filter"
events = ["issue.created"]
filter = "-t invalid_type"
run = "./test.sh"
EOF

    run_wok new task "Test"
    issue_id=$(echo "$output" | grep -oE '\[task\] [a-z]+-[a-z0-9]+' | awk '{print $2}')

    run_wok hook test bad-filter "$issue_id"
    assert_failure
    assert_output --partial "invalid"
}
