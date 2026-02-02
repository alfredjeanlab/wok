# Plan: Add `-C <path>` Flag

## Overview

Add a top-level `-C <path>` option to `wk` (like `git -C`) that changes the working directory before executing any subcommand. This allows running `wk` commands against a project in a different directory without `cd`-ing first:

```bash
wk -C /path/to/project list
wk -C ../other-repo new task "Fix bug"
```

## Project Structure

Files to create or modify:

```
crates/cli/src/
├── main.rs              # Apply chdir before dispatch
├── cli/mod.rs           # Add -C field to Cli struct
├── cli_tests/
│   ├── flags_tests.rs   # Register 'C' in allowed short flags
│   └── init_tests.rs    # Add -C parsing tests (optional)
docs/specs/
│   └── 04-cli-interface.md  # Document -C in command structure
tests/specs/cli/unit/
│   └── flags.bats       # Add -C integration tests
```

## Dependencies

No new external dependencies. Uses only `std::env::set_current_dir` and `std::path::Path`.

## Implementation Phases

### Phase 1: Add `-C` to the CLI struct

**File:** `crates/cli/src/cli/mod.rs`

Add a `directory` field to the `Cli` struct, before the `version` field:

```rust
pub struct Cli {
    /// Run as if wk was started in <path>
    #[arg(short = 'C', long = "directory", global = true, value_name = "path")]
    pub directory: Option<String>,

    /// Print version
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    pub command: Command,
}
```

Using `global = true` allows `-C` to be placed either before or after the subcommand (e.g., `wk -C /tmp list` and `wk list -C /tmp` both work), matching git's behavior.

**Milestone:** `cargo check` passes. `wk -C /tmp --version` parses without error.

### Phase 2: Apply directory change in `main.rs`

**File:** `crates/cli/src/main.rs`

After successful parse and before calling `wkrs::run()`, change directory if `-C` was provided:

```rust
Ok(cli) => {
    // Change directory if -C was specified
    if let Some(ref dir) = cli.directory {
        let path = std::path::Path::new(dir);
        if let Err(e) = std::env::set_current_dir(path) {
            eprintln!("error: cannot change to directory '{}': {}", dir, e);
            std::process::exit(1);
        }
    }
    if let Err(e) = wkrs::run(cli.command) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
```

This works because all commands discover their project via `find_work_dir()` which calls `std::env::current_dir()`. Changing the process's working directory before dispatch makes every command respect `-C` automatically.

Also update the help-formatting error branch to handle `-C` in the args. The `print_formatted_help` function filters non-flag args to find subcommands; `-C <path>` is a flag with a value so it's already filtered out by the `!arg.starts_with('-')` check, but the *value* of `-C` (e.g., `/tmp`) would be treated as a subcommand name. Fix this by stripping `-C` and its value from args before passing to `print_formatted_help`, or by parsing `-C` before `try_parse` runs (pre-parse approach).

**Recommended approach:** Since clap handles `-C` parsing in the success path, we only need to handle the error path. In the error branches, re-collect args skipping the `-C <value>` pair before passing to `print_formatted_help`:

```rust
Err(e) => {
    // Strip -C and its value from args for help formatting
    let args: Vec<String> = std::env::args().collect();
    let args = strip_dash_c(&args);
    // ... rest of help formatting uses stripped args
}
```

Write a small helper `strip_dash_c` that removes `-C` and the following argument, or `--directory=value` / `--directory value` forms.

**Milestone:** `wk -C /some/project list` works. `wk -C /nonexistent list` prints a clear error.

### Phase 3: Update flag consistency tests

**File:** `crates/cli/src/cli_tests/flags_tests.rs`

Add `'C'` to the allowed short flags map:

```rust
let allowed: std::collections::HashMap<char, &str> = [
    ('C', "directory"), // -C, --directory (top-level, like git -C)
    ('v', "version"),
    // ... rest unchanged
]
```

**File:** `crates/cli/src/cli_tests/init_tests.rs` (or a new `cli_tests/directory_tests.rs`)

Add unit tests for CLI parsing:

