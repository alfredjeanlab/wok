#!/usr/bin/env bash
# Append current benchmarks to history.json for trend tracking
set -euo pipefail

HISTORY_FILE="docs/reports/benchmarks/history.json"
LATEST_FILE="checks/benchmarks/results/latest.json"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

if [[ -f "$HISTORY_FILE" ]]; then
    # Append to existing history (keep last 52 weeks)
    jq --slurpfile new "$LATEST_FILE" \
       '. + [($new[0] + {timestamp: "'"$TIMESTAMP"'"})] | .[-52:]' \
       "$HISTORY_FILE" > "${HISTORY_FILE}.tmp"
    mv "${HISTORY_FILE}.tmp" "$HISTORY_FILE"
else
    # Initialize history
    jq '{timestamp: "'"$TIMESTAMP"'"} + .' "$LATEST_FILE" | jq -s '.' > "$HISTORY_FILE"
fi
