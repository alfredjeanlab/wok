# Quality Evaluation Suite for `wk` CLI

A quality evaluation suite to measure the `wk` CLI across key dimensions: lines of code, binary size, memory usage, test coverage, and escape hatch usage.

## Quick Start

```bash
# Run all automated metrics
./evaluate.sh

# Run specific metrics
./evaluate.sh loc size

# View weekly summary
./weekly.sh

# View results
cat ../../reports/quality/*/summary.md
```

## Metrics

| Metric | Script | Description |
|--------|--------|-------------|
| Lines of Code | `metrics/loc.sh` | Source and test code size (counted separately) |
| File Size | `metrics/file_size.sh` | Average/max LOC per file (source: avg <500, max <900; test: avg <700, max <1100) |
| Binary Size | `metrics/size.sh` | Deployable artifact size |
| Memory Usage | `metrics/memory.sh` | Peak RSS during operations |
| Code Coverage | `metrics/coverage.sh` | Test coverage percentage |
| Escape Hatches | `metrics/escapes.sh` | Type safety bypasses |
| Test Time | `metrics/test_time.sh` | Test suite execution time |
| Compile Time | `metrics/compile_time.sh` | Cold compile and clean re-compile times |

## Agent Reviews

Agent-based reviews require Claude and are run separately:

| Review | Prompt | Description |
|--------|--------|-------------|
| Idiomaticness | `agents/idiomaticness.md` | Language convention adherence |
| Security | `agents/security.md` | Vulnerability assessment |

Run agent reviews:
```bash
./agents/run-review.sh idiomaticness
./agents/run-review.sh security
```

## Directory Structure

```
quality/
├── README.md                 # This file
├── evaluate.sh               # Main evaluation runner
├── weekly.sh                 # Weekly summary report
├── metrics/
│   ├── loc.sh                # Lines of code measurement
│   ├── file_size.sh          # Average/max LOC per file
│   ├── size.sh               # Binary size measurement
│   ├── memory.sh             # Memory profiling
│   ├── coverage.sh           # Code coverage collection
│   ├── escapes.sh            # Escape hatch detection
│   ├── test_time.sh          # Test suite run time
│   └── compile_time.sh       # Compilation time (cold and clean)
└── agents/
    ├── idiomaticness.md      # Prompt for idiomaticness review
    ├── security.md           # Prompt for security review
    └── run-review.sh         # Script to run agent reviews
```

## Output

Reports are saved to `reports/quality/YYYYMMDD_HHMMSS/` (in the root project directory):

- `loc.txt` - Lines of code results
- `file_size.txt` - Average/max LOC per file
- `size.txt` - Binary size results
- `memory.txt` - Memory profiling results
- `coverage.txt` - Code coverage results
- `escapes.txt` - Escape hatch counts
- `test_time.txt` - Test suite run time results
- `compile_time.txt` - Compilation time results
- `metrics.json` - Machine-readable metrics (for comparisons)
- `comparison.md` - Comparison with previous report (auto-generated)
- `summary.md` - Combined summary

## Report Management

**Auto-cleanup:** Both `evaluate.sh` and `weekly.sh` automatically remove older reports within 2 hours of newer ones, keeping only the latest in each time window.

**Weekly reports:** Run `./weekly.sh` to see trends across reports:
```bash
./weekly.sh          # Last 7 days
./weekly.sh -n 14    # Last 14 days
```

## Interpretation Guide

### Lines of Code
- Source code located in `crates/cli/src/`, `crates/remote/src/`, and `crates/core/src/`
- Test LOC should be 1x-4x source LOC

### File Size
- **Source files: avg <500 LOC, max <900 LOC**
  - Large files are harder for LLMs to fit in context
- **Test files: avg <700 LOC, max <1100 LOC**
- Smaller files improve LLM context efficiency and code navigability

### Binary Size
- **Lower is better**
- Rust release with LTO and strip is typical ~2-4 MB

### Memory Usage
- **Lower is better**
- Rust typically has lowest footprint
- Watch for linear vs superlinear growth

### Code Coverage
- **Higher is better** (but 100% isn't always practical)
- Aim for >85% line coverage
- Focus on critical paths: error handling, edge cases

### Escape Hatches
- **Lower is better** (<3 in source code)
- Context matters: `.unwrap()` in tests is fine
- Zero `unsafe` is ideal for a CLI tool

### Test Time
- **Lower is better** (<5s) for fast feedback loops
- Watch for tests that take disproportionately long
- Slow tests hurt developer productivity

### Compile Time
- **Lower is better** (5s-30s cold compile)
- Cold compile: time from clean state (no cache)
- Clean re-compile: time to rebuild after `cargo clean`

## Expected Baselines

| Metric | Target |
|--------|--------|
| Test LOC | 1x-4x source LOC |
| Source File Avg | <500 LOC |
| Source File Max | <900 LOC |
| Test File Avg | <700 LOC |
| Test File Max | <1100 LOC |
| Binary Size | 2-4 MB |
| Memory (idle) | 1-2 MB |
| Coverage | >85% |
| Escape Hatches | <3 |
| Test Time | <5s |
| Cold Compile | 5-30s |

