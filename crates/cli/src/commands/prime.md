# Issue Tracker Active

> **Context Recovery**: Run `wok prime` after compaction, clear, or new session

## Core Rules
- Track strategic work as issues (multi-session, dependencies, discovered work)
- TodoWrite is fine for simple single-session linear tasks
- When in doubt, prefer issues to avoid losing context

## Finding Work
- `wok ready` - Find unblocked todo issues to start
- `wok list` - Show open issues (todo + in_progress)
- `wok list -s done` - Show completed issues
- `wok list --blocked` - Show blocked issues
- `wok show <id>` - Issue details with dependencies

## Creating & Updating
- `wok new [type] "title" [--note "description"] [--label label,...]` - New issue
  - Types: task (default), bug, feature
  - Priority: `--label priority:0` through `--label priority:4` (0=critical, 2=medium, 4=backlog)
  - Multiple labels: `--label a,b,c` or `--label a --label b`
- `wok start <id>` - Claim work (todo → in_progress)
- `wok done <id>` - Complete work (in_progress → done)
- `wok close <id> --reason="explanation"` - Close without completing (requires reason)
- `wok reopen <id>` - Return to todo (in_progress → todo, no reason required)
- `wok reopen <id> --reason="explanation"` - Reopen done/closed issue (requires reason)
- `wok edit <id> description "new description"` - Update description
- `wok edit <id> title "new title"` - Update title
- **Tip**: When creating multiple issues, use parallel subagents for efficiency

## Dependencies & Blocking
- `wok dep <blocker> blocks <blocked>` - Add dependency (A blocks B)
- `wok dep <blocked> blocked-by <blockers>` - Add dependency (A blocked by B C D)
- `wok dep <feature> contains <task1> <task2>` - Feature contains tasks
- `wok dep <task> tracked-by <feature>` - Task tracked by feature
- `wok undep <from> <rel> <to>` - Remove dependency
- `wok tree <id>` - Show dependency tree

## Labels & Notes
- `wok label <id> <label>` - Add label (e.g., `priority:1`, `backend`)
- `wok unlabel <id> <label>` - Remove label
- `wok note <id> "content"` - Add note (status recorded automatically)

## Common Workflows

**Starting work:**
```bash
wok ready                   # Find unblocked todo issues
wok start <id>              # Claim it
```

**Completing work:**
```bash
wok done <id>               # Mark complete
```

**Creating a feature with subtasks:**
```bash
wok new feature "User authentication"
wok new "Design auth schema"
wok new "Implement login endpoint"
wok dep <feature-id> contains <schema-id> <login-id>
```
