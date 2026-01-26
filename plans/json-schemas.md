# Implementation Plan: Add Missing JSON Schemas

## Overview

Add `wk schema` support for commands that output JSON but lack schema definitions:

1. `wk schema new` - Schema for `wk new -o json` output
2. `wk schema config-prefixes` - Schema for `wk config prefixes -o json` output

## Current State

Commands with `-o json` support and their schema status:

| Command | Has Schema | Output Structure |
|---------|------------|------------------|
| `list` | ✅ | `{issues: [...]}` |
| `show` | ✅ | `{id, type, title, status, ...}` |
| `ready` | ✅ | `{issues: [...]}` |
| `search` | ✅ | `{issues: [...], truncated}` |
| `new` | ❌ | `{id, type, title, status, labels, assignee}` |
| `config prefixes` | ❌ | `{default, prefixes: [...]}` |

## Project Structure

```
crates/cli/src/
├── cli.rs                    # Add New and ConfigPrefixes to SchemaCommand
├── commands/
│   └── schema.rs             # Add new schema cases
└── schema/
    ├── mod.rs                # Export new schemas
    ├── new.rs                # New: schema for wk new
    └── config_prefixes.rs    # New: schema for wk config prefixes
```

## Implementation Phases

### Phase 1: Add `wk schema new`

**Files to modify:**
- `crates/cli/src/cli.rs`
- `crates/cli/src/commands/schema.rs`
- `crates/cli/src/schema/mod.rs`

**Files to create:**
- `crates/cli/src/schema/new.rs`

**Schema definition:**

```rust
// crates/cli/src/schema/new.rs
use schemars::JsonSchema;
use serde::Serialize;

/// JSON output for `wk new -o json`
#[derive(Debug, Serialize, JsonSchema)]
pub struct NewOutputJson {
    /// Issue ID (e.g., "proj-a3f2")
    pub id: String,
    /// Issue type (feature, task, bug, chore)
    #[serde(rename = "type")]
    pub issue_type: String,
    /// Issue title
    pub title: String,
    /// Issue status (always "todo" for new issues)
    pub status: String,
    /// Labels attached to the issue
    pub labels: Vec<String>,
    /// Assignee (if any)
    pub assignee: Option<String>,
}
```

**CLI changes:**

```rust
// crates/cli/src/cli.rs - Add to SchemaCommand enum
/// Output JSON Schema for 'wok new' JSON output
New,
```

**Schema command changes:**

```rust
// crates/cli/src/commands/schema.rs
SchemaCommand::New => schema_for!(new::NewOutputJson),
```

---

### Phase 2: Add `wk schema config-prefixes`

**Files to modify:**
- `crates/cli/src/cli.rs`
- `crates/cli/src/commands/schema.rs`
- `crates/cli/src/schema/mod.rs`

**Files to create:**
- `crates/cli/src/schema/config_prefixes.rs`

**Schema definition:**

```rust
// crates/cli/src/schema/config_prefixes.rs
use schemars::JsonSchema;
use serde::Serialize;

/// JSON output for `wk config prefixes -o json`
#[derive(Debug, Serialize, JsonSchema)]
pub struct ConfigPrefixesOutputJson {
    /// The default prefix from config
    pub default: String,
    /// All prefixes in the database
    pub prefixes: Vec<PrefixJson>,
}

/// Prefix information
#[derive(Debug, Serialize, JsonSchema)]
pub struct PrefixJson {
    /// Prefix string (e.g., "proj", "api")
    pub prefix: String,
    /// Number of issues with this prefix
    pub issue_count: i64,
    /// Whether this is the default prefix
    pub is_default: bool,
}
```

**CLI changes:**

```rust
// crates/cli/src/cli.rs - Add to SchemaCommand enum
/// Output JSON Schema for 'wok config prefixes' JSON output
ConfigPrefixes,
```

---

### Phase 3: Update Help Text and Examples

**Files to modify:**
- `crates/cli/src/cli.rs`

Update the schema command's after_help:

```rust
#[command(
    subcommand,
    after_help = colors::examples("\
Examples:
  wok schema list              Output schema for 'wok list -o json'
  wok schema show              Output schema for 'wok show <id> -o json'
  wok schema new               Output schema for 'wok new -o json'
  wok schema config-prefixes   Output schema for 'wok config prefixes -o json'

Available schemas: list, show, ready, search, new, config-prefixes")
)]
Schema(SchemaCommand),
```

---

### Phase 4: Tests

**Files to create/modify:**
- `crates/cli/src/commands/schema_tests.rs`
- `tests/specs/cli/unit/schema.bats` (if exists, otherwise create)

**Unit tests:**

```rust
#[test]
fn test_schema_new_outputs_valid_json() {
    // Similar to existing schema tests
}

#[test]
fn test_schema_config_prefixes_outputs_valid_json() {
    // Similar to existing schema tests
}
```

**Spec tests:**

```bash
@test "schema new outputs valid JSON schema" {
    run "$WK_BIN" schema new
    assert_success
    echo "$output" | jq -e '.type == "object"' >/dev/null
    echo "$output" | jq -e '.properties.id' >/dev/null
}

@test "schema config-prefixes outputs valid JSON schema" {
    run "$WK_BIN" schema config-prefixes
    assert_success
    echo "$output" | jq -e '.type == "object"' >/dev/null
    echo "$output" | jq -e '.properties.prefixes' >/dev/null
}
```

## Verification Plan

### Unit Tests

```bash
cargo test -p wk schema
```

### Spec Tests

```bash
make spec-cli ARGS='--filter "schema"'
```

### Manual Testing

```bash
# Verify new schemas work
wk schema new | jq .
wk schema config-prefixes | jq .

# Verify output matches schema
wk init --prefix test --local
wk new "Test" -o json | jq .
wk config prefixes -o json | jq .
```

### Checklist

- [ ] `cargo check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] `make spec-cli` passes
- [ ] `cargo fmt --check` passes
- [ ] Help text lists all available schemas
- [ ] New schemas validate actual command output
