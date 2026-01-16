#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Run agent-based code reviews
# Prepares context for Claude to review the wk implementation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

usage() {
    cat << EOF
Usage: $0 <review-type>

Prepare context for agent-based code reviews.

Arguments:
    review-type     idiomaticness or security

Examples:
    $0 idiomaticness
    $0 security

This script collects source files and prepares review context.
The actual review is performed by Claude using the prompt in:
    agents/<review-type>.md
EOF
}

if [ $# -lt 1 ]; then
    usage
    exit 1
fi

REVIEW_TYPE="$1"
CODE_PATH="$ROOT_DIR/crates/cli/src"

case "$REVIEW_TYPE" in
    idiomaticness|security) ;;
    *)
        echo "Unknown review type: $REVIEW_TYPE"
        echo "Valid options: idiomaticness, security"
        exit 1
        ;;
esac

PROMPT_FILE="$SCRIPT_DIR/${REVIEW_TYPE}.md"
if [ ! -f "$PROMPT_FILE" ]; then
    echo "Prompt file not found: $PROMPT_FILE"
    exit 1
fi

if [ ! -d "$CODE_PATH" ]; then
    echo "Code path not found: $CODE_PATH"
    exit 1
fi

# Collect source files (exclude test files)
FILES=$(find "$CODE_PATH" -name "*.rs" ! -name "*_tests.rs" | head -50)

FILE_COUNT=$(echo "$FILES" | wc -l | tr -d ' ')

echo "=== $REVIEW_TYPE Review: Rust Implementation ==="
echo ""
echo "Code path: $CODE_PATH"
echo "Files to review: $FILE_COUNT"
echo ""

# Prepare context file
CONTEXT_FILE="/tmp/review-context-rust-${REVIEW_TYPE}.md"

cat > "$CONTEXT_FILE" << EOF
# ${REVIEW_TYPE^} Review: Rust Implementation

## Review Instructions

Please review this codebase using the criteria in the prompt below.

## Review Prompt

$(cat "$PROMPT_FILE")

---

## Files to Review

$(echo "$FILES" | while read f; do echo "- ${f#$ROOT_DIR/}"; done)

---

## Source Code

EOF

# Append source files
for f in $FILES; do
    relative_path="${f#$ROOT_DIR/}"
    echo "### $relative_path" >> "$CONTEXT_FILE"
    echo '```rust' >> "$CONTEXT_FILE"
    cat "$f" >> "$CONTEXT_FILE"
    echo '```' >> "$CONTEXT_FILE"
    echo "" >> "$CONTEXT_FILE"
done

echo "Review context prepared: $CONTEXT_FILE"
echo ""
echo "To run the review with Claude:"
echo ""
echo "  Option 1 (Claude Code):"
echo "    claude -p \"Review this code for $REVIEW_TYPE\" < $CONTEXT_FILE"
echo ""
echo "  Option 2 (Interactive):"
echo "    Open $CONTEXT_FILE in your editor and paste to Claude"
echo ""
echo "  Option 3 (Direct with prompt):"
echo "    cat $CONTEXT_FILE | claude"
echo ""

# If claude command is available, offer to run it
if command -v claude &> /dev/null; then
    echo "Claude CLI detected. Run review now? (y/n)"
    read -r response
    if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
        echo "Running review..."
        claude < "$CONTEXT_FILE"
    fi
fi
