#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Alfred Jean LLC
set -euo pipefail

# Issue Metrics Script
# Analyzes .beads/issues.jsonl for work tracking

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

ISSUES_FILE="$ROOT_DIR/.beads/issues.jsonl"

# Default to last 7 days if no date range provided
if [[ "$OSTYPE" == "darwin"* ]]; then
    SINCE="${SINCE:-$(date -v-7d +%Y-%m-%d)}"
else
    SINCE="${SINCE:-$(date -d '-7 days' +%Y-%m-%d)}"
fi

# Output zeros if issues.jsonl doesn't exist or is empty
if [[ ! -f "$ISSUES_FILE" ]] || [[ ! -s "$ISSUES_FILE" ]]; then
    echo "bugs_open: 0"
    echo "bugs_closed: 0"
    echo "bugs_fixed: 0"
    echo "tasks_open: 0"
    echo "tasks_closed: 0"
    echo "chores_open: 0"
    echo "chores_closed: 0"
    echo "epics_open: 0"
    echo "epics_done: 0"
    echo "features_open: 0"
    echo "features_closed: 0"
    echo "since: $SINCE"
    exit 0
fi

# Parse issues.jsonl with jq
# Status values: open, in_progress, closed, fixed (for bugs), done (for epics)

# Bugs
bugs_open=$(jq -s '[.[] | select(.issue_type == "bug" and (.status == "open" or .status == "in_progress"))] | length' "$ISSUES_FILE")
bugs_closed=$(jq -s "[.[] | select(.issue_type == \"bug\" and .status == \"closed\" and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")
bugs_fixed=$(jq -s "[.[] | select(.issue_type == \"bug\" and .status == \"fixed\" and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")

# Tasks (generic work items)
tasks_open=$(jq -s '[.[] | select(.issue_type == "task" and (.status == "open" or .status == "in_progress"))] | length' "$ISSUES_FILE")
tasks_closed=$(jq -s "[.[] | select(.issue_type == \"task\" and .status == \"closed\" and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")

# Chores (maintenance work)
chores_open=$(jq -s '[.[] | select(.issue_type == "chore" and (.status == "open" or .status == "in_progress"))] | length' "$ISSUES_FILE")
chores_closed=$(jq -s "[.[] | select(.issue_type == \"chore\" and .status == \"closed\" and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")

# Epics
epics_open=$(jq -s '[.[] | select(.issue_type == "epic" and (.status == "open" or .status == "in_progress"))] | length' "$ISSUES_FILE")
epics_done=$(jq -s "[.[] | select(.issue_type == \"epic\" and (.status == \"done\" or .status == \"closed\") and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")

# Features
features_open=$(jq -s '[.[] | select(.issue_type == "feature" and (.status == "open" or .status == "in_progress"))] | length' "$ISSUES_FILE")
features_closed=$(jq -s "[.[] | select(.issue_type == \"feature\" and .status == \"closed\" and (.closed_at >= \"$SINCE\" or .closed_at == null))] | length" "$ISSUES_FILE")

echo "bugs_open: $bugs_open"
echo "bugs_closed: $bugs_closed"
echo "bugs_fixed: $bugs_fixed"
echo "tasks_open: $tasks_open"
echo "tasks_closed: $tasks_closed"
echo "chores_open: $chores_open"
echo "chores_closed: $chores_closed"
echo "epics_open: $epics_open"
echo "epics_done: $epics_done"
echo "features_open: $features_open"
echo "features_closed: $features_closed"
echo "since: $SINCE"
