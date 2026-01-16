#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BATS="${SCRIPT_DIR}/bats/bats-core/bin/bats"

# Default executables
export WK_BIN="${WK_BIN:-wk}"
export WK_REMOTE_BIN="${WK_REMOTE_BIN:-wk-remote}"

# Parse arguments
PARALLEL_JOBS=""
BATS_ARGS=()
SUITE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --jobs|-j)
            PARALLEL_JOBS="$2"
            shift 2
            ;;
        --jobs=*)
            PARALLEL_JOBS="${1#*=}"
            shift
            ;;
        -j*)
            PARALLEL_JOBS="${1#-j}"
            shift
            ;;
        cli|cli/)
            SUITE="cli"
            shift
            ;;
        remote|remote/)
            SUITE="remote"
            shift
            ;;
        *)
            BATS_ARGS+=("$1")
            shift
            ;;
    esac
done

# Verify BATS is available
if [ ! -x "$BATS" ]; then
    echo "Error: BATS not found at $BATS" >&2
    exit 1
fi

# Verify wk executable exists
if ! command -v "$WK_BIN" &> /dev/null; then
    # Try as a path
    if [ ! -x "$WK_BIN" ] && [ ! -f "$WK_BIN" ]; then
        echo "Error: Cannot find wk executable: $WK_BIN" >&2
        echo "Set WK_BIN environment variable to the path of your wk binary" >&2
        exit 1
    fi
fi

# For remote tests, verify wk-remote executable exists
if [ "$SUITE" = "remote" ] || [ -z "$SUITE" ]; then
    if ! command -v "$WK_REMOTE_BIN" &> /dev/null; then
        # Try as a path
        if [ ! -x "$WK_REMOTE_BIN" ] && [ ! -f "$WK_REMOTE_BIN" ]; then
            if [ "$SUITE" = "remote" ]; then
                echo "Error: Cannot find wk-remote executable: $WK_REMOTE_BIN" >&2
                echo "Set WK_REMOTE_BIN environment variable to the path of your wk-remote binary" >&2
                exit 1
            else
                # Running all tests but wk-remote not available - skip remote tests
                echo "Note: wk-remote not found, skipping remote tests" >&2
                SUITE="cli"
            fi
        fi
    fi
fi

# Auto-detect parallel support if --jobs not specified
# BATS requires GNU parallel or rush for the -j flag
if [ -z "$PARALLEL_JOBS" ]; then
    PARALLEL_JOBS=1
    if command -v parallel &> /dev/null; then
        PARALLEL_JOBS=4
    elif command -v rush &> /dev/null; then
        PARALLEL_JOBS=4
    fi
fi

echo "Testing: $WK_BIN"
echo "Version: $("$WK_BIN" --version 2>/dev/null || echo 'unknown')"
if [ "$SUITE" != "cli" ]; then
    echo "Remote: $WK_REMOTE_BIN"
fi
echo "Parallel jobs: $PARALLEL_JOBS"
echo ""

# Run tests
if [ ${#BATS_ARGS[@]} -eq 0 ]; then
    # Run tests based on suite selection
    case "$SUITE" in
        cli)
            "$BATS" -j "$PARALLEL_JOBS" --recursive \
                "$SCRIPT_DIR/cli/unit" \
                "$SCRIPT_DIR/cli/integration" \
                "$SCRIPT_DIR/cli/edge_cases"
            ;;
        remote)
            "$BATS" -j "$PARALLEL_JOBS" --recursive \
                "$SCRIPT_DIR/remote/unit" \
                "$SCRIPT_DIR/remote/integration" \
                "$SCRIPT_DIR/remote/edge_cases"
            ;;
        *)
            # Run all tests
            "$BATS" -j "$PARALLEL_JOBS" --recursive \
                "$SCRIPT_DIR/cli/unit" \
                "$SCRIPT_DIR/cli/integration" \
                "$SCRIPT_DIR/cli/edge_cases" \
                "$SCRIPT_DIR/remote/unit" \
                "$SCRIPT_DIR/remote/integration" \
                "$SCRIPT_DIR/remote/edge_cases"
            ;;
    esac
else
    # Run specified test files or options with parallel execution
    "$BATS" -j "$PARALLEL_JOBS" "${BATS_ARGS[@]}"
fi
