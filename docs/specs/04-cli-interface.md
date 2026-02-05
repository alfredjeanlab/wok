# CLI Interface

## Command Structure

```
wok [-C <path>] <command> [args] [options]
```

### Global Options

```bash
# Run as if wok was started in <path>
wok -C <path> <command>
# Examples:
wok -C /path/to/project list
wok -C ../other-repo show prj-a1b2
```

## Commands

### Help

```bash
# General help
wok help
wok -h / wok --help          # (hidden aliases)

# Subcommand help
wok help <command>
wok <command> -h/--help     # (hidden aliases)
# Example: wok help dep
```

### Onboarding

```bash
# Output issue tracker workflow template (useful for AI agents)
wok prime
```

The `prime` command outputs a markdown template with common commands and workflows.
It works without initialization (no `.wok/` directory required) and is useful for:
- AI agent context priming at session start
- Quick reference for available commands
- Onboarding new users to the issue tracker

### Setup

```bash
# Initialize issue tracker (prefix defaults to directory name)
wok init

# Initialize with explicit prefix
wok init --prefix prj

# Initialize at specific path
wok init --path /path/to/shared --prefix prj

# Initialize with workspace link only (no local database)
# Note: workspace directory must exist
wok init --workspace /path/to/workspace

# Initialize with workspace and explicit prefix
wok init --workspace /path/to/workspace --prefix prj
```

**Workspace validation**: When `--workspace` is specified, the workspace directory must exist. The command fails with a clear error if the path does not exist.

### Issue Lifecycle

```bash
# Create issue (type defaults to "task")
wok new [type] <title> [--label <label>[,<label>...]]... [--note "..."] [--link <url>]...
                       [--assignee/-a <name>] [--blocks <ids>] [--blocked-by <ids>]
                       [--tracks <ids>] [--tracked-by <ids>] [--prefix <prefix>]
                       [--output/-o text|json|id]
# Examples:
wok new "Fix login bug"                              # task (default)
wok new task "Fix login bug" --label auth --note "Check session handling"
wok new bug "Memory leak in worker"
wok new feature "User authentication"
wok new task "Port feature" --link "https://github.com/org/repo/issues/123"
wok new task "Multi-labeled" --label a,b,c           # comma-separated labels
wok new "Task" -a alice                              # assign to alice
wok new bug "Fix bug" --blocks prj-1                 # blocks another issue
wok new "Task" --tracked-by prj-feat                 # tracked by a feature
wok new task "My task" -o id                         # output only ID
wok new "Task" --prefix other                        # use different prefix

# Start work (todo → in_progress)
wok start <id>...

# Complete work (in_progress → done, or todo → done with --reason)
wok done <id>...
wok done <id>... --reason "already fixed"   # prior: todo directly to done

# Close without completing (any → closed, requires reason)
wok close <id>... --reason "duplicate of prj-a3f2"

# Return to todo (in_progress/done/closed → todo)
wok reopen <id>...                            # from in_progress: no reason needed
wok reopen <id>... --reason "regression found" # from done/closed: reason required

# Edit issue description, title, type, or assignee
wok edit <id> description "new description"   # Update description
wok edit <id> title "new title"               # Update title
wok edit <id> type <type>                     # Change type (feature|task|bug|chore|idea|epic)
wok edit <id> assignee alice                  # Assign to alice
wok edit <id> assignee none                   # Clear assignment
```

### Viewing Issues

