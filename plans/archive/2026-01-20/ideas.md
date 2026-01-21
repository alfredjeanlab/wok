# Idea Issue Type Implementation Plan

**Root Feature:** `wok-14bc`

## Overview

Add a new `idea` issue type to wok for capturing early-stage thoughts, brainstorms, and proposals that haven't yet been refined into actionable work items. Ideas represent potential work that may or may not become features, tasks, bugs, or chores after further discussion and refinement.

**Semantic purpose**: Ideas bridge the gap between informal thoughts and committed work. Unlike `task` (concrete work) or `feature` (planned initiative), an `idea` signals "this might be worth doing" without commitment.

## Project Structure

Files requiring modification:

```
crates/
├── core/src/
│   ├── issue.rs           # Add Idea variant to IssueType enum
│   └── error.rs           # Update InvalidIssueType hint message
├── cli/src/
│   └── cli.rs             # Update CLI help examples
docs/
└── specs/
    └── 02-core-concepts.md  # Document the new type
checks/
└── specs/
    ├── cli/unit/new.bats    # Add idea creation tests
    ├── cli/unit/list.bats   # Add idea filtering tests
    └── cli/unit/edit.bats   # Add type change tests
```

No new files required - this is a purely additive change to an existing enum.

## Dependencies

No new external dependencies required. The existing infrastructure handles all issue types generically.

## Implementation Phases

### Phase 1: Core Type Definition

**Goal**: Add the `Idea` variant to the `IssueType` enum in wk-core.

**Files to modify**:
- `crates/core/src/issue.rs`
- `crates/core/src/error.rs`

**Changes to `crates/core/src/issue.rs`**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// Large feature or initiative containing multiple tasks.
    Feature,
    /// Standard unit of work.
    Task,
    /// Defect or problem to fix.
    Bug,
    /// Maintenance work (refactoring, cleanup, dependency updates).
    Chore,
    /// Early-stage thought or proposal, not yet refined into actionable work.
    Idea,
}

impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Feature => "feature",
            IssueType::Task => "task",
            IssueType::Bug => "bug",
            IssueType::Chore => "chore",
            IssueType::Idea => "idea",
        }
    }
}

impl FromStr for IssueType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "feature" => Ok(IssueType::Feature),
            "task" => Ok(IssueType::Task),
            "bug" => Ok(IssueType::Bug),
            "chore" => Ok(IssueType::Chore),
            "idea" => Ok(IssueType::Idea),
            _ => Err(Error::InvalidIssueType(s.to_string())),
        }
    }
}
```

**Changes to `crates/core/src/error.rs`**:

```rust
#[error("invalid issue type: '{0}'\n  hint: valid types are: feature, task, bug, chore, idea")]
InvalidIssueType(String),
```

**Milestone**: `cargo test -p wk-core` passes, including existing IssueType tests.

---

### Phase 2: Unit Tests

**Goal**: Add comprehensive tests for the new type.

**Files to modify**:
- `crates/core/src/issue_tests.rs`

**New tests**:

```rust
#[test]
fn parse_idea_type() {
    assert_eq!("idea".parse::<IssueType>().unwrap(), IssueType::Idea);
    assert_eq!("IDEA".parse::<IssueType>().unwrap(), IssueType::Idea);
    assert_eq!("Idea".parse::<IssueType>().unwrap(), IssueType::Idea);
}

#[test]
fn idea_display() {
    assert_eq!(IssueType::Idea.to_string(), "idea");
    assert_eq!(IssueType::Idea.as_str(), "idea");
}

#[test]
fn idea_serialization() {
    let idea = IssueType::Idea;
    let json = serde_json::to_string(&idea).unwrap();
    assert_eq!(json, "\"idea\"");

    let parsed: IssueType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, IssueType::Idea);
}
```

**Milestone**: All IssueType unit tests pass.

---

### Phase 3: CLI Help Updates

**Goal**: Update CLI examples to include the `idea` type.

**Files to modify**:
- `crates/cli/src/cli.rs`

**Updates needed**:

1. `wk new` command help:
   ```
   wk new idea "Better caching strategy"  Create idea for future consideration
   ```

2. `wk list` command help (if type filtering examples exist)

3. `wk edit` command help (if type change examples exist)

**Milestone**: `wk help new` shows idea example.

---

### Phase 4: Documentation

**Goal**: Document the new issue type in specs.

**Files to modify**:
- `docs/specs/02-core-concepts.md`

**Changes**:

```markdown
## Issue Types

