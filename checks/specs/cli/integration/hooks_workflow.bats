#!/usr/bin/env bats
load '../../helpers/common'

# ============================================================================
# Integration with Issue Tracking Workflow
# ============================================================================

@test "hooks can be installed in initialized project" {
    init_project test
    run "$WK_BIN" hooks install -y local
    assert_success
    [ -f ".claude/settings.local.json" ]
}

@test "hooks work alongside .wok directory" {
    init_project test
    run "$WK_BIN" hooks install -y project
    assert_success

    # Both should exist
    [ -d ".wok" ]
    [ -f ".claude/settings.json" ]
}

@test "hooks status after install and uninstall cycle" {
    run "$WK_BIN" hooks install -y local
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "installed"

    run "$WK_BIN" hooks uninstall local
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "No hooks"
}

@test "multiple scope installations tracked separately" {
    run "$WK_BIN" hooks install -y local
    run "$WK_BIN" hooks install -y project
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    # Both should be shown
    assert_output --partial "local"
    assert_output --partial "project"

    # Uninstall one
    run "$WK_BIN" hooks uninstall local
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    # Only project should remain
    refute_output --partial "local.*installed"
    assert_output --partial "project"
}

@test "hooks install and wk commands work together" {
    init_project test
    run "$WK_BIN" hooks install -y local
    assert_success

    # Normal wk commands should still work
    run "$WK_BIN" new task "Test issue"
    assert_success

    run "$WK_BIN" list
    assert_success
    assert_output --partial "Test issue"
}

@test "hooks installed in project visible to collaborators" {
    # Simulate installing to project scope
    run "$WK_BIN" hooks install -y project
    assert_success

    # Create a "collaborator" by changing to different temp home
    local collab_home
    collab_home=$(mktemp -d)
    HOME="$collab_home" run "$WK_BIN" hooks status
    assert_success
    # Project hooks should still be visible (they're in current dir)
    assert_output --partial "project"
}

@test "hooks survive wk operations" {
    init_project test
    run "$WK_BIN" hooks install -y local
    assert_success

    # Perform various wk operations
    run "$WK_BIN" new task "Test"
    assert_success
    local id
    id=$(echo "$output" | grep -oE 'test-[a-z0-9]+')

    run "$WK_BIN" start "$id"
    assert_success

    run "$WK_BIN" done "$id"
    assert_success

    # Hooks should still be there
    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "local"
    assert_output --partial "installed"
}

@test "hooks and wk init order independent" {
    # Install hooks first, then init
    run "$WK_BIN" hooks install -y local
    assert_success

    init_project test

    # Both should work
    [ -f ".claude/settings.local.json" ]
    [ -d ".wok" ]

    run "$WK_BIN" list
    assert_success

    run "$WK_BIN" hooks status
    assert_success
    assert_output --partial "installed"
}

# ============================================================================
# Smart Merge Integration Tests
# ============================================================================

@test "hooks install then uninstall preserves custom hooks" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "custom.sh"}]}]}}
EOF

    # Install wk hooks
    run "$WK_BIN" hooks install -y local
    assert_success

    # Uninstall wk hooks
    run "$WK_BIN" hooks uninstall local
    assert_success

    # Custom hooks should remain
    grep -q 'custom.sh' .claude/settings.local.json
}

@test "hooks status accurately reflects partial installation" {
    mkdir -p .claude
    # Only PreCompact, no SessionStart
    cat > .claude/settings.local.json << 'EOF'
{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "wk prime"}]}]}}
EOF

    run "$WK_BIN" hooks status
    # Should indicate installed (has hooks key)
    assert_success
    assert_output --partial "local"
}

@test "multiple scopes with mixed configurations" {
    # Install to local with custom hooks
    mkdir -p .claude
    echo '{"hooks": {"PreCompact": [{"matcher": "", "hooks": [{"type": "command", "command": "local-hook.sh"}]}]}}' > .claude/settings.local.json

    # Install wk hooks to both
    run "$WK_BIN" hooks install -y local
    run "$WK_BIN" hooks install -y project

    # Verify local has both hooks
    grep -q 'local-hook.sh' .claude/settings.local.json
    grep -q 'wk prime' .claude/settings.local.json

    # Verify project has only wk hooks
    grep -q 'wk prime' .claude/settings.json
    ! grep -q 'local-hook.sh' .claude/settings.json
}

@test "reinstall does not change file when hooks already present" {
    run "$WK_BIN" hooks install -y local
    assert_success
    local first_content
    first_content=$(cat .claude/settings.local.json)

    # Install again
    run "$WK_BIN" hooks install -y local
    assert_success
    local second_content
    second_content=$(cat .claude/settings.local.json)

    # Content should be identical
    [ "$first_content" = "$second_content" ]
}

@test "hooks work with complex existing configuration" {
    mkdir -p .claude
    cat > .claude/settings.local.json << 'EOF'
{
  "mcpServers": {
    "test": {"command": "echo test"}
  },
  "hooks": {
    "PostToolUse": [
      {"matcher": "Bash", "hooks": [{"type": "command", "command": "lint.sh"}]},
      {"matcher": "Edit", "hooks": [{"type": "command", "command": "format.sh"}]}
    ],
    "PreCompact": [
      {"matcher": "", "hooks": [{"type": "command", "command": "save-context.sh"}]}
    ]
  },
  "otherSetting": true
}
EOF

    run "$WK_BIN" hooks install -y local
    assert_success

    # All original content should be preserved
    grep -q 'mcpServers' .claude/settings.local.json
    grep -q 'PostToolUse' .claude/settings.local.json
    grep -q 'lint.sh' .claude/settings.local.json
    grep -q 'format.sh' .claude/settings.local.json
    grep -q 'save-context.sh' .claude/settings.local.json
    grep -q 'otherSetting' .claude/settings.local.json

    # wk hooks should be added
    grep -q 'wk prime' .claude/settings.local.json
    grep -q 'SessionStart' .claude/settings.local.json
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
