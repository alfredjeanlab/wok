#!/usr/bin/env bats
load '../../helpers/common'

# ============================================================================
# Help and Documentation
# ============================================================================

@test "hooks shows help with no subcommand" {
    run timeout 3 "$WK_BIN" hooks
    # Should show usage, not hang waiting for input (exit 2 for missing subcommand is OK)
    [ "$status" -ne 124 ]  # Not killed by timeout
    assert_output --partial "Usage"
}

@test "hooks install --help shows usage" {
    run "$WK_BIN" hooks install --help
    assert_success
    assert_output --partial "local"
    assert_output --partial "project"
    assert_output --partial "user"
}

@test "hooks -h shows help" {
    run "$WK_BIN" hooks -h
    assert_success
    assert_output --partial "hooks"
}

# ============================================================================
# Non-Interactive Mode (-y flag)
# ============================================================================

@test "hooks install -y defaults to local scope" {
    run "$WK_BIN" hooks install -y
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks install -y local creates settings.local.json" {
    run "$WK_BIN" hooks install -y local
    assert_success
    [ -f ".claude/settings.local.json" ]
    grep -q '"hooks"' .claude/settings.local.json
}

@test "hooks install -y project creates settings.json" {
    run "$WK_BIN" hooks install -y project
    assert_success
    [ -f ".claude/settings.json" ]
    grep -q '"hooks"' .claude/settings.json
}

@test "hooks install -y user creates ~/.claude/settings.json" {
    run "$WK_BIN" hooks install -y user
    assert_success
    [ -f "$HOME/.claude/settings.json" ]
    grep -q '"hooks"' "$HOME/.claude/settings.json"
}

@test "hooks install -y is idempotent" {
    run "$WK_BIN" hooks install -y local
    assert_success
    local first_content
    first_content=$(cat .claude/settings.local.json)

    run "$WK_BIN" hooks install -y local
    assert_success
    local second_content
    second_content=$(cat .claude/settings.local.json)

    [ "$first_content" = "$second_content" ]
}

@test "hooks install -y preserves existing settings" {
    mkdir -p .claude
    echo '{"mcpServers": {"test": {}}}' > .claude/settings.local.json

    run "$WK_BIN" hooks install -y local
    assert_success

    # Should have both hooks and existing mcpServers
    grep -q '"hooks"' .claude/settings.local.json
    grep -q '"mcpServers"' .claude/settings.local.json
}

# ============================================================================
# Uninstall
# ============================================================================

@test "hooks uninstall removes hooks from local" {
    # Install first
    run "$WK_BIN" hooks install -y local
    assert_success

    # Uninstall
    run "$WK_BIN" hooks uninstall local
    assert_success

    # File may still exist but hooks should be removed
    if [ -f ".claude/settings.local.json" ]; then
        ! grep -q '"PreCompact"' .claude/settings.local.json
    fi
}

@test "hooks uninstall preserves other settings" {
    mkdir -p .claude
    echo '{"mcpServers": {"test": {}}, "hooks": {"PreCompact": []}}' > .claude/settings.local.json

    run "$WK_BIN" hooks uninstall local
    assert_success

    # mcpServers should remain
    grep -q '"mcpServers"' .claude/settings.local.json
}

@test "hooks uninstall on non-existent file succeeds" {
    run "$WK_BIN" hooks uninstall local
    assert_success
}

@test "hooks uninstall does not accept -y flag" {
    run "$WK_BIN" hooks uninstall -y local
    assert_failure
    assert_output --partial "unexpected argument"
}

# ============================================================================
# Status
# ============================================================================

@test "hooks status shows no hooks when none installed" {
    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "No hooks installed"
}

@test "hooks status shows installed hooks" {
    run "$WK_BIN" hooks install -y local
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "local"
    assert_output --partial "installed"
}

@test "hooks status shows multiple scopes" {
    run "$WK_BIN" hooks install -y local
    run "$WK_BIN" hooks install -y project
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "local"
    assert_output --partial "project"
}

# ============================================================================
# Auto-Detection (Non-TTY)
# ============================================================================

