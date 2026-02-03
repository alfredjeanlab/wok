// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

//! Tests for the public `run()` function.
//!
//! The run() function routes Command variants to their respective implementations.
//! Since most commands require filesystem access (via open_db()), they are better
//! tested via integration tests that run the binary.
//!
//! This file contains tests for command variants that can be tested without
//! filesystem dependencies, validating the routing logic works correctly.

use crate::{AssigneeArgs, Command, LimitArgs, OutputFormat, TypeLabelArgs};

// Note: Most Command variants require open_db() which needs filesystem access.
// Those are tested via integration tests in tests/integration.rs.
// Here we test that the Command enum is correctly exported and can be constructed.

#[test]
fn test_command_init_construction() {
    let cmd = Command::Init {
        prefix: Some("test".to_string()),
        path: None,
        private: false,
    };
    // Verify pattern matching works
    if let Command::Init {
        prefix,
        path,
        private,
    } = cmd
    {
        assert_eq!(prefix, Some("test".to_string()));
        assert!(path.is_none());
        assert!(!private);
    } else {
        panic!("Expected Init command");
    }
}

#[test]
fn test_command_init_private_construction() {
    let cmd = Command::Init {
        prefix: Some("test".to_string()),
        path: None,
        private: true,
    };
    if let Command::Init {
        prefix,
        path,
        private,
    } = cmd
    {
        assert_eq!(prefix, Some("test".to_string()));
        assert!(path.is_none());
        assert!(private);
    } else {
        panic!("Expected Init command");
    }
}

#[test]
fn test_command_new_construction() {
    let cmd = Command::New {
        type_or_title: "task".to_string(),
        title: Some("My task".to_string()),
        label: vec!["urgent".to_string()],
        note: Some("A note".to_string()),
        link: vec!["https://github.com/org/repo/issues/123".to_string()],
        assignee: Some("alice".to_string()),
        priority: Some(2),
        description: None,
        blocks: vec![],
        blocked_by: vec![],
        tracks: vec![],
        tracked_by: vec![],
        output: OutputFormat::Text,
        prefix: None,
    };
    if let Command::New {
        type_or_title,
        title,
        label,
        note,
        link,
        assignee,
        priority,
        description,
        blocks,
        blocked_by,
        tracks,
        tracked_by,
        ..
    } = cmd
    {
        assert_eq!(type_or_title, "task");
        assert_eq!(title, Some("My task".to_string()));
        assert_eq!(label, vec!["urgent".to_string()]);
        assert_eq!(note, Some("A note".to_string()));
        assert_eq!(
            link,
            vec!["https://github.com/org/repo/issues/123".to_string()]
        );
        assert_eq!(assignee, Some("alice".to_string()));
        assert_eq!(priority, Some(2));
        assert!(description.is_none());
        assert!(blocks.is_empty());
        assert!(blocked_by.is_empty());
        assert!(tracks.is_empty());
        assert!(tracked_by.is_empty());
    } else {
        panic!("Expected New command");
    }
}

#[test]
fn test_command_new_with_dependencies_construction() {
    let cmd = Command::New {
        type_or_title: "bug".to_string(),
        title: Some("Fix crash".to_string()),
        label: vec![],
        note: None,
        link: vec![],
        assignee: None,
        priority: None,
        description: None,
        blocks: vec!["task-1".to_string()],
        blocked_by: vec!["task-2".to_string()],
        tracks: vec![],
        tracked_by: vec!["feature-1".to_string()],
        output: OutputFormat::Text,
        prefix: None,
    };
    if let Command::New {
        blocks,
        blocked_by,
        tracks,
        tracked_by,
        ..
    } = cmd
    {
        assert_eq!(blocks, vec!["task-1".to_string()]);
        assert_eq!(blocked_by, vec!["task-2".to_string()]);
        assert!(tracks.is_empty());
        assert_eq!(tracked_by, vec!["feature-1".to_string()]);
    } else {
        panic!("Expected New command");
    }
}