```bash
# List issues (default: open issues - todo + in_progress)
wok list [--status/-s <status>[,<status>...]]   # todo|in_progress|done|closed
        [--type/-t <type>[,<type>...]]         # feature|task|bug|chore|idea|epic
        [--label/-l <label>[,<label>...]]...   # repeatable
        [--assignee/-a <name>[,<name>...]]     # filter by assignee
        [--unassigned]                          # show only unassigned issues
        [--filter/-q <expr>]...                 # temporal filter expression
        [--blocked]                             # show only blocked issues
        [--all]                                 # ignore default status filter
        [--limit/-n <N>] [--offset <N>]         # pagination
        [--output/-o text|json|id]             # output format (default: text)
# Sort order: priority ASC (0=highest first), then created_at DESC (newest first)

# Show ready issues (unblocked todo items only)
wok ready [--type/-t <type>[,<type>...]]        # feature|task|bug|chore|idea|epic
         [--label/-l <label>[,<label>...]]...  # repeatable
         [--assignee/-a <name>[,<name>...]]    # filter by assignee
         [--unassigned]                         # show only unassigned issues
         [--all-assignees]                      # show all regardless of assignment
         [--output/-o text|json]               # output format (default: text)
# Note: ready = unblocked todo by definition (no --status, --all, or --blocked flags)
# Default: shows unassigned issues only (use --all-assignees to see all)
# Sort order:
#   1. Recent issues (created <48h ago) come first, sorted by priority ASC
#   2. Old issues (created >=48h ago) come after, sorted by created_at ASC (oldest first)
#   3. Tiebreaker: created_at ASC

# Filter logic:
#   Comma-separated = OR (any match):  --label mod:wkrs,mod:wkgo
#   Repeated flags = AND (all match):  --label urgent --label security
#   Cross-filter = AND:                --status todo --label auth
#   Negation with ! prefix:            --label '!wontfix' (exclude label)
#
# Examples:
wok list --status todo,in_progress              # todo OR in_progress
wok list --label mod:wkrs,mod:wkgo              # wkrs OR wkgo module
wok list --label mod:wkrs,mod:wkgo --label urgent   # (wkrs OR wkgo) AND urgent
wok list --type task,bug --status todo          # (task OR bug) AND todo
wok list -a alice                               # issues assigned to alice
wok list --unassigned                           # unassigned issues only
wok list -q "age < 3d"                          # issues created in last 3 days
wok list -q "updated > 1w"                      # issues not updated in 7+ days
wok list --limit 10                             # first 10 results only
wok list --all                                  # all issues (any status)
wok list --label '!wontfix'                     # exclude issues with wontfix label
wok list --label '!plan:needed'                 # exclude issues needing planning
wok list --label 'bug,!wontfix'                 # (has bug) OR (lacks wontfix)
wok list --label '!a' --label '!b'              # (lacks a) AND (lacks b)

# Filter Expressions (-q/--filter):
#   Syntax: FIELD [OPERATOR VALUE]
#   Fields: age, activity (updated), completed, skipped, closed
#   Status shortcuts: 'closed', 'skipped', 'completed' (no operator needed)
#   Operators: < <= > >= = != (or: lt lte gt gte eq ne)
#   Values: durations (30d, 1w, 24h, 5m, 10s), dates (2024-01-01), or 'now'
#   Duration units: ms, s, m, h, d, w, M (30d), y (365d)

# Show single issue with full details (includes deps, notes, events)
wok show <id> [--output json]

# Show dependency tree rooted at an issue
wok tree <id>
# Example output:
# auth-a1b2: Build auth system
# ├── auth-c3d4: Design database schema [done]
# └── auth-e5f6: Implement login endpoint [in_progress]
#     └── (blocked by auth-c3d4)

# JSON output for list and search commands returns a plain array:
# wok list --output json
[
  {"id": "prj-a3f2", "issue_type": "task", "status": "todo", "title": "Example", "labels": ["label1"]}
]

# wok ready --output json (ready still uses an object with "issues" key)
{
  "issues": [
    {"id": "prj-a3f2", "issue_type": "task", "status": "todo", "title": "Example", "labels": ["label1"]}
  ]
}
```

### Search

```bash
# Search issues by text (searches title, description, notes)
wok search <query> [--status/-s <status>[,<status>...]]
                   [--type/-t <type>[,<type>...]]
                   [--label/-l <label>[,<label>...]]...
                   [--assignee/-a <name>[,<name>...]]
                   [--filter/-q <expr>]...
                   [--limit/-n <N>] [--offset <N>]
                   [--output/-o text|json]

# Examples:
wok search "login"                    # Search for 'login' in all fields
wok search "auth" -s todo             # Search todo issues only
wok search "bug" -t bug               # Search bugs only
wok search "task" -a alice            # Search issues assigned to alice
wok search "auth" -q "age < 30d"      # Search with time filter
wok search "auth" -n 5                # Limit to 5 results
```

### Dependencies

