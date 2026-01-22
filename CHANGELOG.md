# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2026-01-21

### Added

- **Idea issue type**: New `idea` type alongside task, feature, and bug. Ideas bridge informal thoughts and committed work, signaling "this might be worth doing" without commitment.

- **`--format ids` option**: New output format for `list`, `ready`, and `search` commands that outputs space-separated issue IDs for shell composition:
  ```bash
  wk close $(wk list -f ids --status done) --reason outdated
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
