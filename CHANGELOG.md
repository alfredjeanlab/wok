# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

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
