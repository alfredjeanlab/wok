// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

#[test]
fn test_validate_and_normalize_title_ok() {
    assert!(validate_and_normalize_title("A normal title").is_ok());
    assert!(validate_and_normalize_title(&"x".repeat(MAX_TITLE_LENGTH)).is_ok());
}

#[test]
fn test_validate_and_normalize_title_long_truncates() {
    // Long titles are now truncated instead of rejected
    let long_title = "x".repeat(MAX_TITLE_LENGTH + 1);
    let result = validate_and_normalize_title(&long_title).unwrap();
    // Title should be truncated with "..."
    assert!(result.title.ends_with("..."));
    assert!(result.title.len() <= 123); // 120 + "..."
                                        // Full text should be in description
    assert!(result.extracted_description.is_some());
    assert_eq!(result.extracted_description.unwrap(), long_title);
}

#[test]
fn test_validate_description_ok() {
    assert!(validate_description("A normal description").is_ok());
    assert!(validate_description(&"x".repeat(MAX_DESCRIPTION_LENGTH)).is_ok());
}

#[test]
fn test_validate_description_too_long() {
    let long_desc = "x".repeat(MAX_DESCRIPTION_LENGTH + 1);
    assert!(validate_description(&long_desc).is_err());
}

#[test]
fn test_validate_description_empty_ok() {
    // Empty descriptions are allowed (to clear the field)
    assert!(validate_description("").is_ok());
}

#[test]
fn test_validate_label_ok() {
    assert!(validate_label("urgent").is_ok());
    assert!(validate_label(&"x".repeat(MAX_LABEL_LENGTH)).is_ok());
}

#[test]
fn test_validate_label_too_long() {
    let long_label = "x".repeat(MAX_LABEL_LENGTH + 1);
    assert!(validate_label(&long_label).is_err());
}

#[test]
fn test_validate_note_ok() {
    assert!(validate_note("A note").is_ok());
    assert!(validate_note(&"x".repeat(MAX_NOTE_LENGTH)).is_ok());
}

#[test]
fn test_validate_note_too_long() {
    let long_note = "x".repeat(MAX_NOTE_LENGTH + 1);
    assert!(validate_note(&long_note).is_err());
}

#[test]
fn test_validate_label_count_ok() {
    assert!(validate_label_count(0).is_ok());
    assert!(validate_label_count(MAX_LABELS_PER_ISSUE - 1).is_ok());
}

#[test]
fn test_validate_label_count_at_limit() {
    assert!(validate_label_count(MAX_LABELS_PER_ISSUE).is_err());
}

#[test]
fn test_validate_export_path_ok() {
    assert!(validate_export_path("issues.jsonl").is_ok());
    assert!(validate_export_path("/tmp/issues.jsonl").is_ok());
    assert!(validate_export_path("./output/issues.jsonl").is_ok());
    assert!(validate_export_path("../issues.jsonl").is_ok());
    assert!(validate_export_path("foo/../bar.jsonl").is_ok());
}

#[test]
fn test_validate_export_path_empty() {
    assert!(validate_export_path("").is_err());
    assert!(validate_export_path("   ").is_err());
}

#[test]
fn test_validate_reason_ok() {
    assert!(validate_reason("A valid reason").is_ok());
    assert!(validate_reason(&"x".repeat(MAX_REASON_LENGTH)).is_ok());
}

#[test]
fn test_validate_reason_too_long() {
    let long_reason = "x".repeat(MAX_REASON_LENGTH + 1);
    let result = validate_reason(&long_reason);
    assert!(result.is_err());
}

#[test]
fn test_validate_and_normalize_title_simple() {
    let result = validate_and_normalize_title("Hello").unwrap();
    assert_eq!(result.title, "Hello");
    assert!(result.extracted_description.is_none());
}

#[test]
fn test_validate_and_normalize_title_trimmed() {
    let result = validate_and_normalize_title("  Hello  ").unwrap();
    assert_eq!(result.title, "Hello");
    assert!(result.extracted_description.is_none());
}

#[test]
fn test_validate_and_normalize_title_with_split() {
    let result = validate_and_normalize_title("Fix the bug\n\nDescription").unwrap();
    assert_eq!(result.title, "Fix the bug");
    assert_eq!(result.extracted_description.as_deref(), Some("Description"));
}

#[test]
fn test_validate_and_normalize_title_empty_rejected() {
    assert!(validate_and_normalize_title("").is_err());
    assert!(validate_and_normalize_title("   ").is_err());
    assert!(validate_and_normalize_title("\n\n\n").is_err());
}

#[test]
fn test_validate_and_normalize_title_very_long_truncates() {
    // Very long titles are truncated and full content moves to description
    let long_title = "a".repeat(MAX_TITLE_LENGTH + 100);
    let result = validate_and_normalize_title(&long_title).unwrap();
    assert!(result.title.ends_with("..."));
    assert!(result.extracted_description.is_some());
}

#[test]
fn test_validate_and_trim_note_simple() {
    let result = validate_and_trim_note("Note content").unwrap();
    assert_eq!(result, "Note content");
}

#[test]
fn test_validate_and_trim_note_trimmed() {
    let result = validate_and_trim_note("  Note  ").unwrap();
    assert_eq!(result, "Note");
}

#[test]
fn test_validate_and_trim_note_multiline() {
    let result = validate_and_trim_note("Line1\nLine2").unwrap();
    assert_eq!(result, "Line1\nLine2");
}

#[test]
fn test_validate_and_trim_description_simple() {
    let result = validate_and_trim_description("Description").unwrap();
    assert_eq!(result, "Description");
}

#[test]
fn test_validate_and_trim_description_trimmed() {
    let result = validate_and_trim_description("  Desc  ").unwrap();
    assert_eq!(result, "Desc");
}

#[test]
fn test_validate_and_trim_reason_simple() {
    let result = validate_and_trim_reason("Reason text").unwrap();
    assert_eq!(result, "Reason text");
}

#[test]
fn test_validate_and_trim_reason_trimmed() {
    let result = validate_and_trim_reason("  Reason  ").unwrap();
    assert_eq!(result, "Reason");
}

#[test]
fn test_validate_description_too_long_error_type() {
    let long_desc = "x".repeat(MAX_DESCRIPTION_LENGTH + 1);
    let result = validate_description(&long_desc);
    assert!(matches!(
        result,
        Err(Error::FieldTooLong {
            field: "Description",
            ..
        })
    ));
}

#[test]
fn test_validate_and_normalize_title_empty_error_type() {
    let result = validate_and_normalize_title("");
    assert!(matches!(result, Err(Error::FieldEmpty { field: "Title" })));
}

#[test]
fn test_validate_label_too_long_error_type() {
    let long_label = "x".repeat(MAX_LABEL_LENGTH + 1);
    let result = validate_label(&long_label);
    assert!(matches!(
        result,
        Err(Error::FieldTooLong { field: "Label", .. })
    ));
}

#[test]
fn test_validate_label_count_exceeded_error_type() {
    let result = validate_label_count(MAX_LABELS_PER_ISSUE);
    assert!(matches!(result, Err(Error::LabelLimitExceeded { .. })));
}

#[test]
fn test_validate_export_path_empty_error_type() {
    let result = validate_export_path("");
    assert!(matches!(result, Err(Error::ExportPathEmpty)));
}
