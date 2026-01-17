# GitHub CI/CD Implementation Plan

**Root Feature:** `wok-3cc4`

## Overview

Complete the GitHub CI/CD infrastructure with report dashboards for quality metrics and benchmarks. The core workflows are already implemented; this plan addresses remaining gaps:

- Missing `checks/benchmarks/compare.sh` script for PR regression detection
- HTML report dashboards for GitHub Pages
- Complete Chart.js visualization code (currently a stub)
- Directory structure for report deployment

## Current State

**Implemented (workflows in `.github/workflows/`):**
- `ci.yml` - Build, test, lint, format, audit
- `specs.yml` - BATS specification tests (cli/remote matrix)
- `coverage.yml` - Code coverage with threshold enforcement
- `quality.yml` - Quality metrics with Pages deployment
- `benchmarks.yml` - Performance benchmarks with PR comments
- `release.yml` - Multi-platform release automation

**Implemented (supporting scripts):**
- `scripts/append-quality-history.sh` - Quality history tracking
- `scripts/append-bench-history.sh` - Benchmark history tracking
- `scripts/generate-bench-report.sh` - Report generation (partial)
- `checks/quality/evaluate.sh` - Quality metrics collection
- `checks/benchmarks/run.sh` - Benchmark execution

**Missing:**
- `checks/benchmarks/compare.sh` - Referenced in workflow but doesn't exist
- `docs/reports/` - Directory structure doesn't exist
- `docs/reports/index.html` - Landing dashboard
- `docs/reports/quality/index.html` - Quality dashboard with charts
- Complete Chart.js code in benchmark dashboard

## Project Structure

```
wok/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml              # ✓ Implemented
│   │   ├── specs.yml           # ✓ Implemented
│   │   ├── quality.yml         # ✓ Implemented
│   │   ├── benchmarks.yml      # ✓ Implemented (needs compare.sh)
│   │   ├── coverage.yml        # ✓ Implemented
│   │   └── release.yml         # ✓ Implemented
│   └── thresholds.json         # ✓ Implemented
├── checks/
│   └── benchmarks/
│       └── compare.sh          # NEW: Regression detection
├── scripts/
│   ├── append-quality-history.sh   # ✓ Implemented
│   ├── append-bench-history.sh     # ✓ Implemented
│   └── generate-bench-report.sh    # UPDATE: Complete Chart.js
└── docs/
    └── reports/                    # NEW: Full structure
        ├── .nojekyll
        ├── index.html              # NEW: Landing dashboard
        ├── quality/
        │   └── index.html          # NEW: Quality dashboard
        └── benchmarks/
            └── index.html          # UPDATE: Complete viz
```

## Dependencies

**Already configured in workflows:**
- `actions/checkout@v4`, `actions/cache@v4`, `actions/upload-artifact@v4`
- `actions/deploy-pages@v4`, `actions/configure-pages@v4`
- `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`
- `taiki-e/install-action@v2` (cargo-audit, cargo-llvm-cov)
- `codecov/codecov-action@v4`
- `softprops/action-gh-release@v1`
- `actions/github-script@v7`

**External:**
- GitHub Pages (report hosting)
- hyperfine v1.18.0 (benchmarks)
- Chart.js CDN (visualizations)
- BATS with bats-assert, bats-support

## Implementation Phases

### Phase 1: Benchmark Comparison Script

Create the missing script referenced by `benchmarks.yml` for PR regression detection.

**File:** `checks/benchmarks/compare.sh`

