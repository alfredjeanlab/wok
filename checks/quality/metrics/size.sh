#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Binary Size measurement

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

get_size() {
    local path="$1"
    if [ -f "$path" ]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            stat -f%z "$path"
        else
            stat -c%s "$path"
        fi
    else
        echo "0"
    fi
}

format_size() {
    local bytes="$1"
    if [ "$bytes" -eq 0 ]; then
        echo "N/A"
    elif [ "$bytes" -lt 1024 ]; then
        echo "${bytes} B"
    elif [ "$bytes" -lt 1048576 ]; then
        echo "$(( bytes / 1024 )) KB"
    else
        echo "$(echo "scale=1; $bytes / 1048576" | bc) MB"
    fi
}

echo "Binary Size Analysis"
echo "===================="
echo ""

RUST_RELEASE="$ROOT_DIR/crates/cli/target/release/wk"

# Build if not exists
if [ ! -f "$RUST_RELEASE" ]; then
    echo "Building release binary..."
    (cd "$ROOT_DIR/crates/cli" && cargo build --release) > /dev/null 2>&1 || {
        echo "Build failed"
        exit 1
    }
fi

if [ -f "$RUST_RELEASE" ]; then
    local_size=$(get_size "$RUST_RELEASE")
    echo "Release binary: $(format_size "$local_size") ($local_size bytes)"

    # Check if it's a binary we can strip
    if file "$RUST_RELEASE" 2>/dev/null | grep -q "executable\|Mach-O"; then
        stripped=$(mktemp)
        cp "$RUST_RELEASE" "$stripped"
        if strip "$stripped" 2>/dev/null; then
            stripped_size=$(get_size "$stripped")
            echo "Stripped:       $(format_size "$stripped_size") ($stripped_size bytes)"
        fi
        rm -f "$stripped"
    fi
else
    echo "Binary not found: $RUST_RELEASE"
fi

echo ""
echo "Note: Lower is better"
