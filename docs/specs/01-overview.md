# Overview & Design Philosophy

## Overview

A minimal local issue tracker with hierarchical blocking dependencies, typed issues, status-aware notes, and audit trail. Uses SQLite for storage.

---

## Design Philosophy

### Unix Philosophy

This tool follows the Unix philosophy of doing one thing well:

- **Single purpose**: Just tracks issues + dependencies. Nothing more.
- **Composable**: Output is plain text, parseable, pipeable to other tools.

### AI-First Design

This tool is designed primarily for AI agents (LLMs) as the primary user:

- **Concise output**: No decorative formatting, spinners, or progress bars. Every character matters in context windows.
- **Predictable structure**: Consistent output format that's easy to parse programmatically.
- **Self-documenting**: Help text includes examples. Error messages are actionable.
- **Idempotent where possible**: Safe to retry operations.
- **No interactive prompts**: All input via arguments and flags. Never prompt for confirmation.
- **Scriptable**: Exit codes are meaningful. Errors go to stderr.
- **Permissive input, strict output**:
  - Accept unambiguous variations in input (e.g. `help`, `-h`, `--help` all work), but only document the canonical form.
  - Attempts to improve generated training data while minimizing token usage.

### Hidden Flags

Hidden flags are undocumented CLI arguments that remain functional but are excluded from help text. They serve specific use cases:

- **AI agent workflows**: Provide semantically meaningful flags (e.g., `--description`) that AI agents may prefer, while keeping the documented interface minimal for human users.
- **Permissive input**: Accept hidden flags as input variations, but only document the canonical form (e.g., document `--note`, accept `--description` as hidden alias).
- **Convenience flags**: Provide shortcuts for common operations (e.g., `--priority 0` instead of `--label priority:0`).

Hidden flags follow these principles:
- No short flag (e.g., no `-d` for `--description`) to keep them truly hidden
- Identical behavior to their documented equivalent
- If both hidden and documented flags are provided, the documented flag takes precedence

Current hidden flags on `new` command:
- `--description "text"`: Alias for `--note`. If both provided, `--note` takes precedence.
- `--priority <0-4>`: Convenience flag that adds `priority:N` label. Equivalent to `--label priority:N`.

### Priority Tag System

Priority is an internal-only attribute derived from issue labels. It is used for sorting in `wk ready` and `wk list` commands but is not displayed in output.

**Tag format:**
- `priority:N` or `p:N` where N is 0-4
- Numeric values: `priority:0` through `priority:4`
- Named values: `priority:highest` (0), `priority:high` (1), `priority:medium` (2), `priority:low` (3), `priority:lowest` (4)

**Precedence rules:**
- If both `priority:` and `p:` tags are present, `priority:` takes precedence
- First matching tag wins if multiple of same prefix exist
- Default priority (no tag): 2 (medium)
- Invalid values are ignored (fall through to next tag or default)

**Priority value mapping:**

| Tag Value | Priority | Description |
|-----------|----------|-------------|
| `priority:0` / `priority:highest` | 0 | Critical/urgent |
| `priority:1` / `priority:high` | 1 | High priority |
| `priority:2` / `priority:medium` / `priority:med` | 2 | Normal (default) |
| `priority:3` / `priority:low` | 3 | Low priority |
| `priority:4` / `priority:lowest` | 4 | Nice to have |

### Out of Scope

The following behaviors are explicitly NOT part of this tool:

- **Time tracking**: No estimates, no time logging
- **Compatibility**: No one-for-one compatibility with other issue trackers
- **Redundant Features**: Avoids aliases, and duplicate or overlapping features

### Semantic Flag Policy

Short flags have consistent meaning across all commands:

| Short | Long | Meaning |
|-------|------|---------|
| `-h` | `--help` | Show help |
| `-r` | `--reason` | Reason for action |
| `-t` | `--type` | Filter by type |
| `-l` | `--label` | Filter by label |
| `-s` | `--status` | Filter by status |
| `-q` | `--filter` | Filter expression (query) |
| `-n` | `--limit` | Limit number of results |
| `-i` | `--interactive` | Interactive mode |
| `-f` | `--format` | Output format |
| `-y` | `--yes` | Non-interactive/auto-confirm |

Flags without shorthands (to avoid ambiguity):
- `--link`, `--note`, `--input`, `--blocked`, `--dry-run`, etc.