#[test]
fn test_command_lifecycle_construction() {
    // Test Start (single ID)
    let cmd = Command::Start {
        ids: vec!["test-1".to_string()],
    };
    assert!(matches!(cmd, Command::Start { ids } if ids == vec!["test-1"]));

    // Test Start (multiple IDs)
    let cmd = Command::Start {
        ids: vec!["test-1".to_string(), "test-2".to_string()],
    };
    assert!(matches!(cmd, Command::Start { ids } if ids == vec!["test-1", "test-2"]));

    // Test Done
    let cmd = Command::Done {
        ids: vec!["test-1".to_string()],
        reason: Some("completed".to_string()),
    };
    assert!(
        matches!(cmd, Command::Done { ids, reason } if ids == vec!["test-1"] && reason == Some("completed".to_string()))
    );

    // Test Close
    let cmd = Command::Close {
        ids: vec!["test-1".to_string()],
        reason: Some("wont fix".to_string()),
    };
    assert!(
        matches!(cmd, Command::Close { ids, reason } if ids == vec!["test-1"] && reason == Some("wont fix".to_string()))
    );

    // Test Close without reason (for human interactive mode)
    let cmd = Command::Close {
        ids: vec!["test-1".to_string()],
        reason: None,
    };
    assert!(
        matches!(cmd, Command::Close { ids, reason } if ids == vec!["test-1"] && reason.is_none())
    );

    // Test Reopen
    let cmd = Command::Reopen {
        ids: vec!["test-1".to_string()],
        reason: Some("need more work".to_string()),
    };
    assert!(
        matches!(cmd, Command::Reopen { ids, reason } if ids == vec!["test-1"] && reason == Some("need more work".to_string()))
    );

    // Test Reopen without reason (for human interactive mode)
    let cmd = Command::Reopen {
        ids: vec!["test-1".to_string()],
        reason: None,
    };
    assert!(
        matches!(cmd, Command::Reopen { ids, reason } if ids == vec!["test-1"] && reason.is_none())
    );
}

