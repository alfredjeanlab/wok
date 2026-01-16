// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::models::IssueType;
use chrono::{TimeZone, Utc};

fn create_test_issue(id: &str, title: &str, issue_type: IssueType, status: Status) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type,
        title: title.to_string(),
        description: None,
        status,
        assignee: None,
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap(),
        updated_at: Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap(),
        closed_at: None,
    }
}

fn create_test_event(issue_id: &str, action: Action) -> Event {
    Event {
        id: 1,
        issue_id: issue_id.to_string(),
        action,
        old_value: None,
        new_value: None,
        reason: None,
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 12, 0, 0).unwrap(),
    }
}

// format_issue_line tests
#[test]
fn test_format_issue_line() {
    let issue = create_test_issue("prj-1234", "Test issue", IssueType::Task, Status::Todo);
    let line = format_issue_line(&issue);
    assert!(line.contains("[task]"));
    assert!(line.contains("(todo)"));
    assert!(line.contains("prj-1234"));
    assert!(line.contains("Test issue"));
}

#[test]
fn test_format_issue_line_different_types() {
    let feature = create_test_issue("f-1", "Feature", IssueType::Feature, Status::InProgress);
    let bug = create_test_issue("b-1", "Bug", IssueType::Bug, Status::Done);

    assert!(format_issue_line(&feature).contains("[feature]"));
    assert!(format_issue_line(&feature).contains("(in_progress)"));
    assert!(format_issue_line(&bug).contains("[bug]"));
    assert!(format_issue_line(&bug).contains("(done)"));
}

#[test]
fn test_format_issue_line_with_assignee() {
    let mut issue = create_test_issue("prj-1234", "Test issue", IssueType::Task, Status::Todo);
    issue.assignee = Some("alice".to_string());
    let line = format_issue_line(&issue);
    assert!(line.contains("(todo, @alice)"));
    assert!(line.contains("[task]"));
    assert!(line.contains("prj-1234"));
}

#[test]
fn test_format_issue_line_with_assignee_in_progress() {
    let mut issue = create_test_issue(
        "prj-5678",
        "Another test",
        IssueType::Bug,
        Status::InProgress,
    );
    issue.assignee = Some("bob".to_string());
    let line = format_issue_line(&issue);
    assert!(line.contains("(in_progress, @bob)"));
}

// format_issue_details tests
#[test]
fn test_format_issue_details_minimal() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[]);

    // First line: [type] id
    assert!(output.starts_with("[task] prj-1234"));
    // Separate lines for metadata
    assert!(output.contains("Title: Test"));
    assert!(output.contains("Status: todo"));
    assert!(output.contains("Created: 2024-01-10 12:00"));
    assert!(output.contains("Updated: 2024-01-10 12:00"));
    assert!(!output.contains("Labels:"));
    assert!(!output.contains("Blocked by:"));
    assert!(!output.contains("Log:"));
    // Assignee should not appear when not set
    assert!(!output.contains("Assignee:"));
}

#[test]
fn test_format_issue_details_with_assignee() {
    let mut issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    issue.assignee = Some("alice".to_string());
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[]);

    // Assignee should appear after Status
    assert!(output.contains("Status: todo"));
    assert!(output.contains("Assignee: alice"));
}

#[test]
fn test_format_issue_details_with_labels() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let labels = vec!["urgent".to_string(), "frontend".to_string()];
    let output = format_issue_details(&issue, &labels, &[], &[], &[], &[], &[], &[], &[]);

    assert!(output.contains("Labels: urgent, frontend"));
}

#[test]
fn test_format_issue_details_with_blockers() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let blockers = vec!["prj-aaaa".to_string()];
    let output = format_issue_details(&issue, &[], &blockers, &[], &[], &[], &[], &[], &[]);

    assert!(output.contains("Blocked by:"));
    assert!(output.contains("prj-aaaa"));
}

#[test]
fn test_format_issue_details_with_blocking() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let blocking = vec!["prj-bbbb".to_string()];
    let output = format_issue_details(&issue, &[], &[], &blocking, &[], &[], &[], &[], &[]);

    assert!(output.contains("Blocks:"));
    assert!(output.contains("prj-bbbb"));
}

