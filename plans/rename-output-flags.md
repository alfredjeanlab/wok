# Rename Output Flags: `-f/--format` → `-o/--output`

**Root Feature:** `wok-9bc8`

## Overview

Rename the `-f/--format` flag to `-o/--output` across all commands that use it for output format selection. This affects five commands (`list`, `ready`, `search`, `show`, `import`) and requires updates to implementation code, documentation, tests, and benchmarks.

## Project Structure

Key files requiring changes:

```
wok/
├── crates/cli/
│   ├── src/
│   │   └── cli.rs              # Clap argument definitions (5 commands)
│   ├── tests/
│   │   ├── output.rs           # Unit tests
│   │   └── import.rs           # Import tests
│   └── REQUIREMENTS.md         # CLI requirements documentation
├── docs/specs/
│   ├── 01-overview.md          # Short flag policy table
│   └── 04-cli-interface.md     # Command documentation
├── checks/
│   ├── specs/cli/
│   │   ├── consistency/shared_flags.bats
│   │   ├── unit/{list,ready,search,import}.bats
│   │   ├── integration/{sorting,external_links}.bats
│   │   └── edge_cases/import_edge.bats
│   └── benchmarks/scenarios/
│       └── {search,write,list,ready}.sh
└── CHANGELOG.md
```

## Dependencies

No new external dependencies required. This is a pure rename refactor.

## Implementation Phases

### Phase 1: Update Clap Argument Definitions

**File:** `crates/cli/src/cli.rs`

Update the `#[arg]` attributes for all five commands:

| Command | Line | Change |
|---------|------|--------|
| `list`   | 259 | `#[arg(long, short, ...)]` → `#[arg(long = "output", short = 'o', ...)]` |
| `ready`  | 293 | Same pattern |
| `search` | 349 | Same pattern |
| `show`   | 359 | Same pattern |
| `import` | 531 | Same pattern |

**Pattern for each command:**

```rust
// Before
#[arg(long, short, value_enum, default_value = "text")]
pub format: OutputFormat,

// After
#[arg(long = "output", short = 'o', value_enum, default_value = "text")]
pub output: OutputFormat,
```

Also update help text examples in the `about` attributes (around lines 198-220):

```rust
// Before: wk list -f json
// After:  wk list -o json
```

**Milestone:** `cargo check` passes, `wk list --help` shows `-o/--output`

---

### Phase 2: Update Field Names in Dispatch Code

**File:** `crates/cli/src/lib.rs`

Update struct field references from `format` to `output`:

| Lines | Command | Change |
|-------|---------|--------|
| 116-129 | `List` | `format` → `output` |
| 130 | `Show` | `format` → `output` |
| 158-167 | `Import` | `format` → `output` |
| 168-175 | `Ready` | `format` → `output` |
| 176-188 | `Search` | `format` → `output` |

**Milestone:** `cargo check` passes

---

### Phase 3: Update Documentation

**Files to update:**

1. **`docs/specs/01-overview.md:96`** - Short flag policy table
   ```markdown
   | `-o` | `--output` | Output format |
   ```

2. **`docs/specs/04-cli-interface.md`** - Command documentation
   - Line 100: `wk list [--output/-o text|json|ids]`
   - Line 106: `wk ready [--output/-o text|json]`
   - Line 125: `wk show <id> [--output json]`

3. **`crates/cli/REQUIREMENTS.md:96`** - Requirements documentation
   ```markdown
   `--output <format>` / `-o <format>`
   ```

4. **`CHANGELOG.md:11`** - Update any references to `--format`

**Milestone:** Documentation is consistent with new flag names

---

### Phase 4: Update Spec Tests

**Files to update:**

1. **`checks/specs/cli/consistency/shared_flags.bats`**
   - Line 87: Update test "show accepts -o as short form for --output"
   - Line 94: Update test "import accepts -o as short form for --output"

2. **`checks/specs/cli/unit/list.bats`** (multiple occurrences)
   - Replace all `--format` with `--output`
   - Replace all `-f` with `-o` where used for format

3. **`checks/specs/cli/unit/ready.bats`** - Same pattern

4. **`checks/specs/cli/unit/search.bats`** - Same pattern

5. **`checks/specs/cli/unit/import.bats`** - Same pattern

6. **`checks/specs/cli/edge_cases/import_edge.bats`** - Same pattern

7. **`checks/specs/cli/integration/sorting.bats`** - Lines 124, 137

8. **`checks/specs/cli/integration/external_links.bats`** - Line 59

**Milestone:** `make spec-cli` passes

---

### Phase 5: Update Rust Unit Tests

**Files to update:**

1. **`crates/cli/tests/output.rs`** - Lines 83, 121
   - Replace `--format` with `--output`

2. **`crates/cli/tests/import.rs`** - Line 155
   - Replace `--format` with `--output`

3. **`crates/cli/src/cli_tests/show_tests.rs`** - Line 29
   - Replace `--format json` with `--output json`

**Milestone:** `cargo test` passes

---

### Phase 6: Update Benchmarks and Archive Plans

**Files to update:**

1. **`checks/benchmarks/scenarios/search.sh`** - Lines 147, 151
2. **`checks/benchmarks/scenarios/write.sh`** - Lines 48, 67, 86, 109, 129, 150, 173
3. **`checks/benchmarks/scenarios/list.sh`** - Lines 169, 173
4. **`checks/benchmarks/scenarios/ready.sh`** - Line 26

Replace all `--format json` with `--output json`.

**Optional:** Update `plans/archive/2026-01-20/list-ids.md` for consistency (archive documentation).

**Milestone:** `make validate` passes

## Key Implementation Details

### Clap Attribute Pattern

When renaming the flag, use explicit `long = "output"` to keep the struct field name short:

```rust
#[arg(long = "output", short = 'o', value_enum, default_value = "text")]
pub output: OutputFormat,
```

This allows the field to be named `output` while the CLI flag is `--output`.

### Search and Replace Strategy

Use these patterns for safe bulk replacement:

1. **Long flag:** `--format` → `--output` (in test commands)
2. **Short flag:** `-f json` → `-o json`, `-f text` → `-o text`, `-f ids` → `-o ids`
3. **Documentation:** `--format/-f` → `--output/-o`
4. **Help text:** `wk list -f json` → `wk list -o json`

**Be careful not to replace:**
- `--format` in `git log --format` or other external commands
- `-f` when used for other purposes (e.g., `-f` for file in other contexts)

### OutputFormat Enum

The `OutputFormat` enum itself (`Text`, `Json`, `Ids`) does not need renaming - only the flag and field names change.

## Verification Plan

### Per-Phase Verification

| Phase | Verification Command |
|-------|---------------------|
| 1 | `cargo check && wk list --help \| grep -E '\-o.*--output'` |
| 2 | `cargo check` |
| 3 | Manual review of documentation |
| 4 | `make spec-cli` |
| 5 | `cargo test` |
| 6 | `make validate` |

### Final Verification Checklist

```bash
# Full quality check
make check

# Full spec suite
make spec

# Verify help text shows new flags
wk list --help
wk ready --help
wk search --help
wk show --help
wk import --help

# Verify both short and long forms work
wk list -o json
wk list --output json
```

### Regression Testing

Ensure no references to old flag remain:

```bash
# Should return no matches in source code
grep -r '\-\-format' crates/ docs/ checks/ --include='*.rs' --include='*.md' --include='*.bats' --include='*.sh'

# Verify -f is not used for format (careful: -f may be used elsewhere)
grep -r '\-f json\|\-f text\|\-f ids' crates/ docs/ checks/
```
