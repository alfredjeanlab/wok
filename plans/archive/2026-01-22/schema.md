# Implementation Plan: `wk schema` Command

**Root Feature:** `wok-eb3c`

## Overview

Add a new `wk schema` command that outputs JSON Schema specifications for subcommands that support JSON formatted output. This enables programmatic discovery of output structures for tooling, code generation, and AI integration.

Commands with JSON output:
- `wk list` → `ListOutputJson`
- `wk show` → `IssueDetails`
- `wk ready` → `ReadyOutputJson`
- `wk search` → `SearchOutputJson`

## Project Structure

```
crates/cli/
├── Cargo.toml                    # Add schemars dependency
├── src/
│   ├── cli.rs                    # Add Schema subcommand enum
│   ├── lib.rs                    # Add schema command routing
│   ├── commands/
│   │   ├── mod.rs                # Export schema module
│   │   ├── schema.rs             # New: schema command implementation
│   │   └── schema_tests.rs       # New: unit tests
│   └── schema/                   # New: schema definitions module
│       ├── mod.rs                # Re-exports and registry
│       ├── list.rs               # List output schema
│       ├── show.rs               # Show output schema
│       ├── ready.rs              # Ready output schema
│       └── search.rs             # Search output schema
checks/specs/cli/unit/
└── schema.bats                   # New: e2e tests
```

## Dependencies

Add to `crates/cli/Cargo.toml`:

```toml
[dependencies]
schemars = "0.8"
```

The `schemars` crate provides `#[derive(JsonSchema)]` for automatic JSON Schema generation from Rust types.

## Implementation Phases

### Phase 1: Add schemars and Create Schema Types

**Goal:** Add dependency and create dedicated schema types that mirror JSON output structures.

1. Add `schemars = "0.8"` to `crates/cli/Cargo.toml`

2. Create `src/schema/mod.rs` with schema type definitions:

```rust
// Schema types for JSON output structures
// These are separate from runtime types to allow schema-specific annotations
// and to avoid adding schemars dependency to production output paths

use schemars::JsonSchema;
use serde::Serialize;

pub mod list;
pub mod show;
pub mod ready;
pub mod search;

/// Available schemas for commands with JSON output
pub const SCHEMA_COMMANDS: &[&str] = &["list", "show", "ready", "search"];
```

3. Create schema types for each command (e.g., `src/schema/list.rs`):

```rust
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;

/// Issue type classification
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    Feature,
    Task,
    Bug,
    Chore,
    Idea,
}

/// Workflow status of an issue
#[derive(JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
    Closed,
}

/// JSON representation of an issue in list output
#[derive(JsonSchema, Serialize)]
pub struct ListIssueJson {
    pub id: String,
    pub issue_type: IssueType,
    pub status: Status,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    pub labels: Vec<String>,
}

/// JSON output structure for the list command
#[derive(JsonSchema, Serialize)]
pub struct ListOutputJson {
    pub issues: Vec<ListIssueJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}
```

**Verification:** `cargo check` passes with new schema module.

### Phase 2: Implement Schema Command

**Goal:** Add the `wk schema` command with subcommands for each JSON-outputting command.

1. Add `SchemaCommand` enum to `src/cli.rs`:

```rust
/// Schema output commands
#[derive(Subcommand)]
pub enum SchemaCommand {
    /// Output JSON Schema for 'wk list' JSON output
    List,
    /// Output JSON Schema for 'wk show' JSON output
    Show,
    /// Output JSON Schema for 'wk ready' JSON output
    Ready,
    /// Output JSON Schema for 'wk search' JSON output
    Search,
}
```

2. Add `Schema` variant to main `Command` enum:

```rust
/// Output JSON Schema for commands with JSON output
#[command(subcommand)]
Schema(SchemaCommand),
```

3. Create `src/commands/schema.rs`:

```rust
use crate::cli::SchemaCommand;
use crate::error::Result;
use crate::schema::{list, show, ready, search};
use schemars::schema_for;

pub fn run(cmd: SchemaCommand) -> Result<()> {
    let schema = match cmd {
        SchemaCommand::List => schema_for!(list::ListOutputJson),
        SchemaCommand::Show => schema_for!(show::IssueDetails),
        SchemaCommand::Ready => schema_for!(ready::ReadyOutputJson),
        SchemaCommand::Search => schema_for!(search::SearchOutputJson),
    };

    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);
    Ok(())
}
```