#[test]
fn test_format_issue_details_with_parents() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let parents = vec!["prj-parent".to_string()];
    let output = format_issue_details(&issue, &[], &[], &[], &parents, &[], &[], &[], &[]);

    assert!(output.contains("Tracked by:"));
    assert!(output.contains("prj-parent"));
}

#[test]
fn test_format_issue_details_with_children() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Feature, Status::Todo);
    let children = vec!["prj-child1".to_string(), "prj-child2".to_string()];
    let output = format_issue_details(&issue, &[], &[], &[], &[], &children, &[], &[], &[]);

    assert!(output.contains("Tracks:"));
    assert!(output.contains("prj-child1"));
    assert!(output.contains("prj-child2"));
}

#[test]
fn test_format_issue_details_with_notes() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::InProgress);
    let note = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::InProgress,
        content: "Working on it".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 14, 15, 0).unwrap(),
    };
    let notes = vec![(Status::InProgress, vec![note])];
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &notes, &[], &[]);

    // Semantic label instead of status
    assert!(output.contains("Progress:"));
    // Timestamp on its own line
    assert!(output.contains("2024-01-10 14:15"));
    // Content indented
    assert!(output.contains("    Working on it"));
}

#[test]
fn test_format_issue_details_with_events() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let event = create_test_event("prj-1234", Action::Started);
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[event]);

    assert!(output.contains("Log:"));
    assert!(output.contains("started"));
}

#[test]
fn test_format_issue_details_omits_created_event() {
    // Created event is redundant with the Created: line, so should be omitted
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let created_event = create_test_event("prj-1234", Action::Created);
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[created_event]);

    // Log section should not appear when only event is Created
    assert!(!output.contains("Log:"));
    assert!(!output.contains("created"));
}

#[test]
fn test_format_issue_details_filters_created_from_multiple_events() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::InProgress);
    let created_event = create_test_event("prj-1234", Action::Created);
    let started_event = create_test_event("prj-1234", Action::Started);
    let output = format_issue_details(
        &issue,
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
        &[created_event, started_event],
    );

    // Log should appear with started but not created
    assert!(output.contains("Log:"));
    assert!(output.contains("started"));
    assert!(!output.contains("created"));
}

#[test]
fn test_format_issue_details_omits_noted_at_creation_time() {
    // Noted events at creation time are shown in Description section, not log
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let mut noted_event = create_test_event("prj-1234", Action::Noted);
    noted_event.new_value = Some("Description note".to_string());
    // Event timestamp matches issue creation time (both use same default in helpers)
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[noted_event]);

    // Log section should not appear when only event is Noted at creation time
    assert!(!output.contains("Log:"));
    assert!(!output.contains("noted"));
}

#[test]
fn test_format_issue_details_shows_noted_after_creation() {
    // Noted events added later should still appear in log
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::InProgress);
    let noted_event = Event {
        id: 1,
        issue_id: "prj-1234".to_string(),
        action: Action::Noted,
        old_value: None,
        new_value: Some("Progress note".to_string()),
        reason: None,
        // Different timestamp from issue creation
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 14, 0, 0).unwrap(),
    };
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &[], &[], &[noted_event]);

    // Log should show the noted event since it's after creation
    assert!(output.contains("Log:"));
    assert!(output.contains("noted"));
}

// format_event tests
#[test]
fn test_format_event_created() {
    let event = create_test_event("prj-1234", Action::Created);
    let line = format_event(&event);
    assert!(line.contains("created"));
    assert!(line.contains("2024-01-10 12:00"));
}

#[test]
fn test_format_event_edited_with_new_value() {
    let mut event = create_test_event("prj-1234", Action::Edited);
    event.new_value = Some("new title".to_string());
    let line = format_event(&event);
    assert!(line.contains("edited"));
    assert!(line.contains("-> new title"));
}

