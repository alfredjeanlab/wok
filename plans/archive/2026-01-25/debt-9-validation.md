# Plan: Replace Stringly-typed InvalidInput with Structured ValidationError

**Root Feature:** `wok-43f5`

## Overview

Replace generic `Error::InvalidInput(String)` calls in `validate.rs` with structured enum variants that provide type-safe, machine-readable validation errors. This improves error handling, enables pattern matching on specific error types, and maintains consistent error messages.

## Project Structure

Key files involved:

```
crates/cli/src/
├── error.rs              # Add new ValidationError enum variants
├── error_tests.rs        # Add tests for new error variants
├── validate.rs           # Update to use structured errors
└── validate_tests.rs     # Update tests to verify specific errors
```

## Dependencies

No new dependencies required. Uses existing `thiserror` crate.

## Implementation Phases

### Phase 1: Define ValidationError Variants in error.rs

Add new structured variants to the `Error` enum in `crates/cli/src/error.rs`:

```rust
#[error("{field} too long ({actual} chars, max {max})")]
FieldTooLong {
    field: &'static str,
    actual: usize,
    max: usize,
},

#[error("{field} cannot be empty")]
FieldEmpty { field: &'static str },

#[error("too many labels (max {max} per issue)")]
LabelLimitExceeded { max: usize },

#[error("export path cannot be empty")]
ExportPathEmpty,
```

The variants use `&'static str` for field names since they are compile-time constants (e.g., "Title", "Description", "Label").

**Verification:** `cargo check -p wk`

### Phase 2: Update validate.rs Functions

Replace each `Error::InvalidInput(format!(...))` call with the appropriate structured variant.

**validate_description:**
```rust
// Before
Err(Error::InvalidInput(format!(
    "Description too long ({} chars, max {})",
    description.len(), MAX_DESCRIPTION_LENGTH
)))

// After
Err(Error::FieldTooLong {
    field: "Description",
    actual: description.len(),
    max: MAX_DESCRIPTION_LENGTH,
})
```

**validate_label:**
```rust
Err(Error::FieldTooLong {
    field: "Label",
    actual: label.len(),
    max: MAX_LABEL_LENGTH,
})
```

**validate_assignee:**
```rust
// Empty case
Err(Error::FieldEmpty { field: "Assignee" })

// Too long case
Err(Error::FieldTooLong {
    field: "Assignee",
    actual: trimmed.len(),
    max: MAX_ASSIGNEE_LENGTH,
})
```

**validate_note:**
```rust
Err(Error::FieldTooLong {
    field: "Note",
    actual: note.len(),
    max: MAX_NOTE_LENGTH,
})
```

**validate_reason:**
```rust
Err(Error::FieldTooLong {
    field: "Reason",
    actual: reason.len(),
    max: MAX_REASON_LENGTH,
})
```

**validate_label_count:**
```rust
Err(Error::LabelLimitExceeded { max: MAX_LABELS_PER_ISSUE })
```

**validate_export_path:**
```rust
Err(Error::ExportPathEmpty)
```

**validate_and_normalize_title:**
```rust
// Empty case
Err(Error::FieldEmpty { field: "Title" })

// Too long case (title)
Err(Error::FieldTooLong {
    field: "Title",
    actual: normalized.title.len(),
    max: MAX_TITLE_LENGTH,
})

// Too long case (extracted description)
Err(Error::FieldTooLong {
    field: "Extracted description",
    actual: desc.len(),
    max: MAX_DESCRIPTION_LENGTH,
})
```

**Verification:** `cargo check -p wk`

### Phase 3: Add Error Display Tests

Add tests in `crates/cli/src/error_tests.rs` to verify the new error messages:

```rust
#[test]
fn test_error_field_too_long_display() {
    let err = Error::FieldTooLong {
        field: "Description",
        actual: 15000,
        max: 10000,
    };
    let msg = err.to_string();
    assert!(msg.contains("Description"));
    assert!(msg.contains("15000"));
    assert!(msg.contains("10000"));
    assert!(msg.contains("too long"));
}

#[test]
fn test_error_field_empty_display() {
    let err = Error::FieldEmpty { field: "Title" };
    assert!(err.to_string().contains("Title"));
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn test_error_label_limit_exceeded_display() {
    let err = Error::LabelLimitExceeded { max: 20 };
    assert!(err.to_string().contains("too many labels"));
    assert!(err.to_string().contains("20"));
}

#[test]
fn test_error_export_path_empty_display() {
    let err = Error::ExportPathEmpty;
    assert!(err.to_string().contains("export path"));
    assert!(err.to_string().contains("cannot be empty"));
}
```

**Verification:** `cargo test -p wk error`

### Phase 4: Update Validation Tests

Update `crates/cli/src/validate_tests.rs` to verify specific error variants where beneficial:

```rust
#[test]
fn test_validate_description_too_long_error_type() {
    let long_desc = "x".repeat(MAX_DESCRIPTION_LENGTH + 1);
    let result = validate_description(&long_desc);
    assert!(matches!(
        result,
        Err(Error::FieldTooLong { field: "Description", .. })
    ));
}

#[test]
fn test_validate_and_normalize_title_empty_error_type() {
    let result = validate_and_normalize_title("");
    assert!(matches!(result, Err(Error::FieldEmpty { field: "Title" })));
}
```

**Verification:** `cargo test -p wk validate`

## Key Implementation Details

1. **Field names as `&'static str`**: Using string literals for field names keeps variants simple and avoids adding a FieldKind enum. The field names match user-facing terminology ("Title", "Description", etc.).

2. **Backward-compatible messages**: The new structured error messages match the existing format exactly, so CLI output remains unchanged for users.

3. **ExportPathEmpty as dedicated variant**: Rather than using `FieldEmpty { field: "Export path" }`, a dedicated `ExportPathEmpty` variant is cleaner since export path validation is a special case not shared with other field validation.

4. **Pattern matching benefits**: Callers can now match on specific error types:
   ```rust
   match result {
       Err(Error::FieldTooLong { field, actual, max }) => {
           // Handle specific field length error
       }
       Err(Error::FieldEmpty { field }) => {
           // Handle empty field error
       }
       _ => {}
   }
   ```

5. **Keep InvalidInput for other uses**: The `InvalidInput(String)` variant remains for other parts of the codebase (parser.rs, commands, etc.) that need dynamic error messages. This refactoring is scoped specifically to `validate.rs`.

## Verification Plan

1. **Compile check**: `cargo check -p wk`
2. **Unit tests**: `cargo test -p wk`
3. **CLI specs**: `make spec-cli`
4. **Linting**: `cargo clippy -p wk`
5. **Format**: `cargo fmt --check`
6. **Message consistency**: Manually verify error messages match expected format

## Out of Scope

- Refactoring `InvalidInput` uses in other files (parser.rs, commands/*.rs)
- Adding ValidationError variants to wk_core
- Internationalization of error messages