@test "hooks install defaults to non-interactive when not a TTY" {
    # Run in subshell with stdin from /dev/null (non-TTY)
    run timeout 3 bash -c 'echo "" | "$WK_BIN" hooks install local'
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks install defaults to non-interactive under CLAUDE_CODE env" {
    CLAUDE_CODE=1 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks install defaults to non-interactive under CODEX_ENV" {
    CODEX_ENV=1 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks install defaults to non-interactive under AIDER_MODEL env" {
    AIDER_MODEL=gpt-4 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

# ============================================================================
# Error Handling
# ============================================================================

@test "hooks install rejects invalid scope" {
    run "$WK_BIN" hooks install -y invalid
    assert_failure
    assert_output --partial "invalid"
}

@test "hooks uninstall rejects invalid scope" {
    run "$WK_BIN" hooks uninstall invalid
    assert_failure
}

@test "hooks install fails gracefully on permission error" {
    # Create read-only directory
    mkdir -p .claude
    chmod 444 .claude

    run "$WK_BIN" hooks install -y local
    assert_failure
    assert_output --partial "permission" || assert_output --partial "Permission"

    # Cleanup
    chmod 755 .claude
}

@test "hooks with both -i and -y flags errors" {
    run "$WK_BIN" hooks install -i -y local
    assert_failure
    assert_output --partial "cannot be used with"
}

# ============================================================================
# JSON Output Format
# ============================================================================

@test "installed hooks contain PreCompact" {
    run "$WK_BIN" hooks install -y local
    assert_success
    grep -q '"PreCompact"' .claude/settings.local.json
}

@test "installed hooks contain valid JSON" {
    run "$WK_BIN" hooks install -y local
    assert_success
    # Validate JSON syntax
    python3 -c "import json; json.load(open('.claude/settings.local.json'))" || \
    jq . .claude/settings.local.json > /dev/null
}

@test "installed hooks reference wk prime command" {
    run "$WK_BIN" hooks install -y local
    assert_success
    grep -q 'wk prime' .claude/settings.local.json
}

# ============================================================================
# Timeout Protection for Potentially Interactive Commands
# ============================================================================

@test "hooks install without scope times out in non-TTY" {
    # This should NOT hang - it should either:
    # 1. Auto-detect non-TTY and use default scope
    # 2. Show help and exit
    run timeout 3 bash -c '"$WK_BIN" hooks install < /dev/null'
    # Should complete within timeout (success or failure, but not hang)
    [ $status -ne 124 ]  # 124 = timeout killed the process
}

@test "hooks command never hangs in CI environment" {
    # Simulate CI by setting common CI env vars
    CI=true GITHUB_ACTIONS=true run timeout 3 "$WK_BIN" hooks install local
    assert_success
}

# ============================================================================
# Works Without Project Initialization
# ============================================================================

@test "hooks install works without wk init" {
    # Don't call init_project - this should work anyway
    run "$WK_BIN" hooks install -y local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks status works without wk init" {
    run "$WK_BIN" hooks status
    assert_success
}

# ============================================================================
# Smart Merge Behavior
# ============================================================================

@test "hooks install preserves existing non-wk hooks" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "custom-script.sh"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks install -y local
    assert_success
    # Both custom hook and wk hook should exist
    grep -q 'custom-script.sh' .claude/settings.local.json
    grep -q 'wk prime' .claude/settings.local.json
}

@test "hooks install does not duplicate wk hooks" {
    run "$WK_BIN" hooks install -y local
    run "$WK_BIN" hooks install -y local
    assert_success
    # Count occurrences of "wk prime" - should be exactly 2 (PreCompact + SessionStart)
    local count
    count=$(grep -o 'wk prime' .claude/settings.local.json | wc -l | tr -d ' ')
    [ "$count" -eq 2 ]
}

@test "hooks install adds missing events to partial config" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks install -y local
    assert_success
    # SessionStart should now be added
    grep -q '"SessionStart"' .claude/settings.local.json
}

@test "hooks install preserves hooks on other events" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PostToolUse": [
      {"matcher": "", "hooks": [{"type": "command", "command": "my-hook.sh"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks install -y local
    assert_success
    grep -q 'PostToolUse' .claude/settings.local.json
    grep -q 'my-hook.sh' .claude/settings.local.json
}

@test "hooks uninstall only removes wk hooks preserving others" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]},
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks uninstall local
    assert_success
    # Custom hook should remain
    grep -q 'custom.sh' .claude/settings.local.json
    # wk prime should be gone
    ! grep -q 'wk prime' .claude/settings.local.json
}

@test "hooks install detects wk prime with full path" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "/usr/local/bin/wk prime"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks install -y local
    assert_success
    # Should not duplicate - count should be 1 for PreCompact, 1 for SessionStart
    local count
    count=$(grep -o 'wk prime' .claude/settings.local.json | wc -l | tr -d ' ')
    [ "$count" -eq 2 ]
}

@test "hooks install detects wk prime with args" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "hooks": {
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "wk prime --verbose"}]}
    ]
  }
}
EOF
    run "$WK_BIN" hooks install -y local
    assert_success
    # Should not duplicate PreCompact wk hook
    local count
    count=$(grep -c 'PreCompact' .claude/settings.local.json | tr -d ' ')
    [ "$count" -eq 1 ]
}

# ============================================================================
# Setup / Teardown
# ============================================================================

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}
