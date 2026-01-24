#!/bin/bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# install.sh - Install wok from GitHub Releases
#
# Usage:
#   curl -fsSL https://github.com/alfredjeanlab/wok/releases/latest/download/install.sh | bash
#
# Environment variables:
#   WOK_VERSION - Version to install (default: latest)
#   WOK_INSTALL - Installation directory (default: ~/.local/bin)

set -e

WOK_VERSION="${WOK_VERSION:-latest}"
WOK_INSTALL="${WOK_INSTALL:-$HOME/.local/bin}"
WOK_REPO="alfredjeanlab/wok"
GITHUB_API="https://api.github.com"
GITHUB_RELEASES="https://github.com/${WOK_REPO}/releases"

# Colors (disabled if not a terminal)
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  NC='\033[0m' # No Color
else
  RED=''
  GREEN=''
  YELLOW=''
  NC=''
fi

info() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}Warning:${NC} $1"; }
error() { echo -e "${RED}Error:${NC} $1" >&2; exit 1; }

# Check for required commands
for cmd in curl tar; do
  if ! command -v "$cmd" &> /dev/null; then
    error "$cmd is required but not installed"
  fi
done

# Detect platform
detect_platform() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux)  os="linux" ;;
    Darwin) os="macos" ;;
    *)      error "Unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64)  arch="x86_64" ;;
    aarch64) arch="aarch64" ;;
    arm64)   arch="aarch64" ;;
    *)       error "Unsupported architecture: $arch" ;;
  esac

  echo "wok-${os}-${arch}"
}

PLATFORM=$(detect_platform)
info "Detected platform: $PLATFORM"

# Resolve "latest" to actual version
if [ "$WOK_VERSION" = "latest" ]; then
  info "Fetching latest version..."
  WOK_VERSION=$(curl -fsSL "${GITHUB_API}/repos/${WOK_REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
  if [ -z "$WOK_VERSION" ]; then
    error "Could not determine latest version. Check your internet connection."
  fi
fi

info "Installing wok v${WOK_VERSION}..."

# Create temp directory
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# Download tarball and checksum
TARBALL="${PLATFORM}.tar.gz"
CHECKSUM="${TARBALL}.sha256"
DOWNLOAD_URL="${GITHUB_RELEASES}/download/v${WOK_VERSION}"

info "Downloading ${TARBALL}..."
if ! curl -fsSL "${DOWNLOAD_URL}/${TARBALL}" -o "${TMPDIR}/${TARBALL}"; then
  error "Failed to download ${TARBALL}. Version v${WOK_VERSION} may not exist."
fi

info "Downloading checksum..."
if ! curl -fsSL "${DOWNLOAD_URL}/${CHECKSUM}" -o "${TMPDIR}/${CHECKSUM}"; then
  error "Failed to download checksum file"
fi

# Verify checksum
info "Verifying checksum..."
cd "$TMPDIR"
if command -v sha256sum &> /dev/null; then
  sha256sum -c "${CHECKSUM}" --quiet || error "Checksum verification failed!"
elif command -v shasum &> /dev/null; then
  shasum -a 256 -c "${CHECKSUM}" --quiet || error "Checksum verification failed!"
else
  warn "No sha256sum or shasum available, skipping checksum verification"
fi

# Extract tarball
info "Extracting..."
tar -xzf "${TARBALL}"

# Install binaries
mkdir -p "$WOK_INSTALL"
info "Installing to ${WOK_INSTALL}..."
cp wk "$WOK_INSTALL/wok"
cp wk-remote "$WOK_INSTALL/wk-remote"
chmod +x "$WOK_INSTALL/wok" "$WOK_INSTALL/wk-remote"

# Create wk symlink for convenience
ln -sf wok "$WOK_INSTALL/wk"

echo ""
info "wok v${WOK_VERSION} installed successfully!"

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$WOK_INSTALL:"* ]]; then
  echo ""
  warn "$WOK_INSTALL is not in your PATH"
  echo "Add this to your shell profile:"
  echo ""
  echo "  export PATH=\"$WOK_INSTALL:\$PATH\""
fi

echo ""
echo "To get started in a project:"
echo "  cd /path/to/your/project"
echo "  wok init"
