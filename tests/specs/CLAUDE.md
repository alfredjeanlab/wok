# Specs Module

Specs define acceptance criteria. **Specs are updated first**, implementation follows.

Spec failures usually indicate **implementation defects**, not outdated specs. Implementation agents often over-implement, adding features or aliases that violate the spec.

## Before Creating Issues for Failures

Search for existing issues and recent changes before assuming a bug is new:

1. `bd list --status=open` - check for existing issues
2. `cmd/status` - review in-progress and recently completed projects
3. `git log --oneline -10 tests/specs/` - recent spec changes
4. `git log --oneline -10 crates/cli/` - recent CLI changes that may have diverged

## Key Philosophy (REQUIREMENTS.md)

- **Single purpose**: issues + dependencies only, nothing more
- **No redundant features**: avoids aliases and overlapping commands
- **Permissive input, strict output**: accept variations (`-h`, `--help`) but only document canonical forms
- **AI-First**: concise output, no interactive prompts, predictable structure

## Running Tests

```bash
# Build first
cargo build --release

# Run all specs
make spec

# Run by suite
make spec-cli
make spec-remote

# Filter by test name
make spec ARGS='--filter "short flag"'

# Run specific file
make spec ARGS='--file cli/unit/list.bats'

# Combine suite + filter
make spec-cli ARGS='--filter "list"'

# Run unimplemented specs
make spec-todo
```

## Rust Specs

For tests that benefit from Rust's type system or complex setup:

```bash
# Run Rust specs
cargo test --test specs

# Run specific test
cargo test --test specs smoke_test_wk_version
```

### Structure

- `crates/cli/tests/specs.rs` - Entry point, imports prelude
- `crates/cli/tests/specs_prelude.rs` - Helpers (Project, Wk, assertions)
- `tests/specs/prelude.rs` - Reference copy of helpers (canonical documentation)

### Core Helpers

```rust
// Isolated project with temp directory
let project = Project::new("test");

// Run commands in project context
project.wk().args(["new", "task", "My task"]).output().success();

// Create issue and get ID
let id = project.create_issue("task", "My task");

// Standalone command (no project context)
Wk::new().arg("--version").output().success();
```

### When to Use Rust vs BATS

Use **Rust specs** when:
- Complex setup/teardown logic
- Type-safe fixtures or builders
- Testing internal behavior (with `wkrs` lib)
- Parameterized tests with `yare`

Use **BATS specs** when:
- Simple command-line invocation checks
- Testing shell integration (pipes, redirects)
- Quick iteration on CLI behavior
- Documenting user-facing examples