4. Add routing in `src/lib.rs`:

```rust
Command::Schema(cmd) => commands::schema::run(cmd),
```

5. Update `src/commands/mod.rs` to export the new module.

**Verification:** `wk schema list` outputs valid JSON Schema.

### Phase 3: Complete Schema Definitions

**Goal:** Create schema types for all JSON-outputting commands.

1. `src/schema/show.rs` - Full issue details with notes, links, events:

```rust
#[derive(JsonSchema, Serialize)]
pub struct IssueDetails {
    pub id: String,
    pub issue_type: IssueType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    pub labels: Vec<String>,
    pub blockers: Vec<String>,
    pub blocking: Vec<String>,
    pub parents: Vec<String>,
    pub children: Vec<String>,
    pub notes: Vec<Note>,
    pub links: Vec<Link>,
    pub events: Vec<Event>,
}
```

2. `src/schema/ready.rs` - Ready issues output.

3. `src/schema/search.rs` - Search results with `more` field.

4. Shared types in `src/schema/mod.rs` or `src/schema/common.rs`:
   - `IssueType`, `Status`, `Action`
   - `Note`, `Link`, `Event`
   - `LinkType`, `LinkRel`

**Verification:** All four schema commands produce valid JSON Schema.

### Phase 4: Add Help and Error Handling

**Goal:** `wk schema` without subcommand shows helpful usage.

1. Update CLI definition with proper help text:

```rust
/// Output JSON Schema for commands with JSON output
///
/// Use these schemas to validate JSON output or generate type definitions.
#[command(
    subcommand,
    after_help = "Examples:\n  \
        wk schema list    Output schema for 'wk list -f json'\n  \
        wk schema show    Output schema for 'wk show <id> -f json'\n\n\
      Available schemas: list, show, ready, search"
)]
Schema(SchemaCommand),
```

2. Calling `wk schema` alone shows help (via `arg_required_else_help`).

**Verification:** `wk schema` shows available subcommands and examples.

### Phase 5: Add Tests

**Goal:** Comprehensive test coverage for schema command.

#### Rust Unit Tests (`src/commands/schema_tests.rs`)

```rust
#![allow(clippy::unwrap_used)]

use super::*;
use crate::cli::SchemaCommand;

#[test]
fn schema_list_produces_valid_json() {
    // Capture output by running the schema generation directly
    let schema = schemars::schema_for!(crate::schema::list::ListOutputJson);
    let json = serde_json::to_string_pretty(&schema).unwrap();

    assert!(json.contains("\"$schema\""));
    assert!(json.contains("\"ListOutputJson\""));
    assert!(json.contains("\"issues\""));
}

#[test]
fn schema_list_has_required_fields() {
    let schema = schemars::schema_for!(crate::schema::list::ListOutputJson);
    let json = serde_json::to_string(&schema).unwrap();

    // issues is always present (required)
    assert!(json.contains("\"required\""));
}

#[test]
fn schema_show_includes_nested_types() {
    let schema = schemars::schema_for!(crate::schema::show::IssueDetails);
    let json = serde_json::to_string(&schema).unwrap();

    // Should include definitions for nested types
    assert!(json.contains("\"Note\""));
    assert!(json.contains("\"Link\""));
    assert!(json.contains("\"Event\""));
}

#[test]
fn all_schema_commands_produce_output() {
    for cmd in [
        SchemaCommand::List,
        SchemaCommand::Show,
        SchemaCommand::Ready,
        SchemaCommand::Search,
    ] {
        // Just verify no panic - actual output tested in e2e
        let _ = match cmd {
            SchemaCommand::List => schemars::schema_for!(crate::schema::list::ListOutputJson),
            SchemaCommand::Show => schemars::schema_for!(crate::schema::show::IssueDetails),
            SchemaCommand::Ready => schemars::schema_for!(crate::schema::ready::ReadyOutputJson),
            SchemaCommand::Search => schemars::schema_for!(crate::schema::search::SearchOutputJson),
        };
    }
}
```

#### E2E Tests (`checks/specs/cli/unit/schema.bats`)

