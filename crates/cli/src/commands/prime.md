# Issue Tracker Active

> **Context Recovery**: Run `wk prime` after compaction, clear, or new session

## Core Rules
- Track strategic work as issues (multi-session, dependencies, discovered work)
- TodoWrite is fine for simple single-session linear tasks
- When in doubt, prefer issues to avoid losing context

## Finding Work
- `wk ready` - Find unblocked todo issues to start
- `wk list` - Show open issues (todo + in_progress)
- `wk list -s done` - Show completed issues
- `wk list --blocked` - Show blocked issues
- `wk show <id>` - Issue details with dependencies

## Creating & Updating
- `wk new [type] "title" [--note "description"] [--label label,...]` - New issue
  - Types: task (default), bug, feature
  - Priority: `--label priority:0` through `--label priority:4` (0=critical, 2=medium, 4=backlog)
  - Multiple labels: `--label a,b,c` or `--label a --label b`
- `wk start <id>` - Claim work (todo → in_progress)
- `wk done <id>` - Complete work (in_progress → done)
- `wk close <id> --reason="explanation"` - Close without completing (requires reason)
- `wk reopen <id>` - Return to todo (in_progress → todo, no reason required)
- `wk reopen <id> --reason="explanation"` - Reopen done/closed issue (requires reason)
- `wk edit <id> description "new description"` - Update description
- `wk edit <id> title "new title"` - Update title
- **Tip**: When creating multiple issues, use parallel subagents for efficiency

## Dependencies & Blocking
- `wk dep <blocker> blocks <blocked>` - Add dependency (A blocks B)
- `wk dep <blocked> blocked-by <blockers>` - Add dependency (A blocked by B C D)
- `wk dep <feature> contains <task1> <task2>` - Feature contains tasks
- `wk dep <task> tracked-by <feature>` - Task tracked by feature
- `wk undep <from> <rel> <to>` - Remove dependency
- `wk tree <id>` - Show dependency tree

## Labels & Notes
- `wk label <id> <label>` - Add label (e.g., `priority:1`, `backend`)
- `wk unlabel <id> <label>` - Remove label
- `wk note <id> "content"` - Add note (status recorded automatically)

## Common Workflows

**Starting work:**
```bash
wk ready                   # Find unblocked todo issues
wk start <id>              # Claim it
```

**Completing work:**
```bash
wk done <id>               # Mark complete
```

**Creating a feature with subtasks:**
```bash
wk new feature "User authentication"
wk new "Design auth schema"
wk new "Implement login endpoint"
wk dep <feature-id> contains <schema-id> <login-id>
```
