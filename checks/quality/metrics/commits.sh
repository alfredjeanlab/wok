#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Commit Metrics Script
# Analyzes git history by conventional commit type

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Default to last 7 days if no date range provided
if [[ "$OSTYPE" == "darwin"* ]]; then
    SINCE="${SINCE:-$(date -v-7d +%Y-%m-%d)}"
else
    SINCE="${SINCE:-$(date -d '-7 days' +%Y-%m-%d)}"
fi
UNTIL="${UNTIL:-$(date +%Y-%m-%d)}"

cd "$ROOT_DIR"

# Initialize counts (Bash 3.2 compatible - no associative arrays)
total=0
feat=0
fix=0
chore=0
refactor=0
docs=0
other=0

# Count commits by conventional commit type
while IFS= read -r subject; do
    [ -z "$subject" ] && continue
    total=$((total + 1))
    case "$subject" in
        feat:*|feat\(*|Add:*|add:*)
            feat=$((feat + 1))
            ;;
        fix:*|fix\(*|Fix:*)
            fix=$((fix + 1))
            ;;
        chore:*|chore\(*)
            chore=$((chore + 1))
            ;;
        refactor:*|refactor\(*)
            refactor=$((refactor + 1))
            ;;
        docs:*|docs\(*)
            docs=$((docs + 1))
            ;;
        *)
            other=$((other + 1))
            ;;
    esac
done < <(git log --since="$SINCE" --until="$UNTIL" --format="%s" 2>/dev/null || true)

echo "total: $total"
echo "feat: $feat"
echo "fix: $fix"
echo "chore: $chore"
echo "refactor: $refactor"
echo "docs: $docs"
echo "other: $other"
echo "since: $SINCE"
echo "until: $UNTIL"
