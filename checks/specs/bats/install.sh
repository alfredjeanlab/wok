#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Versions to install (can be updated as needed)
BATS_CORE_VERSION="v1.11.0"
BATS_SUPPORT_VERSION="v0.3.0"
BATS_ASSERT_VERSION="v2.1.0"

install_repo() {
    local name="$1"
    local version="$2"
    local dir="$SCRIPT_DIR/$name"

    if [ -d "$dir" ] && [ -f "$dir/.installed" ]; then
        echo "$name already installed"
        return 0
    fi

    echo "Installing $name $version..."
    rm -rf "$dir"

    # Clone specific tag (shallow)
    git clone --depth 1 --branch "$version" \
        "https://github.com/bats-core/$name.git" "$dir"

    # Mark as installed
    echo "$version" > "$dir/.installed"
    echo "$name installed successfully"
}

echo "Installing BATS libraries to $SCRIPT_DIR"
echo ""

install_repo "bats-core" "$BATS_CORE_VERSION"
install_repo "bats-support" "$BATS_SUPPORT_VERSION"
install_repo "bats-assert" "$BATS_ASSERT_VERSION"

echo ""
echo "BATS installation complete!"