```bash
#!/usr/bin/env bash
# Compare benchmark results against baseline for regression detection
set -euo pipefail

CURRENT="${1:?Usage: compare.sh <current.json> <baseline.json> [threshold%]}"
BASELINE="${2:?Usage: compare.sh <current.json> <baseline.json> [threshold%]}"
THRESHOLD="${3:-10}"  # Default 10% regression threshold

if [[ ! -f "$BASELINE" ]]; then
    echo "::notice::No baseline found at $BASELINE, skipping comparison"
    exit 0
fi

if [[ ! -f "$CURRENT" ]]; then
    echo "::error::Current results not found at $CURRENT"
    exit 1
fi

echo "Comparing benchmarks (threshold: ${THRESHOLD}% regression)"
echo "Current: $CURRENT"
echo "Baseline: $BASELINE"
echo

REGRESSIONS=0

# Compare each benchmark result
while IFS= read -r line; do
    cmd=$(echo "$line" | cut -d'|' -f1)
    current_mean=$(echo "$line" | cut -d'|' -f2)

    baseline_mean=$(jq -r --arg cmd "$cmd" \
        '.results[] | select(.command == $cmd) | .mean' "$BASELINE" 2>/dev/null)

    if [[ -z "$baseline_mean" || "$baseline_mean" == "null" ]]; then
        echo "  $cmd: NEW (no baseline)"
        continue
    fi

    # Calculate percentage change
    pct_change=$(echo "scale=2; (($current_mean - $baseline_mean) / $baseline_mean) * 100" | bc)
    current_ms=$(echo "scale=1; $current_mean * 1000" | bc)
    baseline_ms=$(echo "scale=1; $baseline_mean * 1000" | bc)

    if (( $(echo "$pct_change > $THRESHOLD" | bc -l) )); then
        echo "::warning file=checks/benchmarks/compare.sh::Regression: $cmd slowed by ${pct_change}% (${baseline_ms}ms -> ${current_ms}ms)"
        REGRESSIONS=$((REGRESSIONS + 1))
    elif (( $(echo "$pct_change < -$THRESHOLD" | bc -l) )); then
        echo "  $cmd: FASTER by ${pct_change#-}% (${baseline_ms}ms -> ${current_ms}ms)"
    else
        echo "  $cmd: ${current_ms}ms (${pct_change}% change)"
    fi
done < <(jq -r '.results[] | "\(.command)|\(.mean)"' "$CURRENT")

echo
if [[ $REGRESSIONS -gt 0 ]]; then
    echo "::warning::Found $REGRESSIONS regression(s) exceeding ${THRESHOLD}% threshold"
    # Don't fail the build, just warn
fi

exit 0
```

**Verification:**
```bash
chmod +x checks/benchmarks/compare.sh
# Test with sample data
echo '{"results":[{"command":"list_default","mean":0.045}]}' > /tmp/current.json
echo '{"results":[{"command":"list_default","mean":0.040}]}' > /tmp/baseline.json
./checks/benchmarks/compare.sh /tmp/current.json /tmp/baseline.json
# Should show 12.5% regression warning
```

---

### Phase 2: Report Directory Structure

Create the `docs/reports/` structure with initial files for GitHub Pages.

**Files to create:**

1. `docs/reports/.nojekyll` - Empty file to disable Jekyll processing
2. `docs/reports/.gitkeep` - Ensure directory is tracked
3. `docs/reports/quality/.gitkeep` - Quality reports directory
4. `docs/reports/benchmarks/.gitkeep` - Benchmark reports directory

**Verification:**
```bash
ls -la docs/reports/
# Should show .nojekyll and subdirectories
```

---

### Phase 3: Landing Dashboard

Create the main report landing page.

