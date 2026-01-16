#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Code Coverage Collection

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "Code Coverage Analysis"
echo "======================"
echo ""

# Package definitions: name:path
# Note: crates/remote must come before crates/cli because cli's transport tests
# require the wk-remote binary from crates/remote to be built first
PACKAGES=(
    "crates/core:$ROOT_DIR/crates/core"
    "crates/remote:$ROOT_DIR/crates/remote"
    "crates/cli:$ROOT_DIR/crates/cli"
)

echo "=== Per-Package ==="
echo ""

for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name pkg_path <<< "$pkg_def"

    if [ ! -d "$pkg_path" ]; then
        continue
    fi

    echo "--- $pkg_name ---"
    cd "$pkg_path"

    # Use the project's coverage.sh script (uses native rustc coverage with LLVM tools)
    if [ -x "./coverage.sh" ]; then
        ./coverage.sh 2>&1 | grep -E "^(TOTAL|Filename|running [0-9]+ test|test result:)" || ./coverage.sh
    else
        echo "No coverage.sh found"
        echo "Fallback: running tests only (no coverage)..."
        if cargo test 2>&1 | tail -5; then
            echo "Tests passed"
        else
            echo "Tests failed"
        fi
    fi
    echo ""
done

echo "=== Summary ==="
echo ""
echo "Note: Higher coverage is better"
echo "Aim for >85% line coverage"
echo "Packages: crates/cli, crates/remote, crates/core"
