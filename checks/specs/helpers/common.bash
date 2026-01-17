#!/usr/bin/env bash

# Default test timeout (seconds) - can be overridden in individual test files
: "${BATS_TEST_TIMEOUT:=10}"

# Compute the path to bats helpers relative to this file
HELPERS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SPECS_DIR="$(dirname "$HELPERS_DIR")"

# Support both system and local bats libraries
if [[ -n "${BATS_LIB_PATH:-}" ]]; then
    load "${BATS_LIB_PATH}/bats-support/load"
    load "${BATS_LIB_PATH}/bats-assert/load"
else
    # Fallback to relative path for direct bats invocation
    load "$SPECS_DIR/bats/bats-support/load"
    load "$SPECS_DIR/bats/bats-assert/load"
fi

# Default to searching PATH if WK_BIN not set
export WK_BIN="${WK_BIN:-wk}"

# Create isolated test directory
setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"  # Isolate from user's home
}

# Cleanup after each test
teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

# ============================================================================
# FILE-LEVEL SETUP/TEARDOWN HELPERS
# ============================================================================
# Use these in test files that can share a single `wk init` across all tests.
# Tests using file-level setup should NOT define their own setup()/teardown().
#
# Example usage in a test file:
#
#   load '../helpers/common'
#
#   setup_file() {
#       file_setup           # Creates temp dir, sets HOME
#       init_project test    # Initialize once for all tests
#   }
#
#   teardown_file() {
#       file_teardown        # Cleanup temp dir
#   }
#
#   setup() {
#       test_setup           # Reset to file temp dir before each test
#   }
#
#   @test "my test" {
#       # No init needed - already done in setup_file
#       create_issue task "Test"
#       ...
#   }
# ============================================================================

# File-level setup: create shared temp directory
# Call this in setup_file() before init_project
file_setup() {
    export BATS_FILE_TMPDIR="$(mktemp -d)"
    cd "$BATS_FILE_TMPDIR" || exit 1
    export HOME="$BATS_FILE_TMPDIR"
}

# File-level teardown: cleanup shared temp directory
# Call this in teardown_file()
file_teardown() {
    cd / || exit 1
    if [ -n "$BATS_FILE_TMPDIR" ] && [ -d "$BATS_FILE_TMPDIR" ]; then
        rm -rf "$BATS_FILE_TMPDIR"
    fi
}

# Per-test setup when using file-level: reset to shared dir
# Call this in setup() when using file-level setup
test_setup() {
    cd "$BATS_FILE_TMPDIR" || exit 1
}

# Initialize project once (idempotent)
# Call this in setup_file() after file_setup()
# Usage: init_project_once [prefix]
init_project_once() {
    local prefix="${1:-test}"
    if [ ! -d ".wok" ]; then
        "$WK_BIN" init --prefix "$prefix" >/dev/null
    fi
}

# Helper: Initialize a test project
# Usage: init_project [prefix]
init_project() {
    local prefix="${1:-test}"
    run "$WK_BIN" init --prefix "$prefix"
    assert_success
}

# Helper: Create issue and capture ID (returns ID, call without 'run')
# Usage: id=$(create_issue task "My title" --tag foo)
create_issue() {
    local type="$1"
    local title="$2"
    shift 2
    local output
    output=$("$WK_BIN" new "$type" "$title" "$@" 2>&1)
    local status=$?
    if [ $status -ne 0 ]; then
        echo "create_issue failed: $output" >&2
        return 1
    fi
    # Extract ID from output (e.g., "Created test-a3f2" or "- [task] (todo) test-a3f2:")
    echo "$output" | grep -oE '[a-z]+-[a-z0-9]+(-[0-9]+)?' | head -1
}

# Helper: Get issue status
# Usage: status=$(get_status "$id")
# Returns status in parentheses format for test assertions: (todo), (in_progress), etc.
get_status() {
    local id="$1"
    local status
    status=$("$WK_BIN" show "$id" | grep -E '^Status:' | awk '{print $2}')
    echo "($status)"
}

# Helper: Get issue type
# Usage: type=$(get_type "$id")
get_type() {
    local id="$1"
    "$WK_BIN" show "$id" | grep -oE '\[epic\]|\[task\]|\[bug\]' | head -1
}

# Helper: Count issues in list output
# Usage: count=$(count_issues)
count_issues() {
    "$WK_BIN" list --all 2>/dev/null | grep -c '^\- \[' || echo 0
}

# Helper: Check if issue exists in list
# Usage: issue_in_list "$id"
issue_in_list() {
    local id="$1"
    "$WK_BIN" list --all 2>/dev/null | grep -q "$id"
}

# Helper: Check if issue is blocked
# Usage: is_blocked "$id"
is_blocked() {
    local id="$1"
    "$WK_BIN" list --blocked 2>/dev/null | grep -q "$id"
}

# Helper: Check if issue is ready (not blocked)
# Usage: is_ready "$id"
is_ready() {
    local id="$1"
    "$WK_BIN" list 2>/dev/null | grep -q "$id"
}