#[test]
fn test_command_list_construction() {
    let cmd = Command::List {
        status: vec!["todo".to_string(), "in_progress".to_string()],
        type_label: TypeLabelArgs {
            r#type: vec!["task".to_string()],
            label: vec!["urgent".to_string()],
        },
        assignee_args: AssigneeArgs {
            assignee: vec![],
            unassigned: false,
        },
        filter: vec![],
        limits: LimitArgs {
            limit: None,
            no_limit: false,
        },
        blocked: false,
        all: false,
        output: OutputFormat::Text,
    };
    if let Command::List {
        status,
        type_label,
        blocked,
        ..
    } = cmd
    {
        assert_eq!(status.len(), 2);
        assert_eq!(type_label.r#type, vec!["task".to_string()]);
        assert_eq!(type_label.label, vec!["urgent".to_string()]);
        assert!(!blocked);
    } else {
        panic!("Expected List command");
    }
}

#[test]
fn test_command_show_construction() {
    let cmd = Command::Show {
        ids: vec!["test-1".to_string()],
        output: "json".to_string(),
    };
    assert!(
        matches!(cmd, Command::Show { ids, output } if ids == vec!["test-1"] && output == "json")
    );
}

#[test]
fn test_command_dep_construction() {
    let cmd = Command::Dep {
        from_id: "feature-1".to_string(),
        rel: "blocks".to_string(),
        to_ids: vec!["task-1".to_string(), "task-2".to_string()],
    };
    if let Command::Dep {
        from_id,
        rel,
        to_ids,
    } = cmd
    {
        assert_eq!(from_id, "feature-1");
        assert_eq!(rel, "blocks");
        assert_eq!(to_ids.len(), 2);
    } else {
        panic!("Expected Dep command");
    }
}

#[test]
fn test_command_label_construction() {
    // Single ID
    let cmd = Command::Label {
        args: vec!["test-1".to_string(), "urgent".to_string()],
    };
    assert!(matches!(cmd, Command::Label { args } if args == vec!["test-1", "urgent"]));

    // Multiple IDs
    let cmd = Command::Label {
        args: vec![
            "test-1".to_string(),
            "test-2".to_string(),
            "urgent".to_string(),
        ],
    };
    assert!(matches!(cmd, Command::Label { args } if args == vec!["test-1", "test-2", "urgent"]));

    let cmd = Command::Unlabel {
        args: vec!["test-1".to_string(), "urgent".to_string()],
    };
    assert!(matches!(cmd, Command::Unlabel { args } if args == vec!["test-1", "urgent"]));
}

#[test]
fn test_command_note_construction() {
    let cmd = Command::Note {
        id: "test-1".to_string(),
        content: "My note".to_string(),
        replace: true,
    };
    assert!(
        matches!(cmd, Command::Note { id, content, replace } if id == "test-1" && content == "My note" && replace)
    );
}

#[test]
fn test_command_log_construction() {
    let cmd = Command::Log {
        id: Some("test-1".to_string()),
        limits: LimitArgs {
            limit: Some(50),
            no_limit: false,
        },
    };
    assert!(
        matches!(cmd, Command::Log { id, limits } if id == Some("test-1".to_string()) && limits.limit == Some(50) && !limits.no_limit)
    );

    let cmd = Command::Log {
        id: None,
        limits: LimitArgs {
            limit: None,
            no_limit: true,
        },
    };
    assert!(
        matches!(cmd, Command::Log { id, limits } if id.is_none() && limits.limit.is_none() && limits.no_limit)
    );
}

#[test]
fn test_command_export_construction() {
    let cmd = Command::Export {
        filepath: "/tmp/export.jsonl".to_string(),
    };
    assert!(matches!(cmd, Command::Export { filepath } if filepath == "/tmp/export.jsonl"));
}

#[test]
fn test_command_ready_construction() {
    let cmd = Command::Ready {
        type_label: TypeLabelArgs {
            r#type: vec!["bug".to_string()],
            label: vec!["backend".to_string()],
        },
        assignee: vec![],
        unassigned: false,
        all_assignees: false,
        output: OutputFormat::Text,
    };
    assert!(matches!(cmd, Command::Ready { type_label, output, .. }
        if type_label.r#type == vec!["bug".to_string()] && type_label.label == vec!["backend".to_string()] && matches!(output, OutputFormat::Text)
    ));
}

#[test]
fn test_command_tree_construction() {
    let cmd = Command::Tree {
        id: "feature-1".to_string(),
    };
    assert!(matches!(cmd, Command::Tree { id } if id == "feature-1"));
}

#[test]
fn test_command_edit_construction() {
    let cmd = Command::Edit {
        id: "test-1".to_string(),
        attr: Some("title".to_string()),
        value: Some("New title".to_string()),
        flag_title: None,
        flag_description: None,
        flag_type: None,
        flag_assignee: None,
    };
    if let Command::Edit {
        id, attr, value, ..
    } = cmd
    {
        assert_eq!(id, "test-1");
        assert_eq!(attr, Some("title".to_string()));
        assert_eq!(value, Some("New title".to_string()));
    } else {
        panic!("Expected Edit command");
    }
}

#[test]
fn test_command_undep_construction() {
    let cmd = Command::Undep {
        from_id: "feature-1".to_string(),
        rel: "blocks".to_string(),
        to_ids: vec!["task-1".to_string()],
    };
    if let Command::Undep {
        from_id,
        rel,
        to_ids,
    } = cmd
    {
        assert_eq!(from_id, "feature-1");
        assert_eq!(rel, "blocks");
        assert_eq!(to_ids, vec!["task-1".to_string()]);
    } else {
        panic!("Expected Undep command");
    }
}