- `feature` - Large feature or initiative (can contain tasks/bugs)
- `task` - Unit of work
- `bug` - Defect to fix
- `chore` - Maintenance work (refactoring, cleanup, dependency updates)
- `idea` - Early-stage thought or proposal, not yet actionable
```

**Milestone**: Documentation reflects new type.

---

### Phase 5: Specification Tests

**Goal**: Add bats tests for idea type behavior.

**Files to modify**:
- `checks/specs/cli/unit/new.bats`
- `checks/specs/cli/unit/list.bats`
- `checks/specs/cli/unit/edit.bats`

**New tests in `new.bats`**:

```bash
@test "new: creates idea with explicit type" {
    run wk new idea "Better caching"
    assert_success
    assert_output --partial "[idea]"
}
```

**New tests in `list.bats`**:

```bash
@test "list: filters by idea type" {
    wk new idea "Idea 1"
    wk new task "Task 1"

    run wk list -t idea
    assert_success
    assert_output --partial "Idea 1"
    refute_output --partial "Task 1"
}
```

**New tests in `edit.bats`**:

```bash
@test "edit: changes type to idea" {
    id=$(wk new "My task" | grep -oE 'prj-[a-f0-9]+')

    run wk edit "$id" type idea
    assert_success

    run wk show "$id"
    assert_output --partial "[idea]"
}

@test "edit: changes idea to task" {
    id=$(wk new idea "My idea" | grep -oE 'prj-[a-f0-9]+')

    run wk edit "$id" type task
    assert_success

    run wk show "$id"
    assert_output --partial "[task]"
}
```

**Milestone**: `make spec-cli` passes with new tests.

## Key Implementation Details

### Serialization

The `#[serde(rename_all = "snake_case")]` attribute automatically handles serialization:
- Rust enum: `IssueType::Idea`
- JSON/storage: `"idea"`

No additional serialization code needed.

### Database Compatibility

Ideas are stored as `"idea"` in the `type TEXT` column. Since the database uses string storage (not an enum constraint), no schema migration is needed. Existing databases will seamlessly support ideas.

### HLC Conflict Resolution

Type changes use `last_type_hlc` for conflict resolution. No changes needed - the existing mechanism handles all `IssueType` variants.

### Filtering

The generic filtering infrastructure (`parse_filter_groups`, `matches_filter_groups`) works with any `IssueType` variant via `FromStr`. No filter code changes needed.

### Default Type

The default type when creating issues without an explicit type remains `Task`. Ideas must be explicitly specified:
```bash
wk new "My task"           # Creates task (default)
wk new idea "My idea"      # Creates idea (explicit)
```

## Verification Plan

### Unit Tests

```bash
cargo test -p wk-core issue_type
```

Verify:
- Parsing "idea" (case-insensitive)
- Display formatting
- JSON serialization round-trip
- Equality comparisons

### Integration Tests

```bash
make spec-cli ARGS='--filter "idea"'
```

Verify:
- `wk new idea "title"` creates idea
- `wk list -t idea` filters correctly
- `wk edit <id> type idea` changes type
- `wk show <id>` displays `[idea]` prefix
- Multiple type filtering: `wk list -t idea,task`

### Manual Testing

```bash
# Create idea
wk new idea "Improve CLI help system"

# List only ideas
wk list -t idea

# Convert idea to feature after refinement
wk edit prj-xxxx type feature

# Verify in JSON output
wk list -t idea --json | jq '.[] | .issue_type'
```

### Sync Testing

```bash
# Start server
wk-remote --bind 127.0.0.1:7890 --data /tmp/server

# Client 1: Create idea
wk new idea "Remote caching"

# Client 2: Verify sync
wk list -t idea  # Should show the idea
```

## Possible Impacts

### Positive

1. **Better workflow fit**: Users can capture early thoughts without cluttering actionable backlogs
2. **Refinement pipeline**: Ideas → Features/Tasks provides a natural progression
3. **Filter separation**: `wk list -t idea` keeps brainstorms separate from committed work

### Risks & Mitigations

1. **Existing databases**: No risk - string-based storage handles new types gracefully
2. **Sync compatibility**: No risk - HLC mechanisms are type-agnostic
3. **Tooling**: Any external tools parsing wk output may need updates to recognize "idea" type

### Breaking Changes

None. This is a purely additive change:
- Existing issue types unaffected
- Existing issues retain their types
- CLI syntax unchanged (just adds new valid value)
- API/JSON format unchanged (just adds new enum value)

### Future Considerations

1. **Idea-specific commands**: Could add `wk refine <id>` to convert idea to actionable type
2. **Idea expiration**: Could add automatic archival of stale ideas
3. **Voting/prioritization**: Could add lightweight voting for ideas
4. **Relationship**: Ideas could have special "inspired" relationship to resulting features

These are out of scope for initial implementation but could be follow-up work.
