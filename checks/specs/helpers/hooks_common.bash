#!/usr/bin/env bash

# Load parent common helpers
load '../helpers/common'

# Timeout for commands that might hang on interactive input
HOOKS_TIMEOUT=5

# Run command with timeout to prevent hangs
run_with_timeout() {
    local timeout="${1:-$HOOKS_TIMEOUT}"
    shift
    timeout "$timeout" "$@"
}

# Force non-interactive mode in test environment
export WK_FORCE_NON_INTERACTIVE=1

# Setup isolated .claude directories
setup_claude_dirs() {
    mkdir -p "$TEST_DIR/.claude"
    export HOME="$TEST_DIR"
}

# Check if hooks are installed in a settings file
hooks_installed_in() {
    local file="$1"
    [ -f "$file" ] && grep -q '"hooks"' "$file"
}

# Get hook count from settings file
count_hooks_in() {
    local file="$1"
    if [ -f "$file" ]; then
        grep -c '"PreCompact"\|"SessionStart"' "$file" 2>/dev/null || echo 0
    else
        echo 0
    fi
}
