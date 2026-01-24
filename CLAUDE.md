Wok (wk) is a collaborative, offline-first, AI-friendly issue tracker.

## Issue Labels

- `crates/cli/` → `--label crate:cli`
- `crates/core/` → `--label crate:core`
- `crates/remote/` → `--label crate:remote`
- `checks/specs/` → `--label test:specs`
- `checks/stress/` → `--label test:stress`
- `checks/benchmarks/` → `--label test:benchmarks`

## Common Commands

- `cargo fmt` - Format code
- `cargo check` - Check for errors and warnings
- `cargo clippy` - Lint with clippy
- `cargo test` - Run unit tests
- `make spec` - Run all specs
- `make spec-cli` - Run CLI specs
- `make spec-remote` - Run remote specs
- `make spec ARGS='--filter "pattern"'` - Filter tests by name
- `make spec ARGS='--file cli/unit/list.bats'` - Run specific file
- `make quality` - Run quality evaluation
- `make validate` - Run all validation checks (slow)

## Directory Structure

```
wok/
├── crates/           # Rust workspace
│   ├── cli/          # Command-line interface
│   ├── core/         # Core library
│   └── remote/       # Remote functionality
├── checks/           # Testing & quality
│   ├── specs/        # Specification tests
│   ├── quality/      # Code quality checks
│   ├── stress/       # Stress testing
│   └── benchmarks/   # Performance benchmarks
├── scripts/          # Build and utility scripts
├── docs/             # Documentation
└── plans/            # Planning notes
```

## Specs

When adding or modifying features:

- [ ] Keep `docs/specs/` up to date with changes
- [ ] Update specs in `checks/specs/` before changing behavior
  - Unimplemented bats specs are tagged with `# bats test_tags=todo:implement`
- [ ] Run related specs: `make spec-cli` or `make spec-remote`

See `checks/specs/CLAUDE.md` for spec philosophy and guidelines.

## Landing the Plane

Before committing changes:

- [ ] Run `make check` which will
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo check`
  - `quench check`
  - `cargo audit`
  - `cargo build --workspace`
  - `cargo test`
- [ ] Complete per-crate checklists for any crates modified:
  - crates/cli/CLAUDE.md
  - crates/remote/CLAUDE.md
  - crates/core/CLAUDE.md
- [ ] Remove `todo:implement` tag from implemented specs

