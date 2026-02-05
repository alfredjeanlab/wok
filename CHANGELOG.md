# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.4.1]

### Added

- **Negative label filter**: Filter issues with `!label` prefix to exclude labels (e.g., `wok list --label '!blocked'`).
- **Built-in `epic` issue type**: Epics are now a first-class issue type.
- **`-p/--prefix` filter**: Filter issues by ID prefix in `list`, `ready`, and `search` commands.
- **`-C <path>` flag**: Change directory before running any command, like `git -C`.
- **Shell completions in install scripts**: Completions are now installed automatically.
- **Hidden edit flags**: `--title`, `--description`, `--type`, `--assignee` flags for non-interactive editing.
- **Multiple labels in label/unlabel**: Pass multiple labels to `label` and `unlabel` commands in a single invocation.
- **Blocked issues in tree output**: `wok tree` now shows blocked issues.
- **Database IPC infrastructure**: Added IPC infrastructure for user-level daemon mode.
- **Centralized env module**: Runtime environment variable access via `env.rs`.

### Changed

- **Unified domain types**: Domain types are now shared across core, CLI, and daemon crates.
- **Extracted `wk-ipc` crate**: Shared IPC types and framing extracted into a dedicated crate.
- **Sync architecture rewrite**: Two-mode model (user-level + private) for sync.
- **Max description length**: Increased from 10K to 1M characters.
- **Max note size**: Increased from 10KB to 200KB.
- **Flexible state transitions**: Lifecycle commands now work from any status.
- **Auto-filter by project prefix**: Issues are automatically filtered by the configured project prefix.

### Fixed

- **`ready` JSON output**: Now returns an array like `list` and `search`.
- **Reverted strict state transitions**: Rolled back overly strict lifecycle validation.

### Chores

- **Test migration**: Migrated all BATS specs to Rust integration tests.
- **Removed dead daemon-routing infrastructure**.
- **Binary renamed from `wk` to `wok`** (completed in this cycle).

## [0.4.0]

### Changed

- **Simplified JSON output for `list` and `search`**: Both commands now return a plain JSON array of issue objects instead of a wrapper object with a nested `issues` key.

- **`ready` hint when truncated**: When `wk ready` output is truncated to the 5-issue limit, a hint now shows how many additional issues exist and suggests using `wk list`.

## [0.3.2]

### Fixed

- **`wok edit` title normalization**: The `edit` command now normalizes titles the same way the `new` command does (trimming whitespace, collapsing internal whitespace).

## [0.3.1]

### Added

- **Multiple IDs in `wok show`**: Pass multiple issue IDs to `wok show` for batch inspection. Uses compact JSONL format (one issue per line) for easy parsing.

- **Multiple prefix support**: Configure multiple prefixes for multi-project collaboration via prefix configuration.

### Changed

- **CLI refactor**: Extracted shared argument structs to reduce code duplication in the CLI.

### Fixed

- **`wok init` when `.wok` directory exists**: Allow initialization when `.wok` directory exists but has no `config.toml`.

## [0.3.0]

### Added

- **Partial ID resolution**: Use abbreviated issue IDs (minimum 3 characters) across all commands. When a prefix uniquely identifies an issue, it resolves to the full ID; ambiguous prefixes return an error listing matches.

- **`wok unlink` command**: Remove external links from issues.

- **`-v/--version` flags**: Display version information with `wok -v` or `wok --version`.

- **`-o/--output id` flag for `wok new`**: Output just the created issue ID for scripting workflows:
  ```bash
  ID=$(wok new task "My task" -o id)
  wok dep $ID blocked-by prj-1234
  ```

- **Colorized help output**: Help text now uses 256-color ANSI codes with TTY detection and `COLOR`/`NO_COLOR` environment variable support. Section headers, commands, and placeholders are color-coded for readability.

- **Dependency flags for `wok new`**: Create issues with dependencies in a single command using `--blocks`, `--blocked-by`, `--tracks`, and `--tracked-by` flags. Supports comma-separated IDs like `--blocks a,b,c`.

- **`wok schema` command**: Output JSON Schema definitions for `list`, `ready`, `search`, and `show` commands, enabling IDE autocompletion and validation for integrations.

- **`WK_TIMINGS=1` environment variable**: Enable performance debugging output showing timing information for internal operations.

- **`now` value in filter expressions**: Use `now` in filters like `closed < now` meaning "all closed issues up to the current time".

- **Bare status fields in filter expressions**: Status fields (`closed`, `skipped`, `completed`, `done`, `cancelled`) can be used without operators as shorthand for "has this status".

### Changed

- **Issue IDs now use 8 hex characters**: Increased from 4 characters to support partial ID resolution with better prefix matching.

- **Long titles auto-truncated**: Issue titles longer than 120 characters are automatically truncated instead of being rejected.

- **Binary renamed to `wok`**: The primary binary is now `wok` instead of `wk`. The short name `wk` is installed as a symlink for convenience. This affects all installation methods (make install, curlpipe, and Homebrew).

- **Output format flag renamed**: The `-f/--format` flag is now `-o/--output` for commands that specify output format (`list`, `ready`, `search`, `show`). The `import` command retains `-f/--format` since it specifies input format.

- **Default `wok init` is now local mode**: Running `wok init` creates a local-only tracker with no remote sync. Use `wok init --remote .` for git-based sync (the previous default behavior).

- **Default oplog branch renamed**: The default git branch for remote sync is now `wok/oplog` instead of `wk/oplog`.

- **Partial bulk updates for lifecycle commands**: When `start`, `done`, `close`, or `reopen` are given multiple IDs, valid operations proceed even if some fail. Shows a summary of successful transitions and returns exit code 1 for partial success.

- **Default reason messages**: Human-readable default reason messages now use "Marked as done" / "Marked as skipped" format.

### Fixed

- **Empty issue title validation**: Issue titles are now validated at argument parsing, preventing creation of issues with empty titles.

## [0.2.1] - 2026-01-21

### Added

- **Idea issue type**: New `idea` type alongside task, feature, and bug. Ideas bridge informal thoughts and committed work, signaling "this might be worth doing" without commitment.

- **`--output ids` option**: New output format for `list`, `ready`, and `search` commands that outputs space-separated issue IDs for shell composition:
  ```bash
  wk close $(wk list -o ids --status done) --reason outdated
  ```

- **Distinct filter fields for terminal states**: Filter expressions can now distinguish between completed and cancelled issues:
  - `completed` / `done` - issues completed with `wk done`
  - `skipped` / `cancelled` - issues closed with `wk close --reason`
  - `closed` - any terminal state (both done and closed)

- **Filter expression documentation**: Added comprehensive filter expression help to `wk list --help` and `wk search --help` documenting syntax, fields, operators, and duration/date formats.

### Changed

- **Default limit for list command**: Default limit of 100 results; use `--limit N` to customize or `--limit 0` for unlimited.

### Fixed

- **Ready command limited to 5 issues**: Agents can only work on a few things at once.

- **Race condition in issue creation**: Handle UNIQUE constraint violation when multiple processes create issues simultaneously by retrying with a fresh timestamp-based ID.

## [0.2.0] - 2026-01-20

Initial tracked release.
