# Business Rules

## Dependency Constraints

1. Cannot create self-dependency
2. Cannot create cycles in `blocks` relationships (validate on `dep`)
3. Deps are informational - they do NOT prevent status transitions
4. Completing an issue does NOT auto-remove deps (they remain for history)

## External Link Constraints

1. External links are informational (they don't affect issue workflow)
2. Multiple links can be added to the same issue
3. `--reason import` requires a known provider type (github, jira, gitlab) with a detectable issue ID
4. Unknown URLs are accepted (stored without type detection)

## Relationship Semantics

All relationships are informational. They affect filtering (`ready` command, `--blocked` flag) but don't enforce constraints:
- **blocks**: A blocks B = B should wait for A. Used by `ready` command and `list --blocked`.
- **contains**: A contains B = A contains B. Stored as `tracks` on A, `tracked-by` on B.

## Status Transitions

1. `start` - no constraints (blocking is informational)
2. `done` from `in_progress` - no constraints
3. `done` from `todo` - requires reason (prior/already complete)
4. `close` requires a reason
5. `reopen` from `in_progress` - no constraints, returns to backlog
6. `reopen` from `done`/`closed` - requires a reason, returns to todo
7. Cannot transition directly from done/closed to in_progress (use reopen then start)

## Notes

1. Notes record the current status when added
2. All notes are immutable (append-only)
3. `show` command groups notes by status with semantic labels (Description, Progress, Summary, Close Reason)
4. Cannot add notes to closed issues

## Reason Notes

When `--reason` is provided to lifecycle commands, the reason is stored as:
1. **Event** (existing): In the Log section with action context
2. **Note** (new): With semantic label based on action

| Command | Note Status | Section Label |
|---------|-------------|---------------|
| `close --reason` | `closed` | "Close Reason" |
| `done --reason` | `done` | "Summary" |
| `reopen --reason` | `todo` | "Description" |

## Input Limits

1. Issue titles: max 500 characters
2. Issue descriptions: max 1,000,000 characters
3. Note content: max 200,000 characters
4. Label names: max 100 characters
5. Reason text: max 500 characters

## Export Path Validation

1. Export filepath can be absolute or relative
2. Relative paths create files relative to current working directory
