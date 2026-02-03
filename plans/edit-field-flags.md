# Plan: Hidden Flag Aliases for `wok edit`

## Overview

Add hidden `--title`, `--description`, `--type`, and `--assignee` flags to `wok edit` as convenience aliases for AI agents. These flags provide an alternative to the positional `<ATTR> <VALUE>` syntax (e.g., `wok edit prj-1 --description "new desc"` works the same as `wok edit prj-1 description "new desc"`). The flags are hidden from `--help` output using clap's `hide = true`. The positional syntax remains primary and documented.

## Project Structure

Key files to modify:

```
crates/cli/src/cli/mod.rs              # Edit variant: add hidden flag args
crates/cli/src/lib.rs                  # Dispatch: resolve flags → (attr, value)
crates/cli/src/cli_tests/edit_tests.rs # CLI parsing tests for flag variants
tests/specs/cli/unit/edit.bats         # BATS specs for flag variants
```

## Dependencies

No new dependencies. Clap's `#[arg(long, hide = true)]` is already used in this codebase (see `new` command's `--priority` and `--description` flags).

## Implementation Phases

### Phase 1: Add hidden flags to the Edit command struct

**File:** `crates/cli/src/cli/mod.rs`

Add four hidden `Option<String>` flags to the `Edit` variant. Make `attr` and `value` optional so the command can accept either form.

```rust
Edit {
    /// Issue ID
    id: String,

    /// Attribute to edit (title, description, type, assignee)
    #[arg(conflicts_with_all = ["flag_title", "flag_description", "flag_type", "flag_assignee"])]
    attr: Option<String>,

    /// New value for the attribute
    #[arg(requires = "attr")]
    value: Option<String>,

    /// Set title (hidden, undocumented convenience for AI agents)
    #[arg(long = "title", hide = true, value_name = "VALUE", id = "flag_title")]
    flag_title: Option<String>,

    /// Set description (hidden, undocumented convenience for AI agents)
    #[arg(long = "description", hide = true, value_name = "VALUE", id = "flag_description")]
    flag_description: Option<String>,

    /// Set type (hidden, undocumented convenience for AI agents)
    #[arg(long = "type", hide = true, value_name = "VALUE", id = "flag_type")]
    flag_type: Option<String>,

    /// Set assignee (hidden, undocumented convenience for AI agents)
    #[arg(long = "assignee", hide = true, value_name = "VALUE", id = "flag_assignee")]
    flag_assignee: Option<String>,
},
```

**Key decisions:**
- Use `conflicts_with_all` on `attr` to prevent mixing positional and flag forms
- Use `requires = "attr"` on `value` so `value` is only expected with positional `attr`
- Remove `arg_required_else_help` since we need to handle the "no args at all" case in code (flags make it harder for clap to know when to show help)
- Use `id = "flag_title"` etc. to avoid name collisions with the positional `attr` values
- The mutual exclusion between flags is handled by `conflicts_with_all` on `attr` — but we also need a group or validation to ensure exactly one flag is provided when using flag syntax

**Validation approach:** Use a `clap::ArgGroup` or post-parse validation to ensure that either (a) positional `attr` + `value` are provided, or (b) exactly one hidden flag is provided. Post-parse validation in the dispatch code is simpler and follows existing patterns.

### Phase 2: Update dispatch to resolve flags into (attr, value)

**File:** `crates/cli/src/lib.rs`

Update the `Command::Edit` match arm to resolve the flag or positional form into `(attr, value)` before calling `commands::edit::run`.

```rust
Command::Edit {
    id,
    attr,
    value,
    flag_title,
    flag_description,
    flag_type,
    flag_assignee,
} => {
    let (resolved_attr, resolved_value) = if let Some(v) = flag_title {
        ("title".to_string(), v)
    } else if let Some(v) = flag_description {
        ("description".to_string(), v)
    } else if let Some(v) = flag_type {
        ("type".to_string(), v)
    } else if let Some(v) = flag_assignee {
        ("assignee".to_string(), v)
    } else if let (Some(a), Some(v)) = (attr, value) {
        (a, v)
    } else {
        return Err(Error::Usage {
            message: "missing attribute and value".to_string(),
        });
    };
    commands::edit::run(&id, &resolved_attr, &resolved_value)
}
```

**Note:** The `commands::edit::run` function signature stays unchanged — it already accepts `(&str, &str, &str)`. No changes needed in `crates/cli/src/commands/edit.rs`.

