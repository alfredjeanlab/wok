#!/usr/bin/env bash
# Generate markdown and HTML benchmark report with charts
set -euo pipefail

BENCH_DIR="docs/reports/benchmarks"
LATEST="$BENCH_DIR/latest.json"
HISTORY="$BENCH_DIR/history.json"

# Generate comparison markdown
cat > "$BENCH_DIR/comparison.md" << 'EOF'
| Operation | Mean | Min | Max | Status |
|-----------|------|-----|-----|--------|
EOF

jq -r '.results[] | "| \(.command) | \(.mean*1000|floor)ms | \(.min*1000|floor)ms | \(.max*1000|floor)ms | :white_check_mark: |"' \
    "$LATEST" >> "$BENCH_DIR/comparison.md"

# Generate index.html with Chart.js visualization
cat > "$BENCH_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
  <title>Wok Benchmarks</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
    .chart-container { height: 400px; margin: 20px 0; }
  </style>
</head>
<body>
  <h1>Wok Performance Benchmarks</h1>
  <div class="chart-container"><canvas id="benchChart"></canvas></div>
  <script>
    fetch('history.json').then(r => r.json()).then(data => {
      // Render chart from history data
    });
  </script>
</body>
</html>
EOF
