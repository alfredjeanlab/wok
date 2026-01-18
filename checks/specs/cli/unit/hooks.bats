#!/usr/bin/env bats
load '../../helpers/common'


setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

@test "hooks help and documentation" {
    # hooks shows help with no subcommand
    run timeout 3 "$WK_BIN" hooks
    [ "$status" -ne 124 ]  # Not killed by timeout
    assert_output --partial "Usage"

    # hooks install --help shows usage
    run "$WK_BIN" hooks install --help
    assert_success
    assert_output --partial "local"
    assert_output --partial "project"
    assert_output --partial "user"

    # hooks -h shows help
    run "$WK_BIN" hooks -h
    assert_success
    assert_output --partial "hooks"
}

@test "hooks install -y creates settings for each scope" {
    # -y defaults to local scope
    run "$WK_BIN" hooks install -y
    assert_success
    [ -f ".claude/settings.local.json" ]
    rm -rf .claude

    # -y local creates settings.local.json
    run "$WK_BIN" hooks install -y local
    assert_success
    [ -f ".claude/settings.local.json" ]
    grep -q '"hooks"' .claude/settings.local.json
    rm -rf .claude

    # -y project creates settings.json
    run "$WK_BIN" hooks install -y project
    assert_success
    [ -f ".claude/settings.json" ]
    grep -q '"hooks"' .claude/settings.json
    rm -rf .claude

    # -y user creates ~/.claude/settings.json
    run "$WK_BIN" hooks install -y user
    assert_success
    [ -f "$HOME/.claude/settings.json" ]
    grep -q '"hooks"' "$HOME/.claude/settings.json"
}

@test "hooks install -y is idempotent and preserves existing settings" {
    # First install
    run "$WK_BIN" hooks install -y local
    assert_success
    local first_content
    first_content=$(cat .claude/settings.local.json)

    # Second install should be identical
    run "$WK_BIN" hooks install -y local
    assert_success
    local second_content
    second_content=$(cat .claude/settings.local.json)
    [ "$first_content" = "$second_content" ]

    # Preserves existing settings
    rm -rf .claude
    mkdir -p .claude
    echo '{"mcpServers": {"test": {}}}' > .claude/settings.local.json
    run "$WK_BIN" hooks install -y local
    assert_success
    grep -q '"hooks"' .claude/settings.local.json
    grep -q '"mcpServers"' .claude/settings.local.json
}

@test "hooks uninstall removes hooks and preserves other settings" {
    # Install then uninstall from local
    run "$WK_BIN" hooks install -y local
    assert_success
    run "$WK_BIN" hooks uninstall local
    assert_success
    if [ -f ".claude/settings.local.json" ]; then
        ! grep -q '"PreCompact"' .claude/settings.local.json
    fi

    # Preserves other settings
    rm -rf .claude
    mkdir -p .claude
    echo '{"mcpServers": {"test": {}}, "hooks": {"PreCompact": []}}' > .claude/settings.local.json
    run "$WK_BIN" hooks uninstall local
    assert_success
    grep -q '"mcpServers"' .claude/settings.local.json

    # Uninstall on non-existent file succeeds
    rm -rf .claude
    run "$WK_BIN" hooks uninstall local
    assert_success

    # Uninstall does not accept -y flag
    run "$WK_BIN" hooks uninstall -y local
    assert_failure
    assert_output --partial "unexpected argument"
}

@test "hooks status shows installation state" {
    # No hooks when none installed
    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "No hooks installed"

    # Shows installed hooks for single scope
    run "$WK_BIN" hooks install -y local
    assert_success
    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "local"
    assert_output --partial "installed"

    # Shows multiple scopes
    run "$WK_BIN" hooks install -y project
    assert_success
    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "local"
    assert_output --partial "project"
}

@test "hooks install auto-detects non-interactive mode" {
    # Non-TTY (stdin from /dev/null)
    run timeout 3 bash -c 'echo "" | "$WK_BIN" hooks install local'
    assert_success
    [ -f ".claude/settings.local.json" ]
    rm -rf .claude

    # CLAUDE_CODE env
    CLAUDE_CODE=1 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
    rm -rf .claude

    # CODEX_ENV env
    CODEX_ENV=1 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
    rm -rf .claude

    # AIDER_MODEL env
    AIDER_MODEL=gpt-4 run timeout 3 "$WK_BIN" hooks install local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks install error handling" {
    # Rejects invalid scope
    run "$WK_BIN" hooks install -y invalid
    assert_failure
    assert_output --partial "invalid"

    # uninstall rejects invalid scope
    run "$WK_BIN" hooks uninstall invalid
    assert_failure

    # Fails gracefully on permission error
    mkdir -p .claude
    chmod 444 .claude
    run "$WK_BIN" hooks install -y local
    assert_failure
    assert_output --partial "permission" || assert_output --partial "Permission"
    chmod 755 .claude

    # Both -i and -y flags errors
    run "$WK_BIN" hooks install -i -y local
    assert_failure
    assert_output --partial "cannot be used with"
}

@test "hooks install creates valid JSON with PreCompact and wk prime" {
    run "$WK_BIN" hooks install -y local
    assert_success

    # Contains PreCompact
    grep -q '"PreCompact"' .claude/settings.local.json

    # Valid JSON syntax
    python3 -c "import json; json.load(open('.claude/settings.local.json'))" || \
    jq . .claude/settings.local.json > /dev/null

    # References wk prime command
    grep -q 'wk prime' .claude/settings.local.json
}

@test "hooks install does not hang in non-TTY or CI" {
    # Without scope times out in non-TTY - should not hang
    run timeout 3 bash -c '"$WK_BIN" hooks install < /dev/null'
    [ $status -ne 124 ]  # 124 = timeout killed the process

    # CI environment
    CI=true GITHUB_ACTIONS=true run timeout 3 "$WK_BIN" hooks install local
    assert_success
}

@test "hooks work without wk init" {
    # Install works without wk init
    run "$WK_BIN" hooks install -y local
    assert_success
    [ -f ".claude/settings.local.json" ]

    # Status works without wk init
    run "$WK_BIN" hooks status
    assert_success
}

@test "hooks install smart merge preserves existing hooks" {
    # Preserves existing non-wk hooks
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
    grep -q 'custom-script.sh' .claude/settings.local.json
    grep -q 'wk prime' .claude/settings.local.json

    # Does not duplicate wk hooks
    rm -rf .claude
    run "$WK_BIN" hooks install -y local
    run "$WK_BIN" hooks install -y local
    assert_success
    local count
    count=$(grep -o 'wk prime' .claude/settings.local.json | wc -l | tr -d ' ')
    [ "$count" -eq 2 ]

    # Adds missing events to partial config
    rm -rf .claude
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
    grep -q '"SessionStart"' .claude/settings.local.json

    # Preserves hooks on other events
    rm -rf .claude
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
    grep -q 'custom.sh' .claude/settings.local.json
    ! grep -q 'wk prime' .claude/settings.local.json
}

@test "hooks install detects wk prime with full path or args" {
    # Full path
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
    local count
    count=$(grep -o 'wk prime' .claude/settings.local.json | wc -l | tr -d ' ')
    [ "$count" -eq 2 ]

    # With args
    rm -rf .claude
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
    count=$(grep -c 'PreCompact' .claude/settings.local.json | tr -d ' ')
    [ "$count" -eq 1 ]
}
