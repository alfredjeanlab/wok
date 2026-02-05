// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::error::{Error, Result};
use crate::normalize::{normalize_title, trim_field, NormalizedTitle};

// Input length limits
pub const MAX_TITLE_LENGTH: usize = 500;
pub const MAX_DESCRIPTION_LENGTH: usize = 1_000_000;
pub const MAX_LABEL_LENGTH: usize = 100;
pub const MAX_NOTE_LENGTH: usize = 200_000;
pub const MAX_REASON_LENGTH: usize = 500;
pub const MAX_LABELS_PER_ISSUE: usize = 20;
pub const MAX_ASSIGNEE_LENGTH: usize = 100;

/// Validate that a description is within length limits
pub fn validate_description(description: &str) -> Result<()> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Description",
            actual: description.len(),
            max: MAX_DESCRIPTION_LENGTH,
        });
    }
    Ok(())
}

/// Validate that a label is within length limits
pub fn validate_label(label: &str) -> Result<()> {
    if label.len() > MAX_LABEL_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Label",
            actual: label.len(),
            max: MAX_LABEL_LENGTH,
        });
    }
    Ok(())
}

/// Validate that an assignee is valid (non-empty after trimming, within length limits)
pub fn validate_assignee(assignee: &str) -> Result<()> {
    let trimmed = assignee.trim();
    if trimmed.is_empty() {
        return Err(Error::FieldEmpty { field: "Assignee" });
    }
    if trimmed.len() > MAX_ASSIGNEE_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Assignee",
            actual: trimmed.len(),
            max: MAX_ASSIGNEE_LENGTH,
        });
    }
    Ok(())
}

/// Validate that a note is within length limits
pub fn validate_note(note: &str) -> Result<()> {
    if note.len() > MAX_NOTE_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Note",
            actual: note.len(),
            max: MAX_NOTE_LENGTH,
        });
    }
    Ok(())
}

/// Validate that a reason is within length limits
pub fn validate_reason(reason: &str) -> Result<()> {
    if reason.len() > MAX_REASON_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Reason",
            actual: reason.len(),
            max: MAX_REASON_LENGTH,
        });
    }
    Ok(())
}

/// Validate that adding a label won't exceed the label limit
pub fn validate_label_count(current_count: usize) -> Result<()> {
    if current_count >= MAX_LABELS_PER_ISSUE {
        return Err(Error::LabelLimitExceeded {
            max: MAX_LABELS_PER_ISSUE,
        });
    }
    Ok(())
}

/// Validate an export file path
pub fn validate_export_path(path: &str) -> Result<()> {
    if path.trim().is_empty() {
        return Err(Error::ExportPathEmpty);
    }
    Ok(())
}

/// Validate and normalize a title, returning processed result.
pub fn validate_and_normalize_title(title: &str) -> Result<NormalizedTitle> {
    // Normalize first - this handles truncation of long titles
    let normalized = normalize_title(title);

    // Check if result is empty after normalization
    if normalized.title.is_empty() {
        return Err(Error::FieldEmpty { field: "Title" });
    }

    // Validate length of normalized title
    if normalized.title.len() > MAX_TITLE_LENGTH {
        return Err(Error::FieldTooLong {
            field: "Title",
            actual: normalized.title.len(),
            max: MAX_TITLE_LENGTH,
        });
    }

    // Validate extracted description if present
    if let Some(ref desc) = normalized.extracted_description {
        if desc.len() > MAX_DESCRIPTION_LENGTH {
            return Err(Error::FieldTooLong {
                field: "Extracted description",
                actual: desc.len(),
                max: MAX_DESCRIPTION_LENGTH,
            });
        }
    }

    Ok(normalized)
}

/// Validate and trim a description field.
pub fn validate_and_trim_description(description: &str) -> Result<String> {
    let trimmed = trim_field(description);
    validate_description(&trimmed)?;
    Ok(trimmed)
}

/// Validate and trim a note field.
pub fn validate_and_trim_note(note: &str) -> Result<String> {
    let trimmed = trim_field(note);
    validate_note(&trimmed)?;
    Ok(trimmed)
}

/// Validate and trim a reason field.
pub fn validate_and_trim_reason(reason: &str) -> Result<String> {
    let trimmed = trim_field(reason);
    validate_reason(&trimmed)?;
    Ok(trimmed)
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;