#[test]
fn test_format_event_labeled() {
    let mut event = create_test_event("prj-1234", Action::Labeled);
    event.new_value = Some("urgent".to_string());
    let line = format_event(&event);
    assert!(line.contains("labeled"));
    assert!(line.contains("urgent"));
}

#[test]
fn test_format_event_related() {
    let mut event = create_test_event("prj-1234", Action::Related);
    event.new_value = Some("blocks prj-5678".to_string());
    let line = format_event(&event);
    assert!(line.contains("related"));
    assert!(line.contains("blocks prj-5678"));
}

#[test]
fn test_format_event_linked() {
    let mut event = create_test_event("prj-1234", Action::Linked);
    event.new_value = Some("https://github.com/org/repo/issues/123".to_string());
    let line = format_event(&event);
    assert!(line.contains("linked"));
    assert!(line.contains("github.com"));
}

#[test]
fn test_format_event_done_with_reason() {
    let mut event = create_test_event("prj-1234", Action::Done);
    event.reason = Some("already fixed upstream".to_string());
    let line = format_event(&event);
    assert!(line.contains("done"));
    assert!(line.contains("\"already fixed upstream\""));
}

#[test]
fn test_format_event_closed_with_reason() {
    let mut event = create_test_event("prj-1234", Action::Closed);
    event.reason = Some("wontfix".to_string());
    let line = format_event(&event);
    assert!(line.contains("closed"));
    assert!(line.contains("\"wontfix\""));
}

#[test]
fn test_format_event_reopened_with_reason() {
    let mut event = create_test_event("prj-1234", Action::Reopened);
    event.reason = Some("regression found".to_string());
    let line = format_event(&event);
    assert!(line.contains("reopened"));
    assert!(line.contains("\"regression found\""));
}

#[test]
fn test_format_event_noted_short() {
    let mut event = create_test_event("prj-1234", Action::Noted);
    event.new_value = Some("Short note".to_string());
    let line = format_event(&event);
    assert!(line.contains("noted"));
    assert!(line.contains("\"Short note\""));
}

#[test]
fn test_format_event_noted_truncated() {
    let mut event = create_test_event("prj-1234", Action::Noted);
    event.new_value = Some(
        "This is a very long note that exceeds fifty characters and should be truncated"
            .to_string(),
    );
    let line = format_event(&event);
    assert!(line.contains("noted"));
    assert!(line.contains("..."));
    // Original text should be truncated
    assert!(!line.contains("truncated"));
}

#[test]
fn test_format_event_simple_actions() {
    // These actions don't have extra formatting
    for action in [
        Action::Started,
        Action::Stopped,
        Action::Done,
        Action::Unblocked,
    ] {
        let event = create_test_event("prj-1234", action);
        let line = format_event(&event);
        assert!(line.contains(action.as_str()));
    }
}

// format_event_with_id tests
#[test]
fn test_format_event_with_id_basic() {
    let event = create_test_event("prj-1234", Action::Started);
    let line = format_event_with_id(&event);
    assert!(line.contains("prj-1234"));
    assert!(line.contains("started"));
}

#[test]
fn test_format_event_with_id_closed() {
    let mut event = create_test_event("prj-1234", Action::Closed);
    event.reason = Some("duplicate".to_string());
    let line = format_event_with_id(&event);
    assert!(line.contains("prj-1234"));
    assert!(line.contains("closed"));
    assert!(line.contains("\"duplicate\""));
}

#[test]
fn test_format_event_with_id_labeled() {
    let mut event = create_test_event("prj-1234", Action::Labeled);
    event.new_value = Some("v1.0".to_string());
    let line = format_event_with_id(&event);
    assert!(line.contains("prj-1234"));
    assert!(line.contains("labeled"));
    assert!(line.contains("v1.0"));
}

// format_tree_root tests
#[test]
fn test_format_tree_root_todo() {
    let issue = create_test_issue("prj-1234", "Root issue", IssueType::Feature, Status::Todo);
    let output = format_tree_root(&issue, None);
    assert!(output.contains("prj-1234"));
    assert!(output.contains("Root issue"));
    // Todo status should not be shown explicitly
    assert!(!output.contains("[todo]"));
}

