# Plan: Unify Issue JSON Types

## Overview

Consolidate three identical JSON issue types (`ListIssueJson`, `ReadyIssueJson`, `SearchIssueJson`) into a single `IssueJson` type in the schema module. This eliminates code duplication while maintaining the separate output wrapper types that differ between commands.

## Project Structure

**Files to modify:**
```
crates/cli/src/
├── schema/
│   ├── mod.rs          # Add unified IssueJson type
│   ├── list.rs         # Remove ListIssueJson, use IssueJson
│   ├── ready.rs        # Remove ReadyIssueJson, use IssueJson
│   └── search.rs       # Remove SearchIssueJson, use IssueJson
├── commands/
│   ├── list.rs         # Remove local struct, import from schema
│   ├── ready.rs        # Remove local struct, import from schema
│   ├── search.rs       # Remove local struct, import from schema
│   └── schema_tests.rs # Update test references
```

## Dependencies

No new dependencies required. Existing dependencies:
- `schemars` - JSON Schema generation (schema module only)
- `serde` - Serialization (both schema and command modules)

## Implementation Phases

### Phase 1: Create Unified IssueJson in Schema Module

Add the unified type to `crates/cli/src/schema/mod.rs`:

```rust
/// JSON representation of an issue summary.
/// Used by list, ready, and search command outputs.
#[derive(JsonSchema, Serialize)]
pub struct IssueJson {
    /// Unique issue identifier.
    pub id: String,
    /// Classification of the issue.
    pub issue_type: IssueType,
    /// Current workflow state.
    pub status: Status,
    /// Short description of the work.
    pub title: String,
    /// Person or queue this issue is assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    /// Labels attached to the issue.
    pub labels: Vec<String>,
}
```

**Verification:** `cargo check` passes

### Phase 2: Update Schema Submodules

Update each schema submodule to use `IssueJson` from parent:

1. **`schema/list.rs`** - Remove `ListIssueJson`, update `ListOutputJson`:
   ```rust
   use super::IssueJson;

   pub struct ListOutputJson {
       pub issues: Vec<IssueJson>,
       // ... rest unchanged
   }
   ```

2. **`schema/ready.rs`** - Remove `ReadyIssueJson`, update `ReadyOutputJson`:
   ```rust
   use super::IssueJson;

   pub struct ReadyOutputJson {
       pub issues: Vec<IssueJson>,
   }
   ```

3. **`schema/search.rs`** - Remove `SearchIssueJson`, update `SearchOutputJson`:
   ```rust
   use super::IssueJson;

   pub struct SearchOutputJson {
       pub issues: Vec<IssueJson>,
       // ... rest unchanged
   }
   ```

**Verification:** `cargo check` passes, `cargo test --lib` for schema_tests

### Phase 3: Update Command Files

Replace local issue structs with imports from schema module. The key insight is that command files can reuse the schema types since they only need `Serialize` (which the schema types derive).

1. **`commands/list.rs`**:
   - Remove `ListIssueJson` struct (lines 22-32)
   - Remove `ListOutputJson` struct (lines 34-42)
   - Add import: `use crate::schema::list::{ListOutputJson};`
   - Add import: `use crate::schema::IssueJson;`
   - Update construction: `ListIssueJson { ... }` → `IssueJson { ... }`

2. **`commands/ready.rs`**:
   - Remove `ReadyIssueJson` struct (lines 23-33)
   - Remove `ReadyOutputJson` struct (lines 80-84)
   - Add import: `use crate::schema::ready::ReadyOutputJson;`
   - Add import: `use crate::schema::IssueJson;`
   - Update construction: `ReadyIssueJson { ... }` → `IssueJson { ... }`

3. **`commands/search.rs`**:
   - Remove `SearchIssueJson` struct (lines 17-27)
   - Remove `SearchOutputJson` struct (lines 29-39)
   - Add import: `use crate::schema::search::SearchOutputJson;`
   - Add import: `use crate::schema::IssueJson;`
   - Update construction: `SearchIssueJson { ... }` → `IssueJson { ... }`

**Verification:** `cargo check` passes, all command tests pass

### Phase 4: Update Tests and Cleanup

1. Update `commands/schema_tests.rs`:
   - Change `list::ListIssueJson` → `crate::schema::IssueJson` in test
   - Verify schema output still contains expected field names

2. Remove dead code:
   - Delete unused imports from schema submodules
   - Ensure no orphaned type definitions remain

3. Run full test suite:
   - `cargo test`
   - `make spec-cli`

**Verification:** All tests pass, `cargo clippy` clean

## Key Implementation Details

### Why Reuse Schema Types in Commands

The schema types derive both `JsonSchema` and `Serialize`. Since command files only need `Serialize` for JSON output, they can directly use the schema types. This approach:

- Eliminates duplicate struct definitions
- Ensures schema and runtime output always match
- Keeps `schemars` as an optional dependency (only needed when deriving schemas)

### Output Wrapper Types Remain Separate

The `*OutputJson` wrapper types differ between commands and must stay separate:

| Type | Fields |
|------|--------|
| `ListOutputJson` | issues, filters_applied?, limit? |
| `ReadyOutputJson` | issues |
| `SearchOutputJson` | issues, filters_applied?, limit?, more? |

### Schema Generation Unchanged

The `wk schema` command continues to work identically:
- `wk schema list` → generates schema for `ListOutputJson`
- `wk schema ready` → generates schema for `ReadyOutputJson`
- `wk schema search` → generates schema for `SearchOutputJson`

All three will now reference the shared `IssueJson` definition in their `$defs`.

## Verification Plan

1. **Compilation**: `cargo check` - no errors or warnings
2. **Linting**: `cargo clippy` - no new warnings
3. **Unit tests**: `cargo test` - all pass
4. **Schema tests**: Verify generated schemas contain correct field names
5. **CLI specs**: `make spec-cli` - JSON output format unchanged
6. **Manual verification**:
   - `wk list --json` produces expected output
   - `wk ready --json` produces expected output
   - `wk search foo --json` produces expected output
   - `wk schema list` generates valid schema