**File:** `docs/reports/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Wok Reports</title>
  <style>
    :root {
      --bg: #0d1117;
      --surface: #161b22;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --accent: #58a6ff;
      --green: #3fb950;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
      min-height: 100vh;
      padding: 48px 24px;
    }
    .container { max-width: 960px; margin: 0 auto; }
    h1 { font-size: 2rem; font-weight: 600; margin-bottom: 8px; }
    .subtitle { color: var(--text-muted); margin-bottom: 32px; }
    .cards {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: 16px;
    }
    .card {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 24px;
      text-decoration: none;
      color: inherit;
      transition: border-color 0.15s ease;
    }
    .card:hover { border-color: var(--accent); }
    .card-icon { font-size: 32px; margin-bottom: 16px; }
    .card h2 { font-size: 1.25rem; font-weight: 600; margin-bottom: 8px; }
    .card p { color: var(--text-muted); font-size: 0.875rem; margin-bottom: 16px; }
    .card-meta {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 0.75rem;
      color: var(--text-muted);
    }
    .status-dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--green);
    }
    footer {
      margin-top: 48px;
      padding-top: 24px;
      border-top: 1px solid var(--border);
      text-align: center;
      color: var(--text-muted);
      font-size: 0.875rem;
    }
    footer a { color: var(--accent); text-decoration: none; }
    footer a:hover { text-decoration: underline; }
  </style>
</head>
<body>
  <div class="container">
    <h1>Wok Reports</h1>
    <p class="subtitle">Automated CI/CD metrics and performance tracking</p>

    <div class="cards">
      <a href="quality/" class="card">
        <div class="card-icon">📊</div>
        <h2>Quality Metrics</h2>
        <p>Code coverage, binary size, compile time, escape hatches, and code health metrics tracked over time.</p>
        <div class="card-meta">
          <span class="status-dot"></span>
          <span>Updated on push to main</span>
        </div>
      </a>

      <a href="benchmarks/" class="card">
        <div class="card-icon">⚡</div>
        <h2>Benchmarks</h2>
        <p>Performance measurements for list operations, filters, and output formats with regression detection.</p>
        <div class="card-meta">
          <span class="status-dot"></span>
          <span>Updated on every commit</span>
        </div>
      </a>
    </div>

    <footer>
      <p>Generated by GitHub Actions</p>
    </footer>
  </div>
</body>
</html>
```

**Verification:** Open `docs/reports/index.html` in browser, verify styling and links.

---

### Phase 4: Quality Dashboard

Create the quality metrics dashboard with Chart.js trend visualization.