```bash
#!/usr/bin/env bats
load '../../helpers/common'

# Schema commands don't need an initialized project

@test "schema requires subcommand" {
    run "$WK_BIN" schema
    assert_failure
    assert_output --partial "Usage"
}

@test "schema list outputs valid JSON" {
    run "$WK_BIN" schema list
    assert_success

    # Validate it's JSON by parsing with jq
    echo "$output" | jq . > /dev/null
}

@test "schema list contains expected structure" {
    run "$WK_BIN" schema list
    assert_success

    # Check for key schema elements
    assert_output --partial '"$schema"'
    assert_output --partial '"issues"'
    assert_output --partial '"ListOutputJson"'
}

@test "schema show outputs valid JSON" {
    run "$WK_BIN" schema show
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema show includes nested types" {
    run "$WK_BIN" schema show
    assert_success

    assert_output --partial '"Note"'
    assert_output --partial '"Link"'
    assert_output --partial '"Event"'
}

@test "schema ready outputs valid JSON" {
    run "$WK_BIN" schema ready
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema search outputs valid JSON" {
    run "$WK_BIN" schema search
    assert_success
    echo "$output" | jq . > /dev/null
}

@test "schema search includes 'more' field" {
    run "$WK_BIN" schema search
    assert_success
    assert_output --partial '"more"'
}

@test "all schemas have $schema field" {
    for cmd in list show ready search; do
        run "$WK_BIN" schema "$cmd"
        assert_success
        assert_output --partial '"$schema"'
    done
}

@test "schema -h shows help" {
    run "$WK_BIN" schema -h
    assert_success
    assert_output --partial "list"
    assert_output --partial "show"
    assert_output --partial "ready"
    assert_output --partial "search"
}
```

**Verification:**
- `cargo test` passes with schema tests
- `make spec ARGS='--file cli/unit/schema.bats'` passes

### Phase 6: Documentation and Finalization

**Goal:** Update help text and ensure consistency.

1. Add schema command to `COMMANDS_HELP` in `cli.rs`:

```rust
const COMMANDS_HELP: &str = "\
...
Setup & Configuration:
  init        Initialize issue tracker
  schema      Output JSON Schema for commands
  ...";
```

2. Run full test suite:
   - `cargo check`
   - `cargo clippy`
   - `cargo test`
   - `make spec-cli`

3. Verify landing checklist items from `crates/cli/CLAUDE.md`.

**Verification:** All tests pass, no warnings, help displays correctly.

## Key Implementation Details

### Schema Type Strategy

Two approaches are possible:

**Option A: Separate Schema Types (Recommended)**
- Create dedicated types in `src/schema/` that mirror output types
- Pros: No runtime dependency on schemars, clean separation
- Cons: Some duplication with output types

**Option B: Add JsonSchema to Output Types**
- Add `#[derive(JsonSchema)]` to existing types in commands
- Pros: No duplication, single source of truth
- Cons: Adds schemars to hot path, requires changes to existing types

Recommend **Option A** for:
- Keeping schemars out of normal execution path
- Allowing schema-specific documentation/annotations
- Avoiding changes to existing, tested code

### Schema Naming Convention

- Root schema type name matches command output (e.g., `ListOutputJson`)
- Nested types use clear, consistent names (e.g., `ListIssueJson` vs `SearchIssueJson`)
- Enum variants use `snake_case` per existing convention

### DateTime Handling

`schemars` handles `chrono::DateTime<Utc>` automatically, generating:
```json
{
  "type": "string",
  "format": "date-time"
}
```

## Verification Plan

### Unit Tests
- [ ] Each schema command produces valid JSON Schema
- [ ] Required fields are marked as required
- [ ] Optional fields use `skip_serializing_if`
- [ ] Nested types are included in definitions

### E2E Tests
- [ ] `wk schema` shows help
- [ ] `wk schema list|show|ready|search` produces valid JSON
- [ ] Output can be parsed by `jq`
- [ ] Schema contains expected type names and fields

### Manual Verification
- [ ] Schema validates actual command output:
  ```bash
  wk init --prefix test
  wk new task "Test"
  wk list -f json | jsonschema --instance /dev/stdin <(wk schema list)
  ```

### Integration
- [ ] `cargo check` - no errors
- [ ] `cargo clippy` - no warnings
- [ ] `cargo test` - all tests pass
- [ ] `make spec-cli` - all specs pass
- [ ] `cargo fmt` - properly formatted