```bash
# Add dependencies (one or more targets)
wok dep <from-id> <rel> <to-id>...
# Relationships: blocks, contains
# Examples:
wok dep prj-a3f2 blocks prj-b4c1              # a3f2 blocks b4c1
wok dep prj-a3f2 blocks prj-b4c1 prj-c5d2     # a3f2 blocks both
wok dep prj-feat contains prj-t1 prj-t2 prj-t3  # feature contains multiple tasks

# Remove dependency
wok undep <from-id> <rel> <to-id>...
```

### External Links

```bash
# Add external link to an issue
wok link <id> <url> [--reason <rel>]
# Relationships: import, blocks, tracks, tracked-by

# Examples:
wok link prj-a3f2 https://github.com/org/repo/issues/123
wok link prj-a3f2 jira://PE-5555                      # Jira shorthand
wok link prj-a3f2 https://company.atlassian.net/browse/PE-5555 --reason import

# Link types are auto-detected from URL:
# - GitHub: https://github.com/{owner}/{repo}/issues/{id}
# - Jira: https://*.atlassian.net/browse/{ID} or jira://{ID}
# - GitLab: https://gitlab.com/{path}/issues/{id}
# - Confluence: https://*.atlassian.net/wiki/... (has /wiki in path)

# Remove external link from an issue
wok unlink <id> <url>

# Examples:
wok unlink prj-a3f2 https://github.com/org/repo/issues/123
wok unlink prj-a3f2 jira://PE-5555
```

**Import validation**: When using `--reason import`, the URL must be a known provider (github, jira, gitlab) with a detectable issue ID.

### Labels

```bash
# Add label to one or more issues
wok label <id>... <label>
# Examples:
wok label prj-a3f2 project:auth
wok label prj-a3f2 prj-b4c1 prj-c5d2 urgent

# Remove label from one or more issues
wok unlabel <id>... <label>
```

### Notes

```bash
# Add note (status recorded automatically)
wok note <id> "note content"

# Replace most recent note instead of adding new
wok note <id> "updated content" --replace

# View notes (included in `wok show`)
# Note: Cannot add notes to closed issues
```

### Log

```bash
# View recent activity across all issues
wok log [--[no-]limit N]

# View history for a specific issue
wok log <id>
```

### Export

```bash
# Export all issues to JSONL
wok export <filepath>
```

### Import

```bash
# Import issues from file
wok import <filepath>
wok import -i <filepath>

# Import from stdin
cat issues.jsonl | wok import -
wok import < issues.jsonl

# Specify format explicitly
wok import --format wok issues.jsonl      # wok native format (default)
wok import --format bd .beads/issues.jsonl  # beads format

# Preview changes without applying
wok import --dry-run issues.jsonl

# Filter imported issues (same syntax as list)
wok import issues.jsonl --status todo,in_progress
wok import issues.jsonl --type task,bug
wok import issues.jsonl --label urgent
wok import issues.jsonl --prefix myproj    # Only import issues with prefix

# Auto-detect beads format from path
wok import path/to/.beads/issues.jsonl   # auto-detects bd format
```

**Behavior:**
- Existing issues (same ID) are updated
- New issues are created
- Collisions (same ID, different content) are detected and reported
- Missing dependencies are warned but don't fail import
- Format auto-detected from `.beads/issues.jsonl` suffix
- When importing beads format, 'epic' type is preserved as 'epic'

**Exit codes:**
- 0: Success (may include warnings)
- 1: Error (parse failure, database error)

#### bd Format Field Mapping

| bd Field | wok Mapping |
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

| bd `type` | wok `relation` |
|-----------|---------------|
| `blocks` | `blocks` |
| `parent` | `tracks` |
| `parent-child` | `tracked-by` |
| `contains` | `tracks` |

#### bd Comment Mapping

| bd Comment Field | wok Note Field |
|------------------|---------------|
| `text` | `content` |
| `created_at` | `created_at` (preserved) |
| (inferred) | `status: todo` (all comments become Description notes) |

### Shell Completion

```bash
# Generate shell completion script
wok completion <shell>
# Supported shells: bash, zsh, fish, powershell

# Examples:
wok completion bash > /etc/bash_completion.d/wok
wok completion zsh > ~/.zsh/completions/_wok
wok completion fish > ~/.config/fish/completions/wok.fish
```

### Schema

