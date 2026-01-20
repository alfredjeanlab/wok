# Plan: Add `--format ids` to List Command

**Root Feature:** `wok-3f81`

## Overview

Add a `--format ids` option to the `wk list` command that outputs only issue IDs, one per line. This enables command composition patterns like `wk close $(wk list --format ids --status done)`.

## Project Structure

Files to modify:
```
crates/cli/src/
├── cli.rs              # Add Ids variant to OutputFormat enum
└── commands/
    └── list.rs         # Add ids output handler

checks/specs/cli/unit/
└── list.bats           # Add format ids tests

docs/specs/
└── wk-list.md          # Update documentation (if exists)
```

## Dependencies

No new dependencies required. Uses existing:
- `clap::ValueEnum` for CLI argument parsing
- Standard `println!` for output

## Implementation Phases

### Phase 1: Extend OutputFormat Enum

**File:** `crates/cli/src/cli.rs`

Add `Ids` variant to the existing enum:

```rust
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Ids,  // NEW
}
```

Update help text for format argument (line ~247):

```rust
/// Output format (text, json, ids)
#[arg(long, short, value_enum, default_value = "text")]
format: OutputFormat,
```

**Verification:** `cargo check` passes

### Phase 2: Add IDs Output Handler

**File:** `crates/cli/src/commands/list.rs`

Add match arm in the output section (around line 237):

```rust
match format {
    OutputFormat::Text => {
        for issue in &issues {
            println!("{}", format_issue_line(issue));
        }
    }
    OutputFormat::Json => {
        // existing JSON handling...
    }
    OutputFormat::Ids => {
        for issue in &issues {
            println!("{}", issue.id);
        }
    }
}
```

**Verification:**
- `cargo build`
- Manual test: `wk list --format ids`

### Phase 3: Add Specification Tests

**File:** `checks/specs/cli/unit/list.bats`

Add tests after the existing format tests (~line 162):

```bash
@test "list --format ids outputs one ID per line" {
    id1=$(create_issue task "IDFormat Issue 1")
    id2=$(create_issue task "IDFormat Issue 2")
    run "$WK_BIN" list --format ids
    assert_success
    assert_output --partial "$id1"
    assert_output --partial "$id2"
    # Verify no other content
    [[ ! "$output" =~ "task" ]]
    [[ ! "$output" =~ "todo" ]]
}

@test "list --format ids works with filters" {
    id=$(create_issue task "FilterID Task")
    create_issue bug "FilterID Bug"
    run "$WK_BIN" list --type task --format ids
    assert_success
    assert_output "$id"
}

@test "list --format ids respects limit" {
    for i in {1..15}; do
        create_issue task "LimitID Issue $i" --label "test:limit-ids"
    done
    run "$WK_BIN" list --label "test:limit-ids" --format ids --limit 10
    assert_success
    local count=$(echo "$output" | wc -l | tr -d ' ')
    [ "$count" -eq 10 ]
}

@test "list -f ids works as short flag" {
    id=$(create_issue task "ShortFlagID Issue")
    run "$WK_BIN" list -f ids
    assert_success
    assert_output --partial "$id"
}

@test "list --format ids can be piped to other commands" {
    id=$(create_issue task "Pipe Test Issue")
    # Verify output is clean for command substitution
    run "$WK_BIN" list --format ids
    assert_success
    # Output should be just IDs, no extra whitespace or formatting
    [[ "$output" =~ ^[a-z0-9-]+$ ]] || [[ "$output" =~ $'\n' ]]
}
```

**Verification:** `make spec ARGS='--file cli/unit/list.bats'`

### Phase 4: Update Documentation

**File:** `docs/specs/wk-list.md` (if exists)

Add `ids` to format options documentation with usage examples:

```markdown
### Output Formats

- `text` (default): Human-readable formatted output
- `json`: Structured JSON for programmatic access
- `ids`: One issue ID per line, suitable for piping

### Examples

List IDs for piping to other commands:
```bash
# Close all done issues
wk close $(wk list --status done --format ids)

# Assign all bugs to a user
wk list --type bug --format ids | xargs -I{} wk assign {} alice
```
```

**Verification:** Review documentation for accuracy

## Key Implementation Details

### Output Format Behavior

| Format | Content | Use Case |
|--------|---------|----------|
| `text` | `- [type] (status) id: title` | Human reading |
| `json` | Full issue data + metadata | Programmatic access |
| `ids` | One ID per line | Command composition |

### Design Decisions

1. **One ID per line**: Standard Unix convention for piping
2. **No metadata**: Clean output for `$()` substitution
3. **Respects all filters**: Same filtering/sorting as other formats
4. **Respects limit**: Honors `--limit` flag (default 100)
5. **Lowercase `ids`**: Consistent with `text` and `json`

### Edge Cases

- **Empty results**: Output nothing (no header, no message)
- **With `--limit 0`**: Output all matching IDs (unlimited)
- **Single issue**: Just the one ID, no trailing newline issues

## Verification Plan

1. **Unit tests**: `cargo test -p wok-cli`
2. **Spec tests**: `make spec ARGS='--file cli/unit/list.bats'`
3. **Manual integration**:
   ```bash
   # Create test issues
   wk new task "Test 1"
   wk new task "Test 2"

   # Verify format
   wk list --format ids

   # Verify piping works
   echo "IDs: $(wk list --format ids | tr '\n' ' ')"

   # Verify with filters
   wk list --type task --format ids
   ```
4. **Quality checks**: `cargo fmt && cargo clippy`
5. **Full validation**: `make validate`
