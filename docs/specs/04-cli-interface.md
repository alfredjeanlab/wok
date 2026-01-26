# CLI Interface

## Command Structure

```
wk <command> [args] [options]
```

## Commands

### Help

```bash
# General help
wk help
wk -h / wk --help          # (hidden aliases)

# Subcommand help
wk help <command>
wk <command> -h/--help     # (hidden aliases)
# Example: wk help dep
```

### Onboarding

```bash
# Output issue tracker workflow template (useful for AI agents)
wk prime
```

The `prime` command outputs a markdown template with common commands and workflows.
It works without initialization (no `.wok/` directory required) and is useful for:
- AI agent context priming at session start
- Quick reference for available commands
- Onboarding new users to the issue tracker

### Setup

```bash
# Initialize issue tracker (prefix defaults to directory name)
wk init

# Initialize with explicit prefix
wk init --prefix prj

# Initialize at specific path
wk init --path /path/to/shared --prefix prj

# Initialize with workspace link only (no local database)
# Note: workspace directory must exist
wk init --workspace /path/to/workspace

# Initialize with workspace and explicit prefix
wk init --workspace /path/to/workspace --prefix prj
```

**Workspace validation**: When `--workspace` is specified, the workspace directory must exist. The command fails with a clear error if the path does not exist.

### Issue Lifecycle

```bash
# Create issue (type defaults to "task")
wk new [type] <title> [--label <label>[,<label>...]]... [--note "..."] [--link <url>]...
# Examples:
wk new "Fix login bug"                              # task (default)
wk new task "Fix login bug" --label auth --note "Check session handling"
wk new bug "Memory leak in worker"
wk new feature "User authentication"
wk new task "Port feature" --link "https://github.com/org/repo/issues/123"
wk new task "Multi-labeled" --label a,b,c           # comma-separated labels

# Start work (todo → in_progress)
wk start <id>...

# Complete work (in_progress → done, or todo → done with --reason)
wk done <id>...
wk done <id>... --reason "already fixed"   # prior: todo directly to done

# Close without completing (any → closed, requires reason)
wk close <id>... --reason "duplicate of prj-a3f2"

# Return to todo (in_progress/done/closed → todo)
wk reopen <id>...                            # from in_progress: no reason needed
wk reopen <id>... --reason "regression found" # from done/closed: reason required

# Edit issue description, title, or type
wk edit <id> description "new description"   # Update description
wk edit <id> title "new title"               # Update title
wk edit <id> type <type>                     # Change type (feature|task|bug|chore)
```

### Viewing Issues

```bash
# List issues (default: open issues - todo + in_progress)
wk list [--status/-s <status>[,<status>...]]   # todo|in_progress|done|closed
        [--type/-t <type>[,<type>...]]         # feature|task|bug|chore
        [--label/-l <label>[,<label>...]]...   # repeatable
        [--blocked]         # show only blocked issues
        [--output/-o text|json|ids]          # output format (default: text)
# Sort order: priority ASC (0=highest first), then created_at DESC (newest first)

# Show ready issues (unblocked todo items only)
wk ready [--type/-t <type>[,<type>...]]        # feature|task|bug|chore
         [--label/-l <label>[,<label>...]]...  # repeatable
         [--output/-o text|json]            # output format (default: text)
# Note: ready = unblocked todo by definition (no --status, --all, or --blocked flags)
# Sort order:
#   1. Recent issues (created <48h ago) come first, sorted by priority ASC
#   2. Old issues (created >=48h ago) come after, sorted by created_at ASC (oldest first)
#   3. Tiebreaker: created_at ASC

# Filter logic:
#   Comma-separated = OR (any match):  --label mod:wkrs,mod:wkgo
#   Repeated flags = AND (all match):  --label urgent --label security
#   Cross-filter = AND:                --status todo --label auth
#
# Examples:
wk list --status todo,in_progress              # todo OR in_progress
wk list --label mod:wkrs,mod:wkgo              # wkrs OR wkgo module
wk list --label mod:wkrs,mod:wkgo --label urgent   # (wkrs OR wkgo) AND urgent
wk list --type task,bug --status todo          # (task OR bug) AND todo

# Show single issue with full details (includes deps, notes, events)
wk show <id> [--output json]

# Show dependency tree rooted at an issue
wk tree <id>
# Example output:
# auth-a1b2: Build auth system
# ├── auth-c3d4: Design database schema [done]
# └── auth-e5f6: Implement login endpoint [in_progress]
#     └── (blocked by auth-c3d4)

# JSON output for list and ready commands:
# wk list --output json
{
  "issues": [
    {"id": "prj-a3f2", "issue_type": "task", "status": "todo", "title": "Example", "labels": ["label1"]}
  ]
}

# wk ready --output json
{
  "issues": [
    {"id": "prj-a3f2", "issue_type": "task", "status": "todo", "title": "Example", "labels": ["label1"]}
  ]
}
```

### Dependencies

