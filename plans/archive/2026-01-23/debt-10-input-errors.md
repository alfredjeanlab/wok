# Plan: Structure Remaining InvalidInput Error Variants

**Epic:** `wok-43f5`
**Root Feature:** `wok-4ac7`

## Overview

Replace remaining `Error::InvalidInput(String)` usages throughout the codebase with structured enum variants. This continues the work from debt-9 (which focused on `validate.rs`) to cover the ~40 remaining usages in parser, command, and database modules.

## Current State

After debt-9, `InvalidInput(String)` still appears in:

| Location | Count | Category |
|----------|-------|----------|
| `filter/parser.rs` | 15 | Filter expression parsing |
| `commands/lifecycle.rs` | 8 | Agent/transition validation |
| `commands/hooks.rs` | 5 | Hook configuration |
| `commands/import.rs` | 4 | Import format parsing |
| `commands/init.rs` | 2 | Prefix validation |
| `commands/link.rs` | 2 | Link validation |
| `commands/edit.rs` | 2 | Edit field validation |
| `commands/new.rs` | 2 | Issue creation |
| `commands/note.rs` | 2 | Note requirements |
| `commands/show.rs` | 1 | Display format |
| `db/notes.rs` | 1 | Note lookup |
| `lib.rs` | 2 | Label operations |

## Project Structure

```
crates/cli/src/
├── error.rs              # Add new structured variants
├── error_tests.rs        # Add tests for new variants
├── filter/
│   └── parser.rs         # Refactor to use FilterParseError variants
├── commands/
│   ├── lifecycle.rs      # Refactor agent/transition errors
│   ├── hooks.rs          # Refactor hook errors
│   ├── import.rs         # Refactor import errors
│   └── ...               # Other command files
└── ...
```

## Implementation Phases

### Phase 1: Filter Parser Errors

Add structured variants for filter expression parsing in `error.rs`:

```rust
#[error("empty filter expression")]
FilterEmpty,

#[error("unknown filter field: '{field}'")]
FilterUnknownField { field: String },

#[error("invalid filter operator '{op}' for field '{field}'")]
FilterInvalidOperator { field: String, op: String },

#[error("invalid filter value for {field}: {reason}")]
FilterInvalidValue { field: String, reason: String },

#[error("invalid duration: {reason}")]
InvalidDuration { reason: String },
```

Update `filter/parser.rs` to use these variants instead of `InvalidInput(format!(...))`.

**Verification:** `cargo test -p wk filter`

### Phase 2: Command Validation Errors

Add structured variants for common command validation:

```rust
#[error("operation cancelled")]
Cancelled,

#[error("{context} is required for {operation}")]
RequiredFor { context: String, operation: String },

#[error("cannot derive {item} from {source}")]
CannotDerive { item: &'static str, source: String },

#[error("line {line}: {reason}")]
ParseLineError { line: usize, reason: String },
```

Update command files:
- `hooks.rs`: Use `Cancelled` for user cancellation
- `lifecycle.rs`: Use `RequiredFor` for agent assignment validation
- `init.rs`: Use `CannotDerive` for prefix derivation
- `import.rs`: Use `ParseLineError` for JSONL parsing

**Verification:** `cargo test -p wk`

### Phase 3: Link and Edit Errors

Add structured variants for link/edit operations:

```rust
#[error("links require both type and URL")]
LinkIncomplete,

#[error("nothing to edit: specify at least one field")]
EditNoFields,

#[error("ambiguous issue reference: '{reference}' matches multiple issues")]
AmbiguousReference { reference: String },
```

Update:
- `commands/link.rs`: Use `LinkIncomplete`
- `commands/edit.rs`: Use `EditNoFields`, `AmbiguousReference`

**Verification:** `cargo test -p wk`

### Phase 4: Note and Lookup Errors

Add structured variants:

```rust
#[error("note {index} not found on issue {issue_id}")]
NoteNotFound { issue_id: String, index: usize },

#[error("{field} is required")]
FieldRequired { field: &'static str },
```

Update:
- `db/notes.rs`: Use `NoteNotFound`
- `commands/note.rs`, `lib.rs`: Use `FieldRequired`

**Verification:** `cargo test -p wk`

### Phase 5: Deprecate InvalidInput

After all usages are migrated:

1. Add `#[deprecated]` attribute to `InvalidInput` variant
2. Verify no direct usages remain (only in `From<wk_core::Error>`)
3. Consider whether to keep for wk_core error conversion or add more specific mappings

```rust
#[deprecated(note = "use specific error variants instead")]
#[error("{0}")]
InvalidInput(String),
```

**Verification:** `cargo check -p wk 2>&1 | grep -c "InvalidInput"`

## Key Implementation Details

1. **Preserve error messages**: New variant `#[error(...)]` strings should match the current `format!(...)` strings exactly for backwards compatibility.

2. **Use `&'static str` for compile-time strings**: Field names like "Label", "Assignee" should be `&'static str`. Dynamic values like issue IDs should be `String`.

3. **Group related errors**: Filter parsing errors share a common pattern and could alternatively be a nested `FilterError` enum, but flat variants are simpler.

4. **Keep InvalidInput for wk_core bridge**: The `From<wk_core::Error>` impl needs somewhere to map `wk_core::Error::InvalidInput`. Consider adding matching variants to wk_core in a future debt item.

## Verification Plan

1. **Compile check**: `cargo check -p wk`
2. **Unit tests**: `cargo test -p wk`
3. **CLI specs**: `make spec-cli`
4. **Linting**: `cargo clippy -p wk`
5. **Format**: `cargo fmt --check`
6. **Deprecation check**: Verify `InvalidInput` only used in From impl

## Out of Scope

- Adding structured errors to `wk_core` crate
- Internationalization of error messages
- Exit code differentiation based on error type