```rust
#[test]
fn parse_dash_c_before_subcommand() {
    let cli = Cli::try_parse_from(["wk", "-C", "/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_dash_c_with_equals() {
    let cli = Cli::try_parse_from(["wk", "-C=/tmp", "list"]).unwrap();
    assert_eq!(cli.directory, Some("/tmp".to_string()));
}

#[test]
fn parse_no_dash_c() {
    let cli = Cli::try_parse_from(["wk", "list"]).unwrap();
    assert_eq!(cli.directory, None);
}
```

**Milestone:** `cargo test` passes with all flag consistency checks.

### Phase 4: Add integration specs

**File:** `tests/specs/cli/unit/flags.bats` (append new tests)

```bash
@test "-C flag: runs command in specified directory" {
    # Init a project in a subdirectory
    mkdir -p other-project
    "$WK_BIN" init --path other-project --prefix othr

    # Run from TEST_DIR using -C to target other-project
    run "$WK_BIN" -C other-project new task "Remote task"
    assert_success

    # Verify issue was created in the other project
    run "$WK_BIN" -C other-project list
    assert_success
    assert_output --partial "Remote task"
}

@test "-C flag: error on nonexistent directory" {
    run "$WK_BIN" -C /nonexistent/path list
    assert_failure
    assert_output --partial "cannot change to directory"
}

@test "-C flag: works with init" {
    mkdir -p newproj
    run "$WK_BIN" -C newproj init --prefix np
    assert_success
    [ -d "newproj/.wok" ]
}
```

**Milestone:** `make spec-cli ARGS='--filter "-C flag"'` passes.

### Phase 5: Update documentation

**File:** `docs/specs/04-cli-interface.md`

Update the command structure section at the top:

```markdown
## Command Structure

```
wk [-C <path>] <command> [args] [options]
```

### Global Options

```bash
# Run as if wk was started in <path>
wk -C <path> <command>
# Examples:
wk -C /path/to/project list
wk -C ../other-repo show prj-a1b2
```
```

**Milestone:** Documentation reflects the new flag.

## Key Implementation Details

1. **`std::env::set_current_dir` is process-global.** This is safe here because `wk` is single-threaded and the directory change happens once before any command logic runs.

2. **`global = true` on the clap arg** means clap will accept `-C` in any position relative to the subcommand. This matches git's `-C` behavior where `git -C /tmp status` and `git status -C /tmp` are equivalent (though the before-subcommand position is conventional).

3. **Uppercase `-C` does not conflict** with any existing short flag. The flags test uses a char-to-long mapping. Capital `C` is distinct from lowercase `c` (which is also unused).

4. **Help formatting edge case:** When clap fails to parse (e.g., missing subcommand), the `main.rs` help formatter manually inspects `std::env::args()`. The value following `-C` (a path like `/tmp/myproject`) could be mistaken for a subcommand name. The `strip_dash_c` helper prevents this. An alternative is to simply canonicalize the path and rely on `find_subcommand` failing gracefully (it already handles unknown subcommand names by returning the parent command's help).

5. **No changes to `crates/core/`** are needed. The `-C` flag is purely a CLI concern — it changes the process's working directory before any core logic runs, and `find_work_dir()` already uses `std::env::current_dir()`.

## Verification Plan

1. **Unit tests** (`cargo test`):
   - Flag consistency test passes with `'C'` in the allowed set
   - CLI parsing tests verify `-C <path>` is captured correctly
   - Parsing without `-C` still works (no regression)

2. **Integration specs** (`make spec-cli ARGS='--filter "-C flag"'`):
   - `-C` with valid directory runs commands in that directory
   - `-C` with nonexistent directory produces clear error
   - `-C` works with `init`, `new`, `list`, `show` commands

3. **Manual smoke tests:**
   - `wk -C /tmp init --prefix test && wk -C /tmp new task "Hello" && wk -C /tmp list`
   - `wk -C /nonexistent list` → error message
   - `wk -C . list` → same as `wk list`
   - `wk --help` → shows `-C` option in usage line

4. **Full validation:** `make validate` passes