**File:** `docs/reports/quality/index.html`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Quality Metrics - Wok</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    :root {
      --bg: #0d1117;
      --surface: #161b22;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --accent: #58a6ff;
      --green: #3fb950;
      --red: #f85149;
      --yellow: #d29922;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
      padding: 24px;
    }
    .container { max-width: 1200px; margin: 0 auto; }
    header { display: flex; align-items: center; gap: 16px; margin-bottom: 32px; }
    .back {
      color: var(--text-muted);
      text-decoration: none;
      font-size: 1.5rem;
      line-height: 1;
    }
    .back:hover { color: var(--text); }
    h1 { font-size: 1.5rem; font-weight: 600; }
    .metrics-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 12px;
      margin-bottom: 32px;
    }
    .metric {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 16px;
    }
    .metric-label { font-size: 0.75rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
    .metric-value { font-size: 1.75rem; font-weight: 600; margin: 4px 0; }
    .metric-status { font-size: 0.75rem; font-weight: 500; }
    .pass { color: var(--green); }
    .warn { color: var(--yellow); }
    .fail { color: var(--red); }
    .chart-section {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 20px;
      margin-bottom: 24px;
    }
    .chart-section h2 { font-size: 1rem; font-weight: 600; margin-bottom: 16px; }
    .chart-container { position: relative; height: 250px; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 12px; text-align: left; border-bottom: 1px solid var(--border); }
    th { color: var(--text-muted); font-weight: 500; font-size: 0.75rem; text-transform: uppercase; }
    .loading { color: var(--text-muted); font-style: italic; padding: 24px; text-align: center; }
    .no-data { background: var(--surface); border: 1px solid var(--border); border-radius: 6px; padding: 48px; text-align: center; }
    .no-data p { color: var(--text-muted); }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <a href="../" class="back" title="Back to reports">←</a>
      <h1>Quality Metrics</h1>
    </header>

    <div id="content">
      <div class="loading">Loading metrics...</div>
    </div>
  </div>

  <script>
    const THRESHOLDS = {
      coverage: { cli: { lines: 85, functions: 81 }, core: { lines: 90, functions: 86 }, remote: { lines: 44, functions: 39 } },
      escapes: 3,
      binarySize: 4 * 1024 * 1024
    };

    async function loadData() {
      try {
        const [latest, history] = await Promise.all([
          fetch('latest.json').then(r => r.ok ? r.json() : null).catch(() => null),
          fetch('history.json').then(r => r.ok ? r.json() : []).catch(() => [])
        ]);

        if (!latest) {
          showNoData();
          return;
        }

        render(latest, history);
      } catch (e) {
        console.error('Failed to load data:', e);
        showNoData();
      }
    }

    function showNoData() {
      document.getElementById('content').innerHTML = `
        <div class="no-data">
          <h2>No Data Available</h2>
          <p>Quality metrics will appear here after the first CI run on the main branch.</p>
        </div>
      `;
    }

    function render(latest, history) {
      const totalCoverage = latest.coverage?.total?.lines ?? latest.coverage?.lines ?? null;
      const binarySize = latest.binary_size ?? latest.size?.binary ?? null;
      const escapes = latest.escapes?.total ?? latest.escape_hatches ?? 0;
      const sourceLoc = latest.loc?.source ?? latest.lines?.source ?? '—';
      const testLoc = latest.loc?.test ?? latest.lines?.test ?? '—';

      const metrics = [
        {
          label: 'Line Coverage',
          value: totalCoverage !== null ? `${totalCoverage}%` : '—',
          status: totalCoverage >= 80 ? 'pass' : totalCoverage >= 60 ? 'warn' : 'fail',
          statusText: totalCoverage >= 80 ? 'Passing' : totalCoverage >= 60 ? 'Warning' : 'Below threshold'
        },
        {
          label: 'Binary Size',
          value: binarySize ? `${(binarySize / 1024 / 1024).toFixed(1)} MB` : '—',
          status: binarySize && binarySize < THRESHOLDS.binarySize ? 'pass' : 'warn',
          statusText: binarySize && binarySize < THRESHOLDS.binarySize ? 'Within limit' : 'Check size'
        },
        {
          label: 'Source LOC',
          value: typeof sourceLoc === 'number' ? sourceLoc.toLocaleString() : sourceLoc,
          status: 'pass',
          statusText: 'Tracked'
        },
        {
          label: 'Test LOC',
          value: typeof testLoc === 'number' ? testLoc.toLocaleString() : testLoc,
          status: 'pass',
          statusText: 'Tracked'
        },
        {
          label: 'Escape Hatches',
          value: escapes.toString(),
          status: escapes <= THRESHOLDS.escapes ? 'pass' : 'fail',
          statusText: escapes <= THRESHOLDS.escapes ? 'Within limit' : 'Exceeds limit'
        }
      ];

      let html = `
        <div class="metrics-grid">
          ${metrics.map(m => `
            <div class="metric">
              <div class="metric-label">${m.label}</div>
              <div class="metric-value">${m.value}</div>
              <div class="metric-status ${m.status}">${m.statusText}</div>
            </div>
          `).join('')}
        </div>
      `;

      if (history.length > 1) {
        html += `
          <div class="chart-section">
            <h2>Coverage Trend</h2>
            <div class="chart-container"><canvas id="coverageChart"></canvas></div>
          </div>
        `;
      }

      html += `
        <div class="chart-section">
          <h2>Per-Crate Coverage</h2>
          <table>
            <thead>
              <tr><th>Crate</th><th>Line Coverage</th><th>Threshold</th><th>Status</th></tr>
            </thead>
            <tbody id="coverage-table"></tbody>
          </table>
        </div>
      `;

      document.getElementById('content').innerHTML = html;

      // Render coverage table
      const crates = ['cli', 'core', 'remote'];
      const tableBody = document.getElementById('coverage-table');
      tableBody.innerHTML = crates.map(c => {
        const cov = latest.coverage?.[c] ?? latest.coverage?.[`crates/${c}`] ?? {};
        const lines = cov.lines ?? cov.line ?? null;
        const thresh = THRESHOLDS.coverage[c]?.lines ?? 80;
        const passing = lines !== null && lines >= thresh;
        return `
          <tr>
            <td>crates/${c}</td>
            <td>${lines !== null ? `${lines}%` : '—'}</td>
            <td>≥${thresh}%</td>
            <td class="${passing ? 'pass' : 'fail'}">${passing ? '✓ Pass' : '✗ Below'}</td>
          </tr>
        `;
      }).join('');

      // Render chart if we have history
      if (history.length > 1) {
        renderChart(history);
      }
    }

    function renderChart(history) {
      const labels = history.map(h => {
        const ts = h.timestamp || h.date;
        return ts ? ts.slice(0, 10) : '';
      });
      const data = history.map(h => {
        return h.coverage?.total?.lines ?? h.coverage?.lines ?? null;
      });

      new Chart(document.getElementById('coverageChart'), {
        type: 'line',
        data: {
          labels,
          datasets: [{
            label: 'Line Coverage %',
            data,
            borderColor: '#58a6ff',
            backgroundColor: 'rgba(88, 166, 255, 0.1)',
            tension: 0.3,
            fill: true,
            pointRadius: 3,
            pointHoverRadius: 5
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            y: {
              min: 0,
              max: 100,
              grid: { color: '#30363d' },
              ticks: { color: '#8b949e' }
            },
            x: {
              grid: { color: '#30363d' },
              ticks: { color: '#8b949e', maxRotation: 45 }
            }
          },
          plugins: {
            legend: { display: false }
          }
        }
      });
    }

    loadData();
  </script>
</body>
</html>
```

**Verification:** Create sample `latest.json` and verify dashboard renders correctly.

---

### Phase 5: Complete Benchmark Dashboard

Update `scripts/generate-bench-report.sh` to produce a fully functional Chart.js dashboard.

**File:** `scripts/generate-bench-report.sh` (complete rewrite)

```bash
#!/usr/bin/env bash
# Generate markdown and HTML benchmark report with charts
set -euo pipefail

BENCH_DIR="docs/reports/benchmarks"
LATEST="$BENCH_DIR/latest.json"
HISTORY="$BENCH_DIR/history.json"

mkdir -p "$BENCH_DIR"

if [[ ! -f "$LATEST" ]]; then
    echo "No benchmark results found at $LATEST"
    exit 0
fi

# Generate comparison markdown table
{
    echo "| Operation | Mean | Min | Max | Std Dev |"
    echo "|-----------|------|-----|-----|---------|"
    jq -r '.results[] | "| \(.command) | \(.mean*1000|floor)ms | \(.min*1000|floor)ms | \(.max*1000|floor)ms | \(.stddev*1000*100|floor/100)ms |"' "$LATEST"
} > "$BENCH_DIR/comparison.md"

# Generate index.html with full Chart.js visualization
cat > "$BENCH_DIR/index.html" << 'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Benchmarks - Wok</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
  <style>
    :root {
      --bg: #0d1117;
      --surface: #161b22;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --accent: #58a6ff;
      --green: #3fb950;
      --red: #f85149;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
      padding: 24px;
    }
    .container { max-width: 1200px; margin: 0 auto; }
    header { display: flex; align-items: center; gap: 16px; margin-bottom: 32px; }
    .back { color: var(--text-muted); text-decoration: none; font-size: 1.5rem; }
    .back:hover { color: var(--text); }
    h1 { font-size: 1.5rem; font-weight: 600; }
    .section {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 20px;
      margin-bottom: 24px;
    }
    .section h2 { font-size: 1rem; font-weight: 600; margin-bottom: 16px; }
    .chart-container { position: relative; height: 300px; }
    table { width: 100%; border-collapse: collapse; }
    th, td { padding: 12px; text-align: left; border-bottom: 1px solid var(--border); }
    th { color: var(--text-muted); font-weight: 500; font-size: 0.75rem; text-transform: uppercase; }
    .pass { color: var(--green); }
    .fail { color: var(--red); }
    .loading { color: var(--text-muted); font-style: italic; padding: 24px; text-align: center; }
    .no-data { padding: 48px; text-align: center; }
    .no-data p { color: var(--text-muted); }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <a href="../" class="back" title="Back to reports">←</a>
      <h1>Benchmarks</h1>
    </header>

    <div id="content">
      <div class="loading">Loading benchmark data...</div>
    </div>
  </div>

  <script>
    const THRESHOLDS = {
      list_default: 100,
      list_all: 200,
      filter: 150
    };

    async function loadData() {
      try {
        const [latest, history] = await Promise.all([
          fetch('latest.json').then(r => r.ok ? r.json() : null).catch(() => null),
          fetch('history.json').then(r => r.ok ? r.json() : []).catch(() => [])
        ]);

        if (!latest || !latest.results) {
          showNoData();
          return;
        }

        render(latest, history);
      } catch (e) {
        console.error('Failed to load data:', e);
        showNoData();
      }
    }

    function showNoData() {
      document.getElementById('content').innerHTML = `
        <div class="section no-data">
          <h2>No Data Available</h2>
          <p>Benchmark results will appear here after the first CI run.</p>
        </div>
      `;
    }

    function render(latest, history) {
      let html = `
        <div class="section">
          <h2>Latest Results</h2>
          <table>
            <thead>
              <tr><th>Operation</th><th>Mean</th><th>Min</th><th>Max</th><th>Std Dev</th><th>Status</th></tr>
            </thead>
            <tbody>
              ${latest.results.map(r => {
                const mean = r.mean * 1000;
                const threshold = getThreshold(r.command);
                const passing = mean < threshold;
                return `
                  <tr>
                    <td>${r.command}</td>
                    <td>${mean.toFixed(1)}ms</td>
                    <td>${(r.min * 1000).toFixed(1)}ms</td>
                    <td>${(r.max * 1000).toFixed(1)}ms</td>
                    <td>${(r.stddev * 1000).toFixed(2)}ms</td>
                    <td class="${passing ? 'pass' : 'fail'}">${passing ? '✓' : '✗'}</td>
                  </tr>
                `;
              }).join('')}
            </tbody>
          </table>
        </div>
      `;

      if (history.length > 1) {
        html += `
          <div class="section">
            <h2>Performance Trend</h2>
            <div class="chart-container"><canvas id="trendChart"></canvas></div>
          </div>
        `;
      }

      document.getElementById('content').innerHTML = html;

      if (history.length > 1) {
        renderChart(history);
      }
    }

    function getThreshold(cmd) {
      if (cmd.includes('list_default') || cmd.includes('list default')) return THRESHOLDS.list_default;
      if (cmd.includes('list_all') || cmd.includes('list all')) return THRESHOLDS.list_all;
      return THRESHOLDS.filter;
    }

    function renderChart(history) {
      const labels = history.map(h => (h.timestamp || '').slice(0, 10));

      // Extract key benchmarks for trend
      const benchmarks = ['list_default', 'list_all'];
      const colors = ['#58a6ff', '#3fb950'];

      const datasets = benchmarks.map((name, i) => ({
        label: name,
        data: history.map(h => {
          const result = (h.results || []).find(r => r.command && r.command.includes(name));
          return result ? result.mean * 1000 : null;
        }),
        borderColor: colors[i],
        tension: 0.3,
        pointRadius: 2
      }));

      new Chart(document.getElementById('trendChart'), {
        type: 'line',
        data: { labels, datasets },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            y: {
              beginAtZero: true,
              title: { display: true, text: 'Time (ms)', color: '#8b949e' },
              grid: { color: '#30363d' },
              ticks: { color: '#8b949e' }
            },
            x: {
              grid: { color: '#30363d' },
              ticks: { color: '#8b949e', maxRotation: 45 }
            }
          },
          plugins: {
            legend: { labels: { color: '#c9d1d9' } }
          }
        }
      });
    }

    loadData();
  </script>
