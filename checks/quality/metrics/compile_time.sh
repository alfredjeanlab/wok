#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Compilation Time Measurement
# Measures both cold compile (from clean state) and incremental compile times

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

echo "Compilation Time Analysis"
echo "========================="
echo ""

# Packages to compile (crates/core first as it's a dependency)
PACKAGES=(
    "$ROOT_DIR/crates/core"
    "$ROOT_DIR/crates/cli"
    "$ROOT_DIR/crates/remote"
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

# Cold compile (clean first)
echo "Cold compile (after cargo clean)..."
for pkg in "${PACKAGES[@]}"; do
    [ -d "$pkg" ] && (cd "$pkg" && cargo clean 2>/dev/null || true)
done

start_time=$(date +%s.%N)
build_failed=false
for pkg in "${PACKAGES[@]}"; do
    if [ -d "$pkg" ]; then
        if ! (cd "$pkg" && cargo build --release 2>/dev/null); then
            build_failed=true
        fi
    fi
done
end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)

if [ "$build_failed" = false ]; then
    echo "Cold compile time: ${duration}s"
else
    echo "Build failed after ${duration}s"
fi

# Clean re-compile (clean and rebuild)
echo "Clean re-compile..."
for pkg in "${PACKAGES[@]}"; do
    [ -d "$pkg" ] && (cd "$pkg" && cargo clean 2>/dev/null || true)
done

start_time=$(date +%s.%N)
build_failed=false
for pkg in "${PACKAGES[@]}"; do
    if [ -d "$pkg" ]; then
        if ! (cd "$pkg" && cargo build --release 2>/dev/null); then
            build_failed=true
        fi
    fi
done
end_time=$(date +%s.%N)
duration=$(echo "$end_time - $start_time" | bc)

if [ "$build_failed" = false ]; then
    echo "Clean re-compile time: ${duration}s"
else
    echo "Build failed after ${duration}s"
fi

echo ""
echo "Note: Lower compile time is better for fast iteration"
echo "Cold compile = from clean state (no cache)"
echo "Clean re-compile = rebuild after cleaning build artifacts"
echo "Packages: crates/core, crates/cli, crates/remote"
