// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

#[test]
fn test_error_not_initialized_display() {
    let err = Error::NotInitialized;
    assert!(err.to_string().contains("not initialized"));
    assert!(err.to_string().contains("wok init"));
}

#[test]
fn test_error_already_initialized_display() {
    let err = Error::AlreadyInitialized("/path/to/work".to_string());
    assert!(err.to_string().contains("already initialized"));
    assert!(err.to_string().contains("/path/to/work"));
}

#[test]
fn test_error_issue_not_found_display() {
    let err = Error::IssueNotFound("wk-abc123".to_string());
    assert!(err.to_string().contains("issue not found"));
    assert!(err.to_string().contains("wk-abc123"));
}

#[test]
fn test_error_invalid_transition_display() {
    let err = Error::InvalidTransition {
        from: "todo".to_string(),
        to: "done".to_string(),
        valid_targets: "in_progress, closed (with reason)".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid status transition"));
    assert!(msg.contains("todo"));
    assert!(msg.contains("done"));
    assert!(msg.contains("in_progress"));
}

#[test]
fn test_error_cycle_detected_display() {
    let err = Error::CycleDetected;
    assert!(err.to_string().contains("cycle"));
}

#[test]
fn test_error_self_dependency_display() {
    let err = Error::SelfDependency;
    assert!(err.to_string().contains("self-dependency"));
}

#[test]
fn test_error_dependency_not_found_display() {
    let err = Error::DependencyNotFound {
        from: "issue-a".to_string(),
        rel: "blocks".to_string(),
        to: "issue-b".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("dependency not found"));
    assert!(msg.contains("issue-a"));
    assert!(msg.contains("blocks"));
    assert!(msg.contains("issue-b"));
}

#[test]
fn test_error_invalid_issue_type_display() {
    let err = Error::InvalidIssueType("invalid".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid issue type"));
    assert!(msg.contains("invalid"));
    assert!(msg.contains("feature, task, bug"));
}

#[test]
fn test_error_invalid_status_display() {
    let err = Error::InvalidStatus("badstatus".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid status"));
    assert!(msg.contains("badstatus"));
    assert!(msg.contains("todo, in_progress, done, closed"));
}

#[test]
fn test_error_invalid_relation_display() {
    let err = Error::InvalidRelation("depends".to_string());
    let msg = err.to_string();
    assert!(msg.contains("invalid relation"));
    assert!(msg.contains("depends"));
    assert!(msg.contains("blocks, blocked-by, tracks, tracked-by"));
}

#[test]
fn test_error_invalid_prefix_display() {
    let err = Error::InvalidPrefix;
    assert!(err.to_string().contains("invalid prefix"));
    assert!(err.to_string().contains("2+ lowercase"));
}

// Phase 1: Filter Parser Error tests
#[test]
fn test_error_filter_empty_display() {
    let err = Error::FilterEmpty;
    assert!(err.to_string().contains("empty filter expression"));
}

#[test]
fn test_error_filter_unknown_field_display() {
    let err = Error::FilterUnknownField {
        field: "badfield".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("unknown filter field"));
    assert!(msg.contains("badfield"));
}

#[test]
fn test_error_filter_invalid_operator_display() {
    let err = Error::FilterInvalidOperator {
        field: "age".to_string(),
        op: "<<".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid filter operator"));
    assert!(msg.contains("<<"));
    assert!(msg.contains("age"));
}

#[test]
fn test_error_filter_invalid_value_display() {
    let err = Error::FilterInvalidValue {
        field: "age".to_string(),
        reason: "missing value".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid filter value"));
    assert!(msg.contains("age"));
    assert!(msg.contains("missing value"));
}

#[test]
fn test_error_invalid_duration_display() {
    let err = Error::InvalidDuration {
        reason: "empty duration".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid duration"));
    assert!(msg.contains("empty duration"));
}

// Phase 2: Command Validation Error tests
#[test]
fn test_error_cancelled_display() {
    let err = Error::Cancelled;
    assert!(err.to_string().contains("cancelled"));
}

#[test]
fn test_error_required_for_display() {
    let err = Error::RequiredFor {
        context: "--reason",
        operation: "agents",
    };
    let msg = err.to_string();
    assert!(msg.contains("--reason"));
    assert!(msg.contains("required for"));
    assert!(msg.contains("agents"));
}

#[test]
fn test_error_cannot_derive_display() {
    let err = Error::CannotDerive {
        item: "prefix",
        from: "path".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("cannot derive"));
    assert!(msg.contains("prefix"));
    assert!(msg.contains("path"));
}

#[test]
fn test_error_parse_line_error_display() {
    let err = Error::ParseLineError {
        line: 42,
        reason: "invalid JSON".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("line 42"));
    assert!(msg.contains("invalid JSON"));
}

#[test]
fn test_error_invalid_scope_display() {
    let err = Error::InvalidScope {
        scope: "badscope".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid scope"));
    assert!(msg.contains("badscope"));
}

#[test]
fn test_error_tty_required_display() {
    let err = Error::TtyRequired;
    assert!(err.to_string().contains("terminal"));
    assert!(err.to_string().contains("TTY"));
}

#[test]
fn test_error_permission_denied_display() {
    let err = Error::PermissionDenied {
        target: "project".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("permission denied"));
    assert!(msg.contains("project"));
}

#[test]
fn test_error_no_input_file_display() {
    let err = Error::NoInputFile;
    assert!(err.to_string().contains("no input file"));
}

#[test]
fn test_error_invalid_timestamp_display() {
    let err = Error::InvalidTimestamp {
        reason: "created_at: parse error".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("invalid timestamp"));
    assert!(msg.contains("created_at"));
}

// Phase 3: Link and Edit Error tests
#[test]
fn test_error_link_requires_display() {
    let err = Error::LinkRequires {
        requirement: "import",
        dependency: "a known provider type",
    };
    let msg = err.to_string();
    assert!(msg.contains("import"));
    assert!(msg.contains("requires"));
    assert!(msg.contains("provider"));
}

#[test]
fn test_error_unknown_attribute_display() {
    let err = Error::UnknownAttribute {
        attr: "badattr".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("unknown attribute"));
    assert!(msg.contains("badattr"));
}

// Phase 4: Note and Lookup Error tests
#[test]
fn test_error_no_notes_to_replace_display() {
    let err = Error::NoNotesToReplace {
        issue_id: "wk-abc".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("no notes to replace"));
    assert!(msg.contains("wk-abc"));
}

#[test]
fn test_error_field_required_display() {
    let err = Error::FieldRequired { field: "Label" };
    let msg = err.to_string();
    assert!(msg.contains("Label"));
    assert!(msg.contains("required"));
}

#[test]
fn test_error_cannot_note_closed_issue_display() {
    let err = Error::CannotNoteClosedIssue;
    assert!(err.to_string().contains("cannot add notes"));
    assert!(err.to_string().contains("closed"));
}

#[test]
fn test_error_unknown_format_display() {
    let err = Error::UnknownFormat {
        format: "xml".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("unknown format"));
    assert!(msg.contains("xml"));
}

#[test]
fn test_error_cannot_create_issue_display() {
    let err = Error::CannotCreateIssue {
        reason: "no prefix".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("cannot create issue"));
    assert!(msg.contains("no prefix"));
}

#[test]
fn test_error_id_generation_failed_display() {
    let err = Error::IdGenerationFailed;
    assert!(err.to_string().contains("unique issue ID"));
    assert!(err.to_string().contains("retries"));
}

// Legacy InvalidInput test (deprecated)
#[test]
#[allow(deprecated)]
fn test_error_invalid_input_display() {
    let err = Error::InvalidInput("custom error message".to_string());
    assert_eq!(err.to_string(), "custom error message");
}

#[test]
fn test_error_config_display() {
    let err = Error::Config("missing field".to_string());
    assert!(err.to_string().contains("config error"));
    assert!(err.to_string().contains("missing field"));
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: Error = io_err.into();
    assert!(err.to_string().contains("io error"));
}

#[test]
fn test_error_from_json() {
    // Create a JSON parsing error
    let result: std::result::Result<i32, serde_json::Error> = serde_json::from_str("invalid");
    let json_err = result.unwrap_err();
    let err: Error = json_err.into();
    assert!(err.to_string().contains("json error"));
}

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