```bash
# Output JSON Schema for command output validation
wok schema <command>
# Available: list, show, ready, search

# Examples:
wok schema list    # Schema for 'wok list -o json'
wok schema show    # Schema for 'wok show <id> -o json'
wok schema ready   # Schema for 'wok ready -o json'
wok schema search  # Schema for 'wok search -o json'
```

Use schemas to validate JSON output or generate type definitions for tooling integration.

### Configuration Management

```bash
# List all prefixes in the database
wok config prefixes
wok config prefixes -o json       # Output as JSON
wok config prefixes -o id         # Output prefix names only

# Rename issue ID prefix (updates all existing issues in database)
wok config rename <old-prefix> <new-prefix>

# Examples:
wok config rename proj newproj    # Rename prefix from 'proj' to 'newproj'
wok config rename old new         # Rename prefix from 'old' to 'new'
```

**Behavior (`config prefixes`):**
- Lists all prefixes with their issue counts
- Marks the default prefix (from config) with "(default)"
- JSON output includes `default`, `prefixes` array with `prefix`, `issue_count`, `is_default`

**Behavior (`config rename`):**
- Both old and new prefix are required (since database may contain issues with multiple prefixes)
- Only issues matching the old prefix pattern are renamed
- Config file is updated only if old prefix matches the current config prefix
- All related tables are updated atomically (issues, deps, labels, notes, events, links, prefixes)
- Both prefixes must be valid (2+ lowercase alphanumeric with at least one letter)
- If old and new prefix are the same, no changes are made (noop with message)

### Daemon Management

```bash
# Show daemon status
wok daemon status

# Start the daemon
wok daemon start
wok daemon start --foreground  # Run in foreground for debugging

# Stop the daemon
wok daemon stop

# View daemon logs
wok daemon logs
wok daemon logs --follow       # Tail logs (like tail -f)
```

### Remote (Remote Mode)

```bash
# Show remote sync status (daemon state, connection, pending ops)
wok remote status

# Force immediate sync with remote server
wok remote sync

# Stop the background sync daemon
wok remote stop
```

**Behavior when remote is not configured:**
- All commands detect the operating mode based on config
- `remote status` - Shows configuration hint:
  ```
  Status: not applicable (no remote configured)

  To enable remote sync, add a [remote] section to .wok/config.toml:

    [remote]
    url = "ws://your-server:7890"
  ```
- `remote sync` - Silent (nothing to sync in local mode)
- `remote stop` - Prints "Not in remote mode - no daemon to stop."

**Behavior with remote configured:**
- `remote status` - Shows daemon connection status, pending ops, last sync time
- `remote sync` - Spawns daemon if not running, requests immediate sync, reports operations synced
- `remote stop` - Stops the daemon process (it auto-respawns on next command that requires sync)

## Output Format

Concise AI-focused format (not tabular):

```
$ wok list
- [task] (todo) prj-a3f2: Implement user auth
- [bug] (todo, @alice) prj-9bc1: Fix memory leak

$ wok show prj-a3f2
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
wok hooks install

# Install to specific scope
wok hooks install local     # ./.claude/settings.local.json (default)
wok hooks install project   # ./.claude/settings.json
wok hooks install user      # ~/.claude/settings.json

# Force non-interactive mode (for scripts/AI agents)
wok hooks install -y [scope]

# Force interactive mode (TUI picker)
wok hooks install -i [scope]

# Uninstall hooks
wok hooks uninstall [scope]

# Show current hooks status
wok hooks status
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
- Preserves all non-wok hooks (hooks not containing "wok prime")
- Adds wok hooks only if not already present (idempotent)
- Maintains existing hook order, appends wok hooks at end of each event array

**Duplicate detection:**
A hook entry is considered a wok hook if any command in its hooks array contains "wok prime":
- Matches `wok prime` (plain command)
- Matches `/path/to/wok prime` (full path)
- Matches `wok prime --args` (with arguments)

**Uninstall behavior:**
The uninstall command also uses smart merging:
- Only removes hooks containing "wok prime"
- Preserves other hooks in the same event array
- Removes event key only if array becomes empty
- Removes `hooks` key only if object becomes empty

**Hook configuration installed:**
The command installs Claude Code hooks that integrate wok with the AI assistant's workflow:
- PreCompact: Runs `wok prime` before context compaction to preserve issue tracker context
- SessionStart: Runs `wok prime` at session start to inject issue tracker context
