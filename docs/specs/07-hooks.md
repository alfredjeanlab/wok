# Issue Hooks

Issue hooks run scripts when issues change state. They are configured in `.wok/hooks.toml` or `.wok/hooks.json`.

## Configuration

### TOML Format

```toml
# .wok/hooks.toml

[[hooks]]
name = "urgent-bugs"
events = ["issue.created"]
filter = "-t bug -l urgent"
run = "./scripts/page-oncall.sh"

[[hooks]]
name = "state-changes"
events = ["issue.done", "issue.closed", "issue.reopened"]
filter = "-l external"
run = "./scripts/sync-jira.sh"

[[hooks]]
name = "audit-everything"
events = ["issue.*"]
run = "./scripts/audit-log.sh"
```

### JSON Format

```json
{
  "hooks": [
    {
      "name": "urgent-bugs",
      "events": ["issue.created"],
      "filter": "-t bug -l urgent",
      "run": "./scripts/page-oncall.sh"
    }
  ]
}
```

Both formats can coexist - hooks from both files are merged.

## Hook Schema

| Field   | Required | Description |
|---------|----------|-------------|
| name    | Yes      | Unique identifier for the hook |
| events  | Yes      | Array of event patterns |
| filter  | No       | CLI filter syntax string |
| run     | Yes      | Command to execute |

## Event Types

Derived from the `Action` enum:

- `issue.created` - Issue was created
- `issue.edited` - Title or type changed
- `issue.started` - Work began (status -> in_progress)
- `issue.stopped` - Work paused (status -> todo)
- `issue.done` - Issue completed (status -> done)
- `issue.closed` - Closed without completion
- `issue.reopened` - Reopened from done/closed
- `issue.labeled` - Label added
- `issue.unlabeled` - Label removed
- `issue.assigned` - Issue assigned
- `issue.unassigned` - Assignment removed
- `issue.noted` - Note added
- `issue.linked` - External link added
- `issue.unlinked` - External link removed
- `issue.related` - Dependency added
- `issue.unrelated` - Dependency removed
- `issue.unblocked` - Blocking issue resolved

Special pattern: `issue.*` matches all events.

## Filter Syntax

Reuses the CLI filter arg syntax:

| Flag | Long Form    | Description |
|------|--------------|-------------|
| `-t` | `--type`     | Issue type (bug, task, etc.) |
| `-l` | `--label`    | Label match (prefix `!` for NOT) |
| `-s` | `--status`   | Status (todo, in_progress, done, closed) |
| `-a` | `--assignee` | Assigned to |
| `-p` | `--prefix`   | ID prefix |

Examples:
- `-t bug,task` - Type is bug OR task
- `-l urgent -l !wip` - Has urgent AND not wip
- `-s todo,in_progress` - Status is todo OR in_progress

## Execution Model

**Fire-and-forget:**
- Process spawned and detached immediately
- No waiting for completion
- No exit code checking
- No timeouts

## Script Interface

Scripts receive:

### JSON via stdin

```json
{
  "event": "issue.created",
  "timestamp": "2024-01-15T10:30:00Z",
  "issue": {
    "id": "prj-a1b2",
    "type": "bug",
    "title": "Fix login bug",
    "status": "todo",
    "assignee": "alice",
    "labels": ["urgent", "backend"]
  },
  "change": {
    "old_value": null,
    "new_value": null,
    "reason": null
  }
}
```

### Environment Variables

- `WOK_EVENT` - Event name (e.g., "issue.created")
- `WOK_ISSUE_ID` - Issue ID
- `WOK_ISSUE_TYPE` - Issue type
- `WOK_ISSUE_STATUS` - Current status
- `WOK_CHANGE_VALUE` - New value from the change (if any)

## CLI Commands

### List Hooks

```bash
wok hook list              # Show configured hooks
wok hook list -o json      # Output as JSON
```

### Test Hook

```bash
wok hook test my-hook prj-1              # Test with created event
wok hook test my-hook prj-1 --event done # Test with specific event
```

Output indicates whether the hook would fire for the given issue.

## Integration

Hooks are triggered automatically after events are logged in `apply_mutation()`. Errors during hook execution are logged as warnings but don't fail the underlying operation.
