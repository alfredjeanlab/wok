#!/usr/bin/env bats
load '../../helpers/common'

# Test hook list command
@test "hook list with no config shows helpful message" {
    run "$WK_BIN" hook list
    assert_success
    assert_output --partial "No hooks configured"
}

@test "hook list shows configured hooks" {
    # Create hooks config
    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "notify-bugs"
events = ["issue.created"]
filter = "-t bug"
run = "./notify.sh"

[[hooks]]
name = "audit-log"
events = ["issue.*"]
run = "./audit.sh"
EOF

    run "$WK_BIN" hook list
    assert_success
    assert_output --partial "notify-bugs"
    assert_output --partial "issue.created"
    assert_output --partial "-t bug"
    assert_output --partial "./notify.sh"
    assert_output --partial "audit-log"
    assert_output --partial "issue.*"
}

@test "hook list json output" {
    # Create hooks config
    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "test-hook"
events = ["issue.done"]
run = "./test.sh"
EOF

    run "$WK_BIN" hook list -o json
    assert_success
    # Should be valid JSON array
    echo "$output" | jq -e '.[0].name == "test-hook"'
}

@test "hook list no hooks returns empty json array" {
    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
hooks = []
EOF

    run "$WK_BIN" hook list -o json
    assert_success
    assert_output "[]"
}

# Test hook test command
@test "hook test with no hooks shows not found" {
    id=$(create_issue task "Test issue")

    run "$WK_BIN" hook test "my-hook" "$id"
    assert_success
    assert_output --partial "not found"
}

@test "hook test shows hook would fire" {
    id=$(create_issue bug "Test bug")

    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "bug-hook"
events = ["issue.created"]
filter = "-t bug"
run = "./hook.sh"
EOF

    run "$WK_BIN" hook test "bug-hook" "$id"
    assert_success
    assert_output --partial "would fire"
}

@test "hook test shows hook would NOT fire on filter mismatch" {
    id=$(create_issue task "Test task")

    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "bug-only"
events = ["issue.created"]
filter = "-t bug"
run = "./hook.sh"
EOF

    run "$WK_BIN" hook test "bug-only" "$id"
    assert_success
    assert_output --partial "would NOT fire"
}

@test "hook test with specific event" {
    id=$(create_issue task "Test task")

    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "on-done"
events = ["issue.done"]
run = "./done.sh"
EOF

    # Should not fire for created event
    run "$WK_BIN" hook test "on-done" "$id" --event created
    assert_success
    assert_output --partial "would NOT fire"

    # Should fire for done event
    run "$WK_BIN" hook test "on-done" "$id" --event done
    assert_success
    assert_output --partial "would fire"
}

# Test hook loading from JSON
@test "hooks load from hooks.json" {
    mkdir -p .wok
    cat > .wok/hooks.json <<'EOF'
{
  "hooks": [{
    "name": "json-hook",
    "events": ["issue.created"],
    "run": "./json.sh"
  }]
}
EOF

    run "$WK_BIN" hook list
    assert_success
    assert_output --partial "json-hook"
}

# Test merging TOML and JSON
@test "hooks merge from both toml and json" {
    mkdir -p .wok
    cat > .wok/hooks.toml <<'EOF'
[[hooks]]
name = "toml-hook"
events = ["issue.created"]
run = "./toml.sh"
EOF
    cat > .wok/hooks.json <<'EOF'
{
  "hooks": [{
    "name": "json-hook",
    "events": ["issue.done"],
    "run": "./json.sh"
  }]
}
EOF

    run "$WK_BIN" hook list
    assert_success
    assert_output --partial "toml-hook"
    assert_output --partial "json-hook"
}