#[test]
fn test_format_tree_root_in_progress() {
    let issue = create_test_issue(
        "prj-1234",
        "Root issue",
        IssueType::Feature,
        Status::InProgress,
    );
    let output = format_tree_root(&issue, None);
    assert!(output.contains("[in_progress]"));
}

#[test]
fn test_format_tree_root_with_blockers() {
    let issue = create_test_issue("prj-1234", "Root issue", IssueType::Task, Status::Todo);
    let blockers = vec!["prj-aaaa".to_string(), "prj-bbbb".to_string()];
    let output = format_tree_root(&issue, Some(&blockers));
    assert!(output.contains("blocked by prj-aaaa, prj-bbbb"));
}

#[test]
fn test_format_tree_root_empty_blockers() {
    let issue = create_test_issue("prj-1234", "Root issue", IssueType::Task, Status::Todo);
    let blockers: Vec<String> = vec![];
    let output = format_tree_root(&issue, Some(&blockers));
    assert!(!output.contains("blocked by"));
}

// format_tree_child tests
#[test]
fn test_format_tree_child_not_last() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Todo);
    let lines = format_tree_child(&issue, "", false, None);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("├── "));
    assert!(lines[0].contains("prj-1234"));
}

#[test]
fn test_format_tree_child_last() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Todo);
    let lines = format_tree_child(&issue, "", true, None);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("└── "));
}

#[test]
fn test_format_tree_child_with_prefix() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Todo);
    let lines = format_tree_child(&issue, "│   ", false, None);
    assert!(lines[0].starts_with("│   ├── "));
}

#[test]
fn test_format_tree_child_with_status() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Done);
    let lines = format_tree_child(&issue, "", false, None);
    assert!(lines[0].contains("[done]"));
}

#[test]
fn test_format_tree_child_with_blockers_not_last() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Todo);
    let blockers = vec!["prj-aaaa".to_string()];
    let lines = format_tree_child(&issue, "", false, Some(&blockers));
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("blocked by prj-aaaa"));
    assert!(lines[1].starts_with("│   └── "));
}

#[test]
fn test_format_tree_child_with_blockers_last() {
    let issue = create_test_issue("prj-1234", "Child issue", IssueType::Task, Status::Todo);
    let blockers = vec!["prj-aaaa".to_string()];
    let lines = format_tree_child(&issue, "", true, Some(&blockers));
    assert_eq!(lines.len(), 2);
    assert!(lines[1].contains("blocked by prj-aaaa"));
    assert!(lines[1].starts_with("    └── "));
}

// note_section_label tests
#[test]
fn test_note_section_label_todo() {
    assert_eq!(note_section_label(Status::Todo), "Description");
}

#[test]
fn test_note_section_label_in_progress() {
    assert_eq!(note_section_label(Status::InProgress), "Progress");
}

#[test]
fn test_note_section_label_done() {
    assert_eq!(note_section_label(Status::Done), "Summary");
}

#[test]
fn test_note_section_label_closed() {
    // Closed issues have "Close Reason" notes
    assert_eq!(note_section_label(Status::Closed), "Close Reason");
}

// wrap_text tests
#[test]
fn test_wrap_text_short() {
    let text = "Short text";
    assert_eq!(wrap_text(text, 96), "Short text");
}

#[test]
fn test_wrap_text_exact_width() {
    let text = "a".repeat(96);
    assert_eq!(wrap_text(&text, 96), text);
}

#[test]
fn test_wrap_text_long_wraps() {
    let text = "This is a very long line that needs to be wrapped at word boundaries because it exceeds the maximum width";
    let wrapped = wrap_text(text, 50);
    // Should wrap at word boundaries
    for line in wrapped.lines() {
        assert!(line.len() <= 50, "Line too long: {}", line);
    }
    // Should preserve all words
    assert!(wrapped.contains("wrapped"));
    assert!(wrapped.contains("boundaries"));
}

#[test]
fn test_wrap_text_with_newlines_preserved() {
    let text = "Line 1\nLine 2\nLine 3";
    assert_eq!(wrap_text(text, 96), text);
}