</body>
</html>
HTMLEOF

echo "Generated benchmark reports in $BENCH_DIR"
```

**Verification:**
```bash
chmod +x scripts/generate-bench-report.sh
# Create sample data and generate
mkdir -p docs/reports/benchmarks
echo '{"results":[{"command":"list_default","mean":0.045,"min":0.040,"max":0.055,"stddev":0.005}]}' > docs/reports/benchmarks/latest.json
./scripts/generate-bench-report.sh
# Open docs/reports/benchmarks/index.html in browser
```

---

### Phase 6: Workflow Fixes

Address issues in existing workflows.

**Fix 1:** `benchmarks.yml` references `.bench-baseline.json` which doesn't exist.

Update the "Check for regressions" step:
```yaml
- name: Check for regressions
  if: github.event_name == 'pull_request'
  run: |
    # Download baseline from artifacts if available
    if [[ -f checks/benchmarks/results/latest.json ]]; then
      ./checks/benchmarks/compare.sh checks/benchmarks/results/latest.json .bench-baseline.json || true
    fi
  continue-on-error: true
```

**Fix 2:** `quality.yml` glob for metrics output may not match.

Verify the output path of `evaluate.sh` and update:
```yaml
- name: Generate dashboard
  run: |
    mkdir -p docs/reports/quality
    # Find the most recent metrics file
    METRICS=$(ls -t reports/quality/*/metrics.json 2>/dev/null | head -1 || echo "")
    SUMMARY=$(ls -t reports/quality/*/summary.md 2>/dev/null | head -1 || echo "")
    if [[ -n "$METRICS" ]]; then
      cp "$METRICS" docs/reports/quality/latest.json
    fi
    if [[ -n "$SUMMARY" ]]; then
      cp "$SUMMARY" docs/reports/quality/latest.md
    fi
    ./scripts/append-quality-history.sh
