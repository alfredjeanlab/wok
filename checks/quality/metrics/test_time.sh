#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Test Suite Run Time Measurement

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "Test Suite Run Time"
echo "==================="
echo ""

# Packages to test
PACKAGES=(
    "$ROOT_DIR/crates/cli"
    "$ROOT_DIR/crates/remote"
    "$ROOT_DIR/crates/core"
)

# Verify at least one package exists
found_pkg=false
for pkg in "${PACKAGES[@]}"; do
    [ -d "$pkg" ] && found_pkg=true
done
if [ "$found_pkg" = false ]; then
    echo "No packages found"
    exit 1
fi

# Cold run (after cleaning all packages)
echo "Cold run (after cargo clean)..."
for pkg in "${PACKAGES[@]}"; do
    [ -d "$pkg" ] && (cd "$pkg" && cargo clean --quiet 2>/dev/null || true)
done

start_time=$(date +%s.%N)
all_passed=true
for pkg in "${PACKAGES[@]}"; do
    if [ -d "$pkg" ]; then
        if ! (cd "$pkg" && cargo test --quiet 2>/dev/null); then
            all_passed=false
        fi
    fi
done
end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)

if [ "$all_passed" = true ]; then
    echo "Cold test time: ${duration}s"
else
    echo "Cold tests failed after ${duration}s"
fi

# Warm run (immediate re-run)
echo "Warm run (cached build)..."
start_time=$(date +%s.%N)
all_passed=true
for pkg in "${PACKAGES[@]}"; do
    if [ -d "$pkg" ]; then
        if ! (cd "$pkg" && cargo test --quiet 2>/dev/null); then
            all_passed=false
        fi
    fi
done
end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)

if [ "$all_passed" = true ]; then
    echo "Warm test time: ${duration}s"
else
    echo "Warm tests failed after ${duration}s"
fi

echo ""
echo "Note: Lower test time is better for fast feedback"
echo "Cold = includes compilation/build time"
echo "Warm = cached build, measures actual test execution"
echo "Packages: crates/cli, crates/remote, crates/core"