#[test]
fn test_wrap_text_multiline_not_rewrapped() {
    // Even if individual lines are long, if there are newlines, preserve exactly
    let text = "This is a really long first line that would normally be wrapped\nShort second line";
    assert_eq!(wrap_text(text, 50), text);
}

// format_note tests
#[test]
fn test_format_note_basic() {
    let note = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::Todo,
        content: "This is a note".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 10, 30, 0).unwrap(),
    };
    let lines = format_note(&note);

    assert_eq!(lines.len(), 2);
    // First line: timestamp with 2-space indent
    assert_eq!(lines[0], "  2024-01-10 10:30");
    // Second line: content with 4-space indent
    assert_eq!(lines[1], "    This is a note");
}

#[test]
fn test_format_note_multiline() {
    let note = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::InProgress,
        content: "Line 1\nLine 2\nLine 3".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 14, 15, 0).unwrap(),
    };
    let lines = format_note(&note);

    assert_eq!(lines.len(), 4); // timestamp + 3 content lines
    assert_eq!(lines[0], "  2024-01-10 14:15");
    assert_eq!(lines[1], "    Line 1");
    assert_eq!(lines[2], "    Line 2");
    assert_eq!(lines[3], "    Line 3");
}

#[test]
fn test_format_note_long_wraps() {
    let long_content = "This is a very long note that exceeds ninety-six characters and should be automatically wrapped at word boundaries to maintain readability";
    let note = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::Todo,
        content: long_content.to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 10, 30, 0).unwrap(),
    };
    let lines = format_note(&note);

    // Should have timestamp + at least 2 content lines (wrapped)
    assert!(
        lines.len() >= 3,
        "Expected wrapped output, got {} lines",
        lines.len()
    );
    // All content lines should be indented with 4 spaces
    for line in &lines[1..] {
        assert!(
            line.starts_with("    "),
            "Expected 4-space indent: '{}'",
            line
        );
    }
}

// format_issue_details with multiple notes
#[test]
fn test_format_issue_details_multiple_notes_same_status() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Todo);
    let note1 = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::Todo,
        content: "First note".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 10, 0, 0).unwrap(),
    };
    let note2 = Note {
        id: 2,
        issue_id: "prj-1234".to_string(),
        status: Status::Todo,
        content: "Second note".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 11, 0, 0).unwrap(),
    };
    let notes = vec![(Status::Todo, vec![note1, note2])];
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &notes, &[], &[]);

    // Should have Description: label
    assert!(output.contains("Description:"));
    // Both notes should appear
    assert!(output.contains("First note"));
    assert!(output.contains("Second note"));
}

#[test]
fn test_format_issue_details_notes_different_statuses() {
    let issue = create_test_issue("prj-1234", "Test", IssueType::Task, Status::Done);
    let desc_note = Note {
        id: 1,
        issue_id: "prj-1234".to_string(),
        status: Status::Todo,
        content: "Initial requirements".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 9, 0, 0).unwrap(),
    };
    let progress_note = Note {
        id: 2,
        issue_id: "prj-1234".to_string(),
        status: Status::InProgress,
        content: "Working on implementation".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 14, 0, 0).unwrap(),
    };
    let summary_note = Note {
        id: 3,
        issue_id: "prj-1234".to_string(),
        status: Status::Done,
        content: "Completed successfully".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 10, 17, 0, 0).unwrap(),
    };
    let notes = vec![
        (Status::Todo, vec![desc_note]),
        (Status::InProgress, vec![progress_note]),
        (Status::Done, vec![summary_note]),
    ];
    let output = format_issue_details(&issue, &[], &[], &[], &[], &[], &notes, &[], &[]);

    // All three semantic labels
    assert!(output.contains("Description:"));
    assert!(output.contains("Progress:"));
    assert!(output.contains("Summary:"));
    // All content
    assert!(output.contains("Initial requirements"));
    assert!(output.contains("Working on implementation"));
    assert!(output.contains("Completed successfully"));
}
