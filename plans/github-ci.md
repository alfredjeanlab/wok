# GitHub CI/CD Implementation Plan

**Root Feature:** `wok-3cc4`

## Overview

Add comprehensive GitHub Actions CI/CD to the wok project with:
- Automated build, test, and lint on every PR and push
- BATS specification tests with parallel execution
- Quality metrics dashboard published to GitHub Pages
- Benchmark tracking with historical comparison
- Code coverage reporting with threshold enforcement
- Release automation

## Project Structure

```
.github/
├── workflows/
│   ├── ci.yml              # Main CI: build, test, lint, format
│   ├── specs.yml           # BATS specification tests
│   ├── quality.yml         # Quality metrics and reports
│   ├── benchmarks.yml      # Performance benchmarks
│   ├── coverage.yml        # Code coverage with thresholds
│   └── release.yml         # Release automation
├── actions/
│   └── setup-wok/          # Composite action for common setup
│       └── action.yml
└── CODEOWNERS              # Review requirements
docs/
└── reports/                # GitHub Pages source (generated)
    ├── index.html          # Dashboard landing page
    ├── quality/            # Quality metrics reports
    │   ├── latest.json
    │   └── history.json
    └── benchmarks/         # Benchmark reports
        ├── latest.json
        └── history.json
```

## Dependencies

**GitHub Actions:**
- `actions/checkout@v4`
- `actions/cache@v4`
- `actions/upload-artifact@v4`
- `actions/deploy-pages@v4`
- `dtolnay/rust-toolchain@stable`
- `Swatinem/rust-cache@v2`
- `taiki-e/install-action@v2` (for cargo tools)

**Tools (installed in CI):**
- `cargo-audit` - Security vulnerability scanning
- `cargo-llvm-cov` - Code coverage
- `hyperfine` - Benchmarking
- BATS with bats-assert, bats-support

**GitHub Features:**
- GitHub Pages (for reports)
- GitHub Actions cache
- Job artifacts

## Implementation Phases

### Phase 1: Core CI Workflow

Create the foundational CI workflow that runs on every PR and push.

**File:** `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --release
      - name: Upload binary
        uses: actions/upload-artifact@v4
        with:
          name: wk-linux
          path: target/release/wk

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test --workspace

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

  format:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Format check
        run: cargo fmt --all -- --check

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-audit
      - name: Security audit
        run: cargo audit
```

**Milestone:** PR checks show build/test/lint/format/audit status.

---

### Phase 2: BATS Specification Tests

Add parallel BATS test execution for CLI and remote specs.

**File:** `.github/workflows/specs.yml`

```yaml
name: Specs

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  specs:
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        suite: [cli, remote]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build release binary
        run: cargo build --release

      - name: Install BATS
        run: |
          sudo apt-get update
          sudo apt-get install -y bats

      - name: Setup BATS libraries
        run: |
          git clone --depth 1 https://github.com/bats-core/bats-support checks/specs/test_helper/bats-support
          git clone --depth 1 https://github.com/bats-core/bats-assert checks/specs/test_helper/bats-assert

      - name: Run ${{ matrix.suite }} specs
        run: make spec-${{ matrix.suite }}
        env:
          PATH: ${{ github.workspace }}/target/release:$PATH
```

**Milestone:** BATS specs run in parallel (cli/remote matrix) with clear pass/fail.

---

### Phase 3: Code Coverage with Thresholds

Add coverage reporting with per-crate threshold enforcement.

**File:** `.github/workflows/coverage.yml`

```yaml
name: Coverage

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov

      - name: Generate coverage
        run: |
          cargo llvm-cov --workspace --lcov --output-path lcov.info
          cargo llvm-cov report --json --output-path coverage.json

      - name: Check thresholds
        run: |
          # Extract per-package coverage and check thresholds
          # crates/cli: 85% lines, crates/core: 90% lines
          ./checks/quality/coverage.sh --check-only

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false
```

**Milestone:** Coverage reports visible in PRs, threshold failures block merge.

---

### Phase 4: Quality Metrics Dashboard

Create scheduled quality evaluation with GitHub Pages publishing.

**File:** `.github/workflows/quality.yml`

```yaml
name: Quality

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 6 * * 1'  # Weekly on Monday 6am UTC
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  metrics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for git stats
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-llvm-cov

      - name: Build release
        run: cargo build --release

      - name: Run quality evaluation
        run: ./checks/quality/evaluate.sh

      - name: Generate dashboard
        run: |
          mkdir -p docs/reports/quality
          # Copy latest metrics
          cp reports/quality/*/metrics.json docs/reports/quality/latest.json
          cp reports/quality/*/summary.md docs/reports/quality/latest.md
          # Append to history
          ./scripts/append-quality-history.sh

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: quality-report
          path: docs/reports/quality/

  deploy:
    needs: metrics
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          name: quality-report
          path: docs/reports/quality/
      - uses: actions/configure-pages@v4
      - uses: actions/upload-pages-artifact@v3
        with:
          path: docs/reports/
      - id: deployment
        uses: actions/deploy-pages@v4
```

**Supporting script:** `scripts/append-quality-history.sh`

```bash
#!/usr/bin/env bash
# Append current metrics to history.json for trend tracking
HISTORY_FILE="docs/reports/quality/history.json"
LATEST_FILE="docs/reports/quality/latest.json"
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
```

**Milestone:** Quality dashboard at `https://<user>.github.io/<repo>/quality/`.

