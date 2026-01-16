# Specs Module

Specs define acceptance criteria. **Specs are updated first**, implementation follows.

Spec failures usually indicate **implementation defects**, not outdated specs. Implementation agents often over-implement, adding features or aliases that violate the spec.

## Before Creating Issues for Failures

Search for existing issues and recent changes before assuming a bug is new:

1. `bd list --status=open` - check for existing issues
2. `cmd/status` - review in-progress and recently completed projects
3. `git log --oneline -10 checks/specs/` - recent spec changes
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

## Landing Checklist

Specs may be merged before implementation exists. Failing tests are expected until implementation catches up.

- [ ] New specs align with REQUIREMENTS.md
