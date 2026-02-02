Wok (wk) is a collaborative, offline-first, AI-friendly issue tracker.

## Issue Labels

- `crates/cli/` → `--label crate:cli`
- `crates/core/` → `--label crate:core`
- `crates/daemon/` → `--label crate:daemon`
- `tests/specs/` → `--label test:specs`

## Common Commands

- `cargo fmt` - Format code
- `cargo check` - Check for errors and warnings
- `cargo clippy` - Lint with clippy
- `cargo test` - Run unit tests
- `make check` - Run all validation checks (fmt, clippy, audit, build, test)
- `make check-fast` - Fast validation for oj workspaces (skips audit, simpler clippy)
- `make spec` - Run all specs
- `make spec-cli` - Run CLI specs
- `make spec ARGS='--filter "pattern"'` - Filter tests by name
- `make spec ARGS='--file cli/unit/list.bats'` - Run specific file
- `make validate` - Run all validation checks

## Directory Structure

```
wok/
├── crates/           # Rust workspace
│   ├── cli/          # wk - command-line interface
│   ├── core/         # Core library
│   └── daemon/       # wokd - IPC daemon
├── tests/            # Testing
│   └── specs/        # Specification tests (BATS)
├── scripts/          # Build and utility scripts
├── docs/             # Documentation
└── plans/            # Planning notes
```

## Specs

When adding or modifying features:

- [ ] Keep `docs/specs/` up to date with changes
- [ ] Update specs in `tests/specs/` before changing behavior
  - Unimplemented bats specs are tagged with `# bats test_tags=todo:implement`
- [ ] Run related specs: `make spec-cli`

See `tests/specs/CLAUDE.md` for spec philosophy and guidelines.

## Landing the Plane

Before committing changes:

- [ ] Run `make check` (or `make check-fast` in oj workspaces) which will
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo check`
  - `quench check`
  - `cargo audit` (skipped by `check-fast`)
  - `cargo build --workspace`
  - `cargo test`
- [ ] Complete per-crate checklists for any crates modified:
  - crates/cli/CLAUDE.md
  - crates/core/CLAUDE.md
- [ ] Remove `todo:implement` tag from implemented specs

## OJ Workspaces

OtterJobs (oj) runs agents in **ephemeral git worktrees** with per-project namespace
isolation. Each workspace gets its own branch and working directory while sharing the
main repo's cargo build cache via `.cargo/config.toml`:

```toml
[build]
target-dir = "<repo-root>/target"
```

This means:
- Multiple agents can work in parallel without git conflicts
- Compilation artifacts are shared across all workspaces for fast rebuilds
- Use `make check-fast` instead of `make check` (skips `cargo audit`, uses simpler clippy flags)

