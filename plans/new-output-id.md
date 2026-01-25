# Plan: Add `-o/--output id` to `wok new`

## Overview

Add support for `-o/--output id` to `wok new`, which outputs just the newly created issue ID with no other text. This enables scripting workflows like `ID=$(wok new task "My task" -o id)`. The implementation also normalizes the existing `Ids` variant to `Id` across all commands and help text, while silently accepting both "id" and "ids" as input for backwards compatibility.

## Project Structure

Key files to modify:

```
crates/cli/
├── src/
│   ├── cli.rs              # OutputFormat enum, New command args
│   ├── lib.rs              # Command dispatch (pass output to new::run)
│   └── commands/
│       └── new.rs          # Add output format handling
```

## Dependencies

No new dependencies required. Uses existing clap `#[value(alias = "...")]` attribute for alias support.

## Implementation Phases

### Phase 1: Rename `Ids` to `Id` with backward-compatible alias

**Files:** `crates/cli/src/cli.rs`

Update the `OutputFormat` enum to use `Id` as the primary name while accepting `ids` as a hidden alias:

```rust
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    #[value(alias = "ids")]  // Accept "ids" for backwards compatibility
    Id,
}
```

**Verification:** Run `cargo check` and `cargo test` to ensure enum rename doesn't break compilation.

### Phase 2: Add output flag to `wok new` CLI definition

**Files:** `crates/cli/src/cli.rs`

Add the output argument to the `New` command struct (after `tracked_by`):

```rust
New {
    // ... existing fields ...

    /// Issues that track this new issue (comma-separated or repeated)
    #[arg(long, value_name = "IDS")]
    tracked_by: Vec<String>,

    /// Output format (text, json, id)
    #[arg(long = "output", short = 'o', value_enum, default_value = "text")]
    output: OutputFormat,
},
```

Update the command's `after_help` to include an output example.

**Verification:** Run `cargo check` and verify `wok new --help` shows the new flag.

### Phase 3: Update command dispatch in lib.rs

**Files:** `crates/cli/src/lib.rs`

Pass the output format from the command to the run function:

```rust
Command::New {
    type_or_title,
    title,
    label,
    note,
    link,
    assignee,
    priority,
    description,
    blocks,
    blocked_by,
    tracks,
    tracked_by,
    output,  // Add this
} => commands::new::run(
    type_or_title,
    title,
    label,
    note,
    link,
    assignee,
    priority,
    description,
    blocks,
    blocked_by,
    tracks,
    tracked_by,
    output,  // Pass to run()
),
```

**Verification:** Run `cargo check` to verify dispatch compiles.

### Phase 4: Implement output formatting in new.rs

**Files:** `crates/cli/src/commands/new.rs`

1. Add `OutputFormat` to imports
2. Add `output: OutputFormat` parameter to both `run()` and `run_impl()`
3. Replace the println! at the end with format-aware output:

```rust
use crate::cli::OutputFormat;

// In run() signature:
pub fn run(
    // ... existing params ...
    output: OutputFormat,
) -> Result<()> {
    // ... pass output to run_impl ...
}

// In run_impl() signature:
pub(crate) fn run_impl(
    // ... existing params ...
    output: OutputFormat,
) -> Result<()> {
    // ... existing implementation ...

    // Replace the println! with:
    match output {
        OutputFormat::Text => {
            println!(
                "Created [{}] ({}) {}: {}",
                issue_type, issue.status, id, normalized.title
            );
        }
        OutputFormat::Id => {
            println!("{}", id);
        }
        OutputFormat::Json => {
            // Simple JSON output with the created issue
            let labels_vec = db.get_labels(&id)?;
            let json_output = serde_json::json!({
                "id": id,
                "type": issue_type.as_str(),
                "title": normalized.title,
                "status": issue.status.as_str(),
                "labels": labels_vec,
                "assignee": issue.assignee,
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
    }

    Ok(())
}
```

**Verification:**
- `wok new task "Test" -o text` → prints human-readable message
- `wok new task "Test" -o id` → prints just the ID
- `wok new task "Test" -o ids` → prints just the ID (alias works)
- `wok new task "Test" -o json` → prints JSON output

### Phase 5: Update help text and documentation

**Files:** `crates/cli/src/cli.rs`

1. Update the `New` command's `after_help` to add an output example:
   ```
   wok new task \"My task\" -o id      Create task, output only ID
   ```

2. Update help text comments for `list`, `ready`, `search` to say "id" instead of "ids" where applicable (the value_enum will show "id" automatically after Phase 1)

**Verification:** Run `wok list --help`, `wok ready --help`, `wok search --help` and verify help text shows "id" not "ids".

### Phase 6: Add tests

**Files:** `crates/cli/src/commands/new_tests.rs` (or inline)

Add tests for the new output format functionality:

```rust
#[test]
fn new_output_id_only_outputs_id() {
    // Test that -o id outputs just the ID
}

#[test]
fn new_output_ids_alias_works() {
    // Test that -o ids also outputs just the ID (backward compat)
}

#[test]
fn new_output_json_outputs_structured() {
    // Test that -o json outputs valid JSON with expected fields
}
```

**Verification:** `cargo test` passes all new tests.

## Key Implementation Details

### Backward Compatibility

The `#[value(alias = "ids")]` clap attribute ensures:
- `-o id` works (primary, shown in help)
- `-o ids` works (alias, hidden from help)
- Existing scripts using `-o ids` on `list`/`ready`/`search` continue to work

### Output Format Semantics

| Format | Output |
|--------|--------|
| `text` | `Created [type] (status) id: title` (default, human-readable) |
| `id` | `prj-a1b2` (just the ID, for scripting) |
| `json` | `{"id": "...", "type": "...", ...}` (structured, for tooling) |

### Scripting Use Case

```bash
# Capture the new ID for further commands
ID=$(wok new task "Build feature X" -o id)
wok dep $ID blocked-by prj-1234
wok label $ID urgent
```

## Verification Plan

1. **Unit tests:** Add tests in `new_tests.rs` for output format handling
2. **CLI integration:**
   - `wok new --help` shows `-o/--output` with options
   - `wok new task "x" -o id` outputs just the ID
   - `wok new task "x" -o ids` outputs just the ID (alias)
   - `wok new task "x" -o json` outputs valid JSON
3. **Backward compatibility:**
   - `wok list -o ids` still works
   - `wok ready -o ids` still works
   - `wok search "x" -o ids` still works
4. **Quality checks:**
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test`
   - `make spec-cli` (if relevant specs exist)