```bash
# Add dependencies (one or more targets)
wk dep <from-id> <rel> <to-id>...
# Relationships: blocks, contains
# Examples:
wk dep prj-a3f2 blocks prj-b4c1              # a3f2 blocks b4c1
wk dep prj-a3f2 blocks prj-b4c1 prj-c5d2     # a3f2 blocks both
wk dep prj-feat contains prj-t1 prj-t2 prj-t3  # feature contains multiple tasks

# Remove dependency
wk undep <from-id> <rel> <to-id>...
```

### External Links

```bash
# Add external link to an issue
wk link <id> <url> [--reason <rel>]
# Relationships: import, blocks, tracks, tracked-by

# Examples:
wk link prj-a3f2 https://github.com/org/repo/issues/123
wk link prj-a3f2 jira://PE-5555                      # Jira shorthand
wk link prj-a3f2 https://company.atlassian.net/browse/PE-5555 --reason import

# Link types are auto-detected from URL:
# - GitHub: https://github.com/{owner}/{repo}/issues/{id}
# - Jira: https://*.atlassian.net/browse/{ID} or jira://{ID}
# - GitLab: https://gitlab.com/{path}/issues/{id}
# - Confluence: https://*.atlassian.net/wiki/... (has /wiki in path)

# Remove external link from an issue
wk unlink <id> <url>

# Examples:
wk unlink prj-a3f2 https://github.com/org/repo/issues/123
wk unlink prj-a3f2 jira://PE-5555
```

**Import validation**: When using `--reason import`, the URL must be a known provider (github, jira, gitlab) with a detectable issue ID.

### Labels

```bash
# Add label to one or more issues
wk label <id>... <label>
# Examples:
wk label prj-a3f2 project:auth
wk label prj-a3f2 prj-b4c1 prj-c5d2 urgent

# Remove label from one or more issues
wk unlabel <id>... <label>
```

### Notes

```bash
# Add note (status recorded automatically)
wk note <id> "note content"

# View notes (included in `wk show`)
# Note: Cannot add notes to closed issues
```

### Log

```bash
# View recent activity across all issues
wk log [--[no-]limit N]

# View history for a specific issue
wk log <id>
```

### Export

```bash
# Export all issues to JSONL
wk export <filepath>
```

### Import

```bash
# Import issues from file
wk import <filepath>
wk import -i <filepath>

# Import from stdin
cat issues.jsonl | wk import -
wk import < issues.jsonl

# Specify format explicitly
wk import --format wk issues.jsonl      # wk native format (default)
wk import --format bd .beads/issues.jsonl  # beads format

# Preview changes without applying
wk import --dry-run issues.jsonl

# Filter imported issues (same syntax as list)
wk import issues.jsonl --status todo,in_progress
wk import issues.jsonl --type task,bug
wk import issues.jsonl --label urgent
wk import issues.jsonl --prefix myproj    # Only import issues with prefix

# Auto-detect beads format from path
wk import path/to/.beads/issues.jsonl   # auto-detects bd format
```

**Behavior:**
- Existing issues (same ID) are updated
- New issues are created
- Collisions (same ID, different content) are detected and reported
- Missing dependencies are warned but don't fail import
- Format auto-detected from `.beads/issues.jsonl` suffix
- When importing beads format, 'epic' type is mapped to 'feature'

**Exit codes:**
- 0: Success (may include warnings)
- 1: Error (parse failure, database error)

#### bd Format Field Mapping

| bd Field | wk Mapping |
|----------|------------|
| `id` | `id` (preserved) |
| `title` | `title` |
| `description` | `description` |
| `status: "open"` | `status: "todo"` |
| `status: "in_progress"` | `status: "in_progress"` |
| `status: "closed"` | See close_reason logic below |
| `status: "blocked"` | `status: "todo"` |
| `status: "deferred"` | `status: "todo"` |
| `priority` | `label: priority:N` |
| `labels` | `labels` |
| `created_at` | `created_at` (preserved) |
| `updated_at` | `updated_at` (preserved) |
| `closed_at` | Event timestamp for Closed event |
| `close_reason` | Note (Close Reason) + Event reason |

#### bd Close Reason → Status Logic

When bd `status: "closed"`:
- If `close_reason` contains any failure word → `status: "closed"`
- Otherwise → `status: "done"`

**Failure words** (case-insensitive):
```
failed, rejected, wontfix, won't fix, canceled, cancelled,
abandoned, blocked, error, timeout, aborted
```

#### bd Dependency Type Mapping

| bd `type` | wk `relation` |
|-----------|---------------|
| `blocks` | `blocks` |
| `parent` | `tracks` |
| `parent-child` | `tracked-by` |
| `contains` | `tracks` |

#### bd Comment Mapping

| bd Comment Field | wk Note Field |
|------------------|---------------|
| `text` | `content` |
| `created_at` | `created_at` (preserved) |
| (inferred) | `status: todo` (all comments become Description notes) |

### Shell Completion