### Phase 3: Add CLI parsing tests for flag variants

**File:** `crates/cli/src/cli_tests/edit_tests.rs`

Add tests to verify:

1. `--title` flag parses correctly: `["wk", "edit", "prj-1", "--title", "New title"]`
2. `--description` flag parses correctly: `["wk", "edit", "prj-1", "--description", "Desc"]`
3. `--type` flag parses correctly: `["wk", "edit", "prj-1", "--type", "bug"]`
4. `--assignee` flag parses correctly: `["wk", "edit", "prj-1", "--assignee", "alice"]`
5. Flag + positional conflict is rejected: `["wk", "edit", "prj-1", "--title", "X", "title", "Y"]`
6. Existing positional tests still pass (regression)

Since the `Edit` struct fields change shape, update the match arms in existing tests to destructure the new fields. The existing tests match `Command::Edit { id, attr, value }` — these need to handle `attr: Some(...)` and `value: Some(...)`.

### Phase 4: Add BATS specs for flag variants

**File:** `tests/specs/cli/unit/edit.bats`

Add spec tests that exercise the flag syntax end-to-end:

```bash
@test "edit: --title flag updates title" {
  wk new task "Original title"
  wk edit prj-1 --title "Updated title"
  run wk show prj-1
  [[ "$output" == *"Updated title"* ]]
}

@test "edit: --description flag updates description" {
  wk new task "Test issue"
  wk edit prj-1 --description "New description"
  run wk show prj-1
  [[ "$output" == *"New description"* ]]
}

@test "edit: --type flag updates type" {
  wk new task "Test issue"
  wk edit prj-1 --type bug
  run wk show prj-1
  [[ "$output" == *"bug"* ]]
}

@test "edit: --assignee flag updates assignee" {
  wk new task "Test issue"
  wk edit prj-1 --assignee alice
  run wk show prj-1
  [[ "$output" == *"alice"* ]]
}
```

### Phase 5: Verify hidden flags don't appear in help

**File:** `crates/cli/src/cli_tests/edit_tests.rs` (or help_tests.rs)

Add a test that captures `--help` output for the edit subcommand and asserts the hidden flags are absent:

```rust
#[test]
fn test_edit_help_hides_flags() {
    let err = parse(&["wk", "edit", "--help"]).unwrap_err();
    let help = err.to_string();
    assert!(!help.contains("--title"), "flag --title should be hidden");
    assert!(!help.contains("--description"), "flag --description should be hidden");
    assert!(!help.contains("--type"), "flag --type should be hidden");
    assert!(!help.contains("--assignee"), "flag --assignee should be hidden");
}
```

## Key Implementation Details

- **No changes to `commands/edit.rs`**: The handler function `run(id, attr, value)` is reused as-is. All resolution from flags to `(attr, value)` happens in the dispatch layer (`lib.rs`).
- **Clap `hide = true`**: Already used in this codebase for `new --priority` and `new --description`. Hides the flag from `--help` and usage strings.
- **`conflicts_with_all`**: Prevents mixing `--title "X"` with positional `title "X"` in the same invocation, giving a clear clap error.
- **`arg_required_else_help`**: May need to be removed or adjusted since optional positional args combined with optional flags make it hard for clap to determine "no useful args". If removed, the error case (no args at all) should be handled in the dispatch match arm.
- **Multiple hidden flags**: Only one flag should be used at a time. This can be enforced with a clap `ArgGroup` that marks the four flags as a group with `multiple = false`, or via post-parse validation. A group is cleaner:
  ```rust
  #[command(
      group = clap::ArgGroup::new("field_flags").args(["flag_title", "flag_description", "flag_type", "flag_assignee"]).multiple(false)
  )]
  ```

## Verification Plan

1. **`cargo check`** — Compiles without errors
2. **`cargo test`** — All existing edit tests pass (with updated destructuring), new flag tests pass
3. **`make spec ARGS='--file cli/unit/edit.bats'`** — BATS specs pass for both positional and flag syntax
4. **`make check-fast`** — Full lint + build + test suite passes
5. **Manual smoke test** — `wk edit prj-1 --title "X"` and `wk edit prj-1 title "X"` produce identical results
6. **Help output check** — `wk edit --help` shows no mention of `--title`, `--description`, `--type`, or `--assignee`