```

**Verification:** Push changes and verify workflows run without errors.

---

## Key Implementation Details

### Report Data Flow

```
Workflow Run
     │
     ├─▶ Generate Results (JSON)
     │         │
     │         ├─▶ docs/reports/*/latest.json
     │         │
     │         └─▶ Append to history.json (52-week rolling)
     │
     └─▶ GitHub Pages Deployment
                 │
                 └─▶ https://<user>.github.io/<repo>/
```

### Threshold Configuration

Centralized in `.github/thresholds.json`:
```json
{
  "coverage": {
    "crates/cli": { "lines": 85, "functions": 81 },
    "crates/core": { "lines": 90, "functions": 86 },
    "crates/remote": { "lines": 44, "functions": 39 }
  },
  "quality": {
    "source_avg_loc": 500,
    "source_max_loc": 900,
    "escape_hatches": 3,
    "binary_size_mb": 4
  },
  "benchmarks": {
    "list_default_ms": 100,
    "list_all_ms": 200,
    "filter_ms": 150
  }
}
```

### History Management

- Both quality and benchmark history stored in `history.json`
- Rolling 52-week window (1 year of data)
- Appended by `append-*-history.sh` scripts on each main branch run
- Chart.js reads history for trend visualization

---

## Verification Plan

### Phase 1 Verification
```bash
# Test compare.sh with sample data
./checks/benchmarks/compare.sh /tmp/current.json /tmp/baseline.json
# Should show percentage change for each benchmark
```

### Phase 2 Verification
```bash
ls -la docs/reports/
# .nojekyll, quality/, benchmarks/ should exist
```

### Phase 3-5 Verification
```bash
# Local preview
python3 -m http.server 8000 -d docs/reports/
# Open http://localhost:8000
# - Landing page should show two cards
# - Quality and benchmark pages should load (show no-data message)
```

### Phase 6 Verification
```bash
# Push to feature branch
git push origin feature/github-ci

# Monitor GitHub Actions:
# - ci.yml should pass
# - specs.yml should pass
# - benchmarks.yml should run (may have warnings for missing baseline)

# After merge to main:
# - quality.yml and benchmarks.yml deploy to Pages
# - Visit GitHub Pages URL to verify dashboards
```

### End-to-End Checklist
- [ ] `checks/benchmarks/compare.sh` exists and is executable
- [ ] `docs/reports/` directory structure committed
- [ ] Landing page renders at root URL
- [ ] Quality dashboard loads and shows data after first run
- [ ] Benchmark dashboard loads and shows data after first run
- [ ] PR comments show benchmark comparison table
- [ ] History charts populate over multiple runs