```bash
# Generate shell completion script
wk completion <shell>
# Supported shells: bash, zsh, fish, powershell

# Examples:
wk completion bash > /etc/bash_completion.d/wk
wk completion zsh > ~/.zsh/completions/_wk
wk completion fish > ~/.config/fish/completions/wk.fish
```

### Configuration Management

```bash
# Rename issue ID prefix (updates all existing issues in database)
wk config rename <old-prefix> <new-prefix>

# Examples:
wk config rename proj newproj    # Rename prefix from 'proj' to 'newproj'
wk config rename old new         # Rename prefix from 'old' to 'new'
```

**Behavior:**
- Both old and new prefix are required (since database may contain issues with multiple prefixes)
- Only issues matching the old prefix pattern are renamed
- Config file is updated only if old prefix matches the current config prefix
- All related tables are updated atomically (issues, deps, labels, notes, events, links)
- Both prefixes must be valid (2+ lowercase alphanumeric with at least one letter)
- If old and new prefix are the same, no changes are made (noop with message)

### Remote (Remote Mode)

```bash
# Show remote sync status (daemon state, connection, pending ops)
wk remote status

# Force immediate sync with remote server
wk remote sync

# Stop the background sync daemon
wk remote stop
```

**Behavior when remote is not configured:**
- All commands detect the operating mode based on config
- If no `[remote]` section exists in `.wok/config.toml`, prints:
  ```
  Not in remote mode.

  To enable remote sync, add a [remote] section to .wok/config.toml:

    [remote]
    url = "ws://your-server:7890"
  ```

**Behavior with remote configured:**
- `remote status` - Shows daemon connection status, pending ops, last sync time
- `remote sync` - Spawns daemon if not running, requests immediate sync, reports operations synced
- `remote stop` - Stops the daemon process (it auto-respawns on next command that requires sync)

## Output Format

Concise AI-focused format (not tabular):

```
$ wk list
- [task] (todo) prj-a3f2: Implement user auth
- [bug] (todo, @alice) prj-9bc1: Fix memory leak

$ wk show prj-a3f2
[task] prj-a3f2
Title: Implement user auth
Status: todo
Assignee: bob
Created: 2024-01-10 10:30
Updated: 2024-01-10 10:35
Labels: project:auth

Blocked by:
  - prj-c5d2

Blocks:
  - prj-b4c1

Tracked by:
  - prj-feat

Links:
  - [github] https://github.com/org/repo/issues/123
  - [jira] PE-5555 jira://PE-5555 (import)

Description:
  2024-01-10 10:30
    Need OAuth2 support for the login system.
    Should integrate with existing auth middleware.

Log:
  2024-01-10 10:31  labeled project:auth
  2024-01-10 10:32  noted "Need OAuth2 support..."
  2024-01-10 10:33  linked https://github.com/org/repo/issues/123
```

### Hooks (Claude Code Integration)

```bash
# Install hooks (default: local, auto-detect interactive mode)
wk hooks install

# Install to specific scope
wk hooks install local     # ./.claude/settings.local.json (default)
wk hooks install project   # ./.claude/settings.json
wk hooks install user      # ~/.claude/settings.json

# Force non-interactive mode (for scripts/AI agents)
wk hooks install -y [scope]

# Force interactive mode (TUI picker)
wk hooks install -i [scope]

# Uninstall hooks
wk hooks uninstall [scope]

# Show current hooks status
wk hooks status
```

**Mode detection:**
- If stdout is NOT a TTY → non-interactive
- If running under AI assistant (Claude Code, Codex, etc.) → non-interactive
- If CI environment detected → non-interactive
- If process is running in the background (e.g., `cmd &`) → non-interactive
- Otherwise → interactive (shows TUI picker)

**Interactive picker (minimal inline prompt):**
Simple inline radio-button prompt for selecting scope when no positional argument provided:
- Renders at current cursor position (no alternate screen)
- Shows 3 options with highlighting
- Arrow keys / j/k to navigate
- Enter to select
- q/Esc to cancel
- Total height: 5 lines (title, 3 options, help line)

**Hook installation behavior:**
The command uses smart merging to install hooks:
- Parses existing hooks configuration in the target file
- Preserves all non-wk hooks (hooks not containing "wk prime")
- Adds wk hooks only if not already present (idempotent)
- Maintains existing hook order, appends wk hooks at end of each event array

**Duplicate detection:**
A hook entry is considered a wk hook if any command in its hooks array contains "wk prime":
- Matches `wk prime` (plain command)
- Matches `/path/to/wk prime` (full path)
- Matches `wk prime --args` (with arguments)

**Uninstall behavior:**
The uninstall command also uses smart merging:
- Only removes hooks containing "wk prime"
- Preserves other hooks in the same event array
- Removes event key only if array becomes empty
- Removes `hooks` key only if object becomes empty

**Hook configuration installed:**
The command installs Claude Code hooks that integrate wk with the AI assistant's workflow:
- PreCompact: Runs `wk prime` before context compaction to preserve issue tracker context
- SessionStart: Runs `wk prime` at session start to inject issue tracker context
