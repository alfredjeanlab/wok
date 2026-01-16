#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
# Primary entry point for stress tests - runs in Docker container
#
# Docker provides HARD resource limits that cannot be bypassed, unlike ulimit.
# This is the RECOMMENDED way to run stress tests.
#
# Usage:
#   WK_BIN=./crates/cli/target/release/wk ./checks/stress/docker-run.sh
#   WK_BIN=./wk STRESS_MEMORY=4g ./checks/stress/docker-run.sh massive_db 50000

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Required: path to wk binary
WK_BIN="${WK_BIN:?Set WK_BIN to path of wk binary}"

# Docker resource limits (adjustable)
STRESS_MEMORY="${STRESS_MEMORY:-2g}"
STRESS_DISK="${STRESS_DISK:-5g}"
STRESS_CPUS="${STRESS_CPUS:-2}"
STRESS_PIDS="${STRESS_PIDS:-100}"

# Docker image
STRESS_IMAGE="${STRESS_IMAGE:-ubuntu:22.04}"

# Resolve WK_BIN to absolute path
if [[ "$WK_BIN" != /* ]]; then
    WK_BIN="$(cd "$(dirname "$WK_BIN")" && pwd)/$(basename "$WK_BIN")"
fi

# Verify binary exists
if [ ! -f "$WK_BIN" ]; then
    echo "ERROR: wk binary not found: $WK_BIN" >&2
    exit 1
fi

if [ ! -x "$WK_BIN" ]; then
    echo "ERROR: wk binary not executable: $WK_BIN" >&2
    exit 1
fi

# Check if Docker is available
if ! command -v docker &>/dev/null; then
    echo "WARNING: Docker not available, falling back to native execution" >&2
    echo "Native execution uses ulimit (advisory limits) - less safe than Docker" >&2
    echo ""
    exec "$SCRIPT_DIR/run.sh" "$@"
fi

# Check if Docker daemon is running
if ! docker info &>/dev/null; then
    echo "WARNING: Docker daemon not running, falling back to native execution" >&2
    echo "Native execution uses ulimit (advisory limits) - less safe than Docker" >&2
    echo ""
    exec "$SCRIPT_DIR/run.sh" "$@"
fi

echo "=== Running stress tests in Docker container ==="
echo "Binary: $WK_BIN"
echo "Memory limit: $STRESS_MEMORY"
echo "Disk limit: $STRESS_DISK"
echo "CPU limit: $STRESS_CPUS"
echo "PID limit: $STRESS_PIDS"
echo "Image: $STRESS_IMAGE"
echo ""

# Build command line for container
CONTAINER_ARGS=("$@")

# Run in Docker with resource limits
docker run --rm \
    --memory="$STRESS_MEMORY" \
    --memory-swap="$STRESS_MEMORY" \
    --cpus="$STRESS_CPUS" \
    --pids-limit="$STRESS_PIDS" \
    --tmpfs "/tmp:size=$STRESS_DISK" \
    --ulimit nofile=1024:1024 \
    --network=none \
    -e WK_BIN=/usr/local/bin/wk \
    -e STRESS_CONTAINERIZED=1 \
    -e STRESS_MAX_MEMORY_MB="${STRESS_MAX_MEMORY_MB:-2048}" \
    -e STRESS_MAX_DISK_MB="${STRESS_MAX_DISK_MB:-5120}" \
    -e STRESS_TIMEOUT_SEC="${STRESS_TIMEOUT_SEC:-300}" \
    -e STRESS_SKIP_DANGEROUS="${STRESS_SKIP_DANGEROUS:-1}" \
    -v "$WK_BIN:/usr/local/bin/wk:ro" \
    -v "$SCRIPT_DIR:/stress:ro" \
    -w /tmp \
    "$STRESS_IMAGE" \
    /bin/bash -c "
        # Install dependencies
        apt-get update -qq && apt-get install -qq -y sqlite3 bc procps >/dev/null 2>&1 || true

        # Run stress tests
        /stress/run.sh ${CONTAINER_ARGS[*]:-all}
    "

echo ""
echo "=== Docker container exited ==="
