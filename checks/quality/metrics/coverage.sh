#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Code Coverage Collection

echo "Code Coverage Analysis"
echo "======================"
echo ""

# Clean workspace to avoid stale data
cargo llvm-cov clean --workspace

# Package definitions (order matters: remote before cli for transport tests)
PACKAGES=(
    "wk-core:crates/core"
    "wk-remote:crates/remote"
    "wk:crates/cli"
)

echo "=== Per-Package ==="
echo ""

for pkg_def in "${PACKAGES[@]}"; do
    IFS=':' read -r pkg_name pkg_path <<< "$pkg_def"

    echo "--- $pkg_path ---"
    cargo llvm-cov --package "$pkg_name" 2>&1 | grep -E "^(TOTAL|Filename|running [0-9]+ test|test result:)" || cargo llvm-cov --package "$pkg_name"
    echo ""
done

echo "=== Summary ==="
echo ""
echo "Note: Higher coverage is better"
echo "Aim for >85% line coverage"
echo "Packages: crates/cli, crates/remote, crates/core"
