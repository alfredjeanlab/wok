# Core Concepts

## Issue Types

- `feature` - Large feature or initiative (can contain tasks/bugs)
- `task` - Unit of work
- `bug` - Defect to fix
- `chore` - Maintenance work (refactoring, cleanup, dependency updates)

## Status State Machine

```
     ┌─────────────────────────────────────┐
     │                                     │
     ▼         start                       │ reopen
  ┌──────┐ ──────────► ┌─────────────┐     │ (reason required)
  │ todo │             │ in_progress │     │
  └──────┘ ◄────────── └─────────────┘ ────┘
     ▲         reopen         │
     │ close                  │ done
     │ (skip)      done       │
     │             (prior)    │
     │               ╲        ▼
  ┌────────┐          ╲► ┌──────┐
  │ closed │ ───────────►│ done │
  └────────┘   reopen    └──────┘
            (reason required)
```

**Transitions:**
- `todo` → `in_progress` (start work)
- `todo` → `closed` (close without doing - requires reason)
- `todo` → `done` (prior - already complete, requires reason)
- `in_progress` → `todo` (reopen, return to backlog)
- `in_progress` → `done` (complete work)
- `in_progress` → `closed` (abandon - requires reason)
- `done` → `todo` (reopen - requires reason)
- `closed` → `todo` (reopen - requires reason)

## Dependencies (Hierarchical)

Issues can have relationships:
- `A blocks B` = B should wait for A (informational, affects `ready` command and `--blocked` filtering)
- `A contains B` = A contains B (stored as `tracks`, inverse stored as `tracked-by`)

Dependencies are informational only - they don't prevent status transitions. Blocking is hierarchical: if A blocks B and B blocks C, then A transitively blocks C. Cannot create cycles in `blocks` relationships.

## Notes by Status

Notes have semantic meaning based on when they're added:
- **todo notes**: Context/requirements before work starts (displayed as "Description")
- **in_progress notes**: Progress updates, findings, blockers during work (displayed as "Progress")
- **done notes**: Summary of what was accomplished, learnings (displayed as "Summary")

**Note**: Closed issues cannot have notes added. The close reason captures why the issue was closed.

## External Links

Issues can have external links to other issue trackers:

**Link Types** (auto-detected from URL):
- `github` - GitHub issues (https://github.com/{owner}/{repo}/issues/{id})
- `jira` - Jira issues (https://*.atlassian.net/browse/{PROJECT-ID} or jira://{ID} shorthand)
- `gitlab` - GitLab issues (https://gitlab.com/{path}/issues/{id})
- `confluence` - Confluence pages (https://*.atlassian.net/wiki/...)

**Link Relations** (optional, specified with `--reason`):
- `import` - Issue was imported from this external source (requires known provider + detectable ID)
- `blocks` - External issue blocks this issue
- `tracks` - This issue tracks the external issue
- `tracked-by` - This issue is tracked by the external issue

**URL Detection Priority**:
1. `jira://` shorthand (explicit)
2. Confluence (atlassian.net URLs with `/wiki/` in path)
3. GitHub
4. GitLab
5. Jira (atlassian.net/browse/...)
6. Unknown (URL is stored, no type detection)
