// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use tempfile::tempdir;
use wk_core::{Database, Hlc, Issue, IssueType, Oplog, Status};

fn make_test_issue(id: &str, title: &str) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type: IssueType::Task,
        title: title.to_string(),
        status: Status::Todo,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
    }
}

fn make_test_issue_with_hlc(id: &str, title: &str, title_hlc: Hlc) -> Issue {
    Issue {
        id: id.to_string(),
        issue_type: IssueType::Task,
        title: title.to_string(),
        status: Status::Todo,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_status_hlc: None,
        last_title_hlc: Some(title_hlc),
        last_type_hlc: None,
    }
}

#[test]
fn snapshot_issue_to_ops_basic() {
    let issue = make_test_issue("test-123", "Test issue");
    let ops = snapshot_issue_to_ops(&issue, 0);

    // Should generate at least a CreateIssue op
    assert!(!ops.is_empty());

    // First op should be CreateIssue with synthetic HLC
    let first_op = &ops[0];
    assert_eq!(first_op.id.wall_ms, 0);
    match &first_op.payload {
        OpPayload::CreateIssue { id, title, .. } => {
            assert_eq!(id, "test-123");
            assert_eq!(title, "Test issue");
        }
        _ => panic!("Expected CreateIssue op"),
    }
}

#[test]
fn snapshot_issue_to_ops_with_title_hlc() {
    let title_hlc = Hlc::new(1000, 0, 1);
    let issue = make_test_issue_with_hlc("test-456", "Updated title", title_hlc);
    let ops = snapshot_issue_to_ops(&issue, 0);

    // Should have CreateIssue and SetTitle
    assert!(ops.len() >= 2);

    // Find SetTitle op
    let set_title_op = ops
        .iter()
        .find(|op| matches!(&op.payload, OpPayload::SetTitle { .. }));
    assert!(set_title_op.is_some());

    let set_title_op = set_title_op.unwrap();
    assert_eq!(set_title_op.id, title_hlc);
}

#[test]
fn snapshot_issue_to_ops_unique_indices() {
    let issue1 = make_test_issue("test-1", "Issue 1");
    let issue2 = make_test_issue("test-2", "Issue 2");

    let ops1 = snapshot_issue_to_ops(&issue1, 0);
    let ops2 = snapshot_issue_to_ops(&issue2, 1);

    // CreateIssue ops should have different counter values (based on index)
    let create1 = &ops1[0];
    let create2 = &ops2[0];

    assert_eq!(create1.id.counter, 0);
    assert_eq!(create2.id.counter, 1);
}

#[test]
fn apply_snapshot_to_cache_creates_issues() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("issues.db");
    let oplog_path = dir.path().join("oplog.jsonl");

    // Create empty oplog and db
    let _ = Oplog::open(&oplog_path).unwrap();
    let _ = Database::open(&db_path).unwrap();

    // Create test issues
    let issues = vec![
        make_test_issue("snap-1", "Snapshot issue 1"),
        make_test_issue("snap-2", "Snapshot issue 2"),
    ];
    let tags = vec![];
    let since = Hlc::new(2000, 0, 1);

    // Apply snapshot
    let result = apply_snapshot_to_cache(&issues, &tags, since, &db_path, &oplog_path, dir.path());
    assert!(result.is_ok());

    // Verify issues are in database
    let db = Database::open(&db_path).unwrap();
    let all_issues = db.get_all_issues().unwrap();
    assert_eq!(all_issues.len(), 2);

    let ids: Vec<_> = all_issues.iter().map(|i| i.id.as_str()).collect();
    assert!(ids.contains(&"snap-1"));
    assert!(ids.contains(&"snap-2"));
}

#[test]
fn apply_snapshot_to_cache_with_tags() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("issues.db");
    let oplog_path = dir.path().join("oplog.jsonl");

    // Create empty oplog and db
    let _ = Oplog::open(&oplog_path).unwrap();
    let _ = Database::open(&db_path).unwrap();

    // Create test issue with tag
    let issues = vec![make_test_issue("tagged-1", "Tagged issue")];
    let tags = vec![("tagged-1".to_string(), "priority:1".to_string())];
    let since = Hlc::new(3000, 0, 1);

    // Apply snapshot
    let result = apply_snapshot_to_cache(&issues, &tags, since, &db_path, &oplog_path, dir.path());
    assert!(result.is_ok());

    // Verify issue has the tag
    let db = Database::open(&db_path).unwrap();
    let issue = db.get_issue("tagged-1").unwrap();
    let labels = db.get_labels(&issue.id).unwrap();
    assert!(labels.contains(&"priority:1".to_string()));
}

#[test]
fn apply_snapshot_to_cache_deduplicates() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("issues.db");
    let oplog_path = dir.path().join("oplog.jsonl");

    // Create issue first
    let mut oplog = Oplog::open(&oplog_path).unwrap();
    let mut db = Database::open(&db_path).unwrap();

    let create_op = Op::new(
        Hlc::new(1000, 0, 1),
        OpPayload::CreateIssue {
            id: "existing-1".to_string(),
            issue_type: IssueType::Task,
            title: "Original title".to_string(),
        },
    );
    oplog.append(&create_op).unwrap();
    db.apply(&create_op).unwrap();
    drop(oplog);
    drop(db);

    // Now apply snapshot with same issue (should not create duplicate)
    let issues = vec![make_test_issue("existing-1", "Snapshot title")];
    let tags = vec![];
    let since = Hlc::new(2000, 0, 1);

    let result = apply_snapshot_to_cache(&issues, &tags, since, &db_path, &oplog_path, dir.path());
    assert!(result.is_ok());

    // Should still have only 1 issue
    let db = Database::open(&db_path).unwrap();
    let all_issues = db.get_all_issues().unwrap();
    assert_eq!(all_issues.len(), 1);
}

#[test]
fn apply_snapshot_updates_hlc_files() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("issues.db");
    let oplog_path = dir.path().join("oplog.jsonl");

    // Create empty oplog and db
    let _ = Oplog::open(&oplog_path).unwrap();
    let _ = Database::open(&db_path).unwrap();

    let issues = vec![make_test_issue("hlc-test", "HLC test issue")];
    let tags = vec![];
    let since = Hlc::new(5000, 5, 10);

    // Apply snapshot
    let result = apply_snapshot_to_cache(&issues, &tags, since, &db_path, &oplog_path, dir.path());
    assert!(result.is_ok());

    // Verify HLC files were updated
    let server_hlc = crate::commands::read_server_hlc(dir.path());
    assert!(server_hlc.is_some());
    assert_eq!(server_hlc.unwrap(), since);
}
