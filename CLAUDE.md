Wok is a collaborative, offline-first, AI-friendly issue tracker.

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
- `make check` - Fast validation (fmt, clippy, build, test)
- `make ci` - Full CI validation (adds audit, stricter clippy)
- `make spec` - Run all specs
- `make spec-cli` - Run CLI specs
- `make spec ARGS='--filter "pattern"'` - Filter tests by name
- `make spec ARGS='--file cli/unit/list.bats'` - Run specific file
- `make validate` - Run all validation checks

## Directory Structure

```
wok/
├── crates/           # Rust workspace
│   ├── cli/          # wok - command-line interface
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

- [ ] Run `make check` which will
  - `cargo fmt`
  - `cargo clippy --all`
  - `cargo check`
  - `quench check --fix`
  - `cargo build --workspace`
  - `cargo test`
- [ ] Complete per-crate checklists for any crates modified:
  - crates/cli/CLAUDE.md
  - crates/core/CLAUDE.md
- [ ] Remove `todo:implement` tag from implemented specs