---

### Phase 5: Benchmark Tracking

Add benchmark runs with historical comparison and regression detection.

**File:** `.github/workflows/benchmarks.yml`

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 7 * * 1'  # Weekly after quality
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

jobs:
  bench:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install hyperfine
        run: |
          wget https://github.com/sharkdp/hyperfine/releases/download/v1.18.0/hyperfine_1.18.0_amd64.deb
          sudo dpkg -i hyperfine_1.18.0_amd64.deb

      - name: Build release
        run: cargo build --release

      - name: Run benchmarks
        run: |
          export PATH="$PWD/target/release:$PATH"
          cd checks/benchmarks
          ./run.sh --size large --output-dir results

      - name: Check for regressions
        if: github.event_name == 'pull_request'
        run: |
          # Compare against baseline from main branch
          ./checks/benchmarks/compare.sh results/latest.json .bench-baseline.json

      - name: Generate report
        run: |
          mkdir -p docs/reports/benchmarks
          cp checks/benchmarks/results/latest.json docs/reports/benchmarks/
          ./scripts/append-bench-history.sh
          ./scripts/generate-bench-report.sh

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-report
          path: docs/reports/benchmarks/

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('docs/reports/benchmarks/comparison.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## Benchmark Results\n\n' + report
            });

  deploy:
    needs: bench
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment:
      name: github-pages
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          name: benchmark-report
          path: docs/reports/benchmarks/
      - uses: actions/download-artifact@v4
        with:
          name: quality-report
          path: docs/reports/quality/
        continue-on-error: true
      - uses: actions/configure-pages@v4
      - uses: actions/upload-pages-artifact@v3
        with:
          path: docs/reports/
      - uses: actions/deploy-pages@v4
```

**Supporting script:** `scripts/generate-bench-report.sh`

```bash
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

jq -r '.results[] | "| \(.command) | \(.mean*1000|floor)ms | \(.min*1000|floor)ms | \(.max*1000|floor)ms | ✅ |"' \
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
```

**Milestone:** Benchmark dashboard with trends, PR comments show regression alerts.

---

### Phase 6: Release Automation

Add release workflow for tagged versions.

**File:** `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            name: wk-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            name: wk-macos-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            name: wk-macos-aarch64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      - name: Package
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../${{ matrix.name }}.tar.gz wk
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}.tar.gz

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            wk-linux-x86_64/wk-linux-x86_64.tar.gz
            wk-macos-x86_64/wk-macos-x86_64.tar.gz
            wk-macos-aarch64/wk-macos-aarch64.tar.gz
          generate_release_notes: true
```

**Milestone:** Tagged releases automatically publish binaries.

---

## Key Implementation Details

### Caching Strategy

Use `Swatinem/rust-cache@v2` with these considerations:
- Separate cache keys for debug vs release builds
- Share cache between PR and push workflows via `shared-key`
- Cache BATS test helpers to speed up spec runs

### Report Page Architecture

The GitHub Pages site structure:

```
docs/reports/
├── index.html          # Landing dashboard
├── quality/
│   ├── index.html      # Quality metrics dashboard
│   ├── latest.json     # Current metrics
│   ├── latest.md       # Human-readable summary
│   └── history.json    # 52-week trend data
└── benchmarks/
    ├── index.html      # Benchmark dashboard with charts
    ├── latest.json     # Current benchmark results
    ├── comparison.md   # Tabular comparison
    └── history.json    # Historical benchmark data
```

### Threshold Configuration

Centralize thresholds in a config file for easy adjustment:

**File:** `.github/thresholds.json`

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

### PR Comment Formatting

Benchmark PR comments use collapsible sections for detailed results:

```markdown
## Benchmark Results

**Summary:** ✅ No regressions detected

<details>
<summary>Full Results</summary>

| Operation | Mean | vs Main | Status |
|-----------|------|---------|--------|
| list      | 45ms | -2ms    | ✅     |
| list --all| 120ms| +5ms    | ✅     |
...
</details>
```

---

## Verification Plan

### Phase 1 Verification
- [ ] Push to feature branch triggers all CI jobs
- [ ] Build artifact is downloadable
- [ ] Clippy warnings fail the build
- [ ] Format violations fail the build
- [ ] Security audit runs successfully

### Phase 2 Verification
- [ ] BATS specs run in matrix (cli, remote)
- [ ] Test failures properly reported
- [ ] BATS libraries auto-installed

### Phase 3 Verification
- [ ] Coverage report generated
- [ ] Codecov integration working
- [ ] Threshold check fails when coverage drops

### Phase 4 Verification
- [ ] Quality metrics collected on push to main
- [ ] Weekly schedule triggers correctly
- [ ] GitHub Pages deploys quality dashboard
- [ ] History tracks 52 weeks of data

### Phase 5 Verification
- [ ] Benchmarks run on PR and main
- [ ] PR comments show benchmark comparison
- [ ] Regression detection alerts on slowdowns
- [ ] Benchmark charts render on GitHub Pages

### Phase 6 Verification
- [ ] Tagged release triggers build matrix
- [ ] All three platforms build successfully
- [ ] Release created with attached binaries
- [ ] Release notes auto-generated

### End-to-End Verification
- [ ] Full `make validate` passes in CI
- [ ] Dashboard accessible at GitHub Pages URL
- [ ] PR workflow provides actionable feedback
- [ ] Main branch protection uses required checks
