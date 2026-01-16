#!/bin/bash
# Native rustc coverage instrumentation
# Usage: ./coverage.sh [--html]
set -e

LLVM_BIN="/opt/homebrew/opt/llvm/bin"
PROFDATA="$LLVM_BIN/llvm-profdata"
LLVM_COV="$LLVM_BIN/llvm-cov"

# Find workspace root (look for Cargo.toml with [workspace])
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_ROOT="$SCRIPT_DIR/../.."
TARGET_DIR="$WORKSPACE_ROOT/target"

# Clean previous coverage data
rm -rf "$TARGET_DIR/coverage"
mkdir -p "$TARGET_DIR/coverage"

# Build and run tests with coverage
RUSTFLAGS="-C instrument-coverage" \
LLVM_PROFILE_FILE="$TARGET_DIR/coverage/test_%m_%p.profraw" \
    cargo test --tests 2>&1

# Merge profile data
$PROFDATA merge -sparse "$TARGET_DIR/coverage"/*.profraw -o "$TARGET_DIR/coverage/tests.profdata"

# Find test binaries (filter to this crate only)
OBJECTS=$(find "$TARGET_DIR/debug/deps" -maxdepth 1 -type f -perm +111 -name "wk_remote-*" ! -name "*.d" ! -name "*.so" | xargs -I{} echo "--object={}")

if [[ "$1" == "--html" ]]; then
    $LLVM_COV show $OBJECTS \
        --instr-profile="$TARGET_DIR/coverage/tests.profdata" \
        --format=html \
        --output-dir="$TARGET_DIR/coverage/html" \
        --ignore-filename-regex='/.cargo/|/rustc/'
    echo "HTML report: $TARGET_DIR/coverage/html/index.html"
else
    $LLVM_COV report $OBJECTS \
        --instr-profile="$TARGET_DIR/coverage/tests.profdata" \
        --ignore-filename-regex='/.cargo/|/rustc/'
fi
