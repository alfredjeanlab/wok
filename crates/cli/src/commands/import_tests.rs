// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::db::Database;
use tempfile::TempDir;

fn setup_test_db() -> (Database, TempDir) {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).unwrap();
    (db, dir)
}

fn dummy_config() -> Config {
    Config::new("test".to_string()).unwrap()
}

#[test]
fn test_detect_format_explicit() {
    assert_eq!(detect_format("foo.jsonl", "bd"), "bd");
    assert_eq!(detect_format("foo.jsonl", "wok"), "wok");
}

#[test]
fn test_detect_format_auto() {
    assert_eq!(detect_format(".beads/issues.jsonl", "wok"), "bd");
    assert_eq!(detect_format("path/to/.beads/issues.jsonl", "wok"), "bd");
    assert_eq!(detect_format("beads.jsonl", "wok"), "wok");
}

#[test]
fn test_convert_beads_status_basic() {
    assert_eq!(convert_beads_status("open", &None, &None), Status::Todo);
    assert_eq!(
        convert_beads_status("in_progress", &None, &None),
        Status::InProgress
    );
    assert_eq!(convert_beads_status("closed", &None, &None), Status::Done);
    assert_eq!(convert_beads_status("blocked", &None, &None), Status::Todo);
    assert_eq!(convert_beads_status("deferred", &None, &None), Status::Todo);
    assert_eq!(convert_beads_status("unknown", &None, &None), Status::Todo);
}

#[test]
fn test_convert_beads_type() {
    assert_eq!(convert_beads_type("bug"), IssueType::Bug);
    assert_eq!(convert_beads_type("feature"), IssueType::Feature);
    assert_eq!(convert_beads_type("task"), IssueType::Task);
    assert_eq!(convert_beads_type("epic"), IssueType::Epic);
    assert_eq!(convert_beads_type("chore"), IssueType::Chore);
}

#[test]
fn test_parse_wk_format() {
    let json = r#"{"id":"test-1","issue_type":"task","title":"Test task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent"],"notes":[],"deps":[],"events":[]}"#;
    let wk: WkIssue = serde_json::from_str(json).unwrap();
    assert_eq!(wk.issue.id, "test-1");
    assert_eq!(wk.issue.title, "Test task");
    assert_eq!(wk.labels, vec!["urgent"]);
}

#[test]
fn test_parse_beads_format() {
    let json = r#"{"id":"bd-1","title":"Beads task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#;
    let bd: BeadsIssue = serde_json::from_str(json).unwrap();
    assert_eq!(bd.id, "bd-1");
    assert_eq!(bd.title, "Beads task");
    assert_eq!(bd.status, "open");
}

#[test]
fn test_convert_beads_issue() {
    let bd = BeadsIssue {
        id: "bd-1".to_string(),
        title: "Test".to_string(),
        description: None,
        status: "open".to_string(),
        priority: 2,
        issue_type: "bug".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        labels: vec!["urgent".to_string()],
        dependencies: vec![],
        comments: vec![BeadsComment {
            text: "A comment".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }],
        close_reason: None,
        delete_reason: None,
    };

    let (issue, labels, notes, _deps, _close_data, _links) = convert_beads_issue(bd).unwrap();
    assert_eq!(issue.id, "bd-1");
    assert_eq!(issue.issue_type, IssueType::Bug);
    assert_eq!(issue.status, Status::Todo);
    // Labels include both labels and priority
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"priority:2".to_string()));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].1, "A comment");
}

#[test]
fn test_convert_beads_issue_tombstone() {
    let bd = BeadsIssue {
        id: "bd-tomb".to_string(),
        title: "Deleted issue".to_string(),
        description: None,
        status: "tombstone".to_string(),
        priority: 2,
        issue_type: "task".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
        labels: vec![],
        dependencies: vec![],
        comments: vec![],
        close_reason: None,
        delete_reason: Some("batch delete".to_string()),
    };

    let (issue, _labels, notes, _deps, close_data, _links) = convert_beads_issue(bd).unwrap();
    assert_eq!(issue.id, "bd-tomb");
    assert_eq!(issue.status, Status::Closed);

    // Should have close data
    let close_data = close_data.expect("tombstone should have close_data");
    assert_eq!(close_data.reason, "batch delete");
    assert!(close_data.is_failure);

    // Should have a note with the delete reason
    assert!(notes.iter().any(|n| n.1 == "batch delete"));
}

#[test]
fn test_import_creates_issue() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    // Create temp file with import data
    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-imp1","issue_type":"task","title":"Imported","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("test-imp1").unwrap();
    assert_eq!(issue.title, "Imported");
}

#[test]
fn test_import_with_labels() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-labels","issue_type":"task","title":"Labeled","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent","project:auth"],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let labels = db.get_labels("test-labels").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"project:auth".to_string()));
}

#[test]
fn test_import_dry_run() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-dry","issue_type":"task","title":"Dry run","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        true, // dry_run
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    // Issue should NOT exist
    assert!(db.get_issue("test-dry").is_err());
}

#[test]
fn test_import_status_filter() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-todo","issue_type":"task","title":"Todo","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-done","issue_type":"task","title":"Done","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec!["todo".to_string()],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    // Only todo should be imported
    assert!(db.get_issue("test-todo").is_ok());
    assert!(db.get_issue("test-done").is_err());
}

#[test]
fn test_import_prefix_filter() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"myproj-1","issue_type":"task","title":"My project","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"other-1","issue_type":"task","title":"Other project","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        Some("myproj".to_string()),
    )
    .unwrap();

    // Only myproj should be imported
    assert!(db.get_issue("myproj-1").is_ok());
    assert!(db.get_issue("other-1").is_err());
}

#[test]
fn test_import_updates_existing() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    // Create initial issue
    let issue = Issue {
        id: "test-upd".to_string(),
        issue_type: IssueType::Task,
        title: "Original".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&issue).unwrap();

    // Import with updated title
    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-upd","issue_type":"task","title":"Updated","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let updated = db.get_issue("test-upd").unwrap();
    assert_eq!(updated.title, "Updated");
}

#[test]
fn test_import_updates_existing_status() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    // Create initial issue with todo status
    let issue = Issue {
        id: "test-status".to_string(),
        issue_type: IssueType::Task,
        title: "Status test".to_string(),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        closed_at: None,
        last_status_hlc: None,
        last_title_hlc: None,
        last_type_hlc: None,
        last_description_hlc: None,
        last_assignee_hlc: None,
    };
    db.create_issue(&issue).unwrap();

    // Import with done status
    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-status","issue_type":"task","title":"Status test","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let updated = db.get_issue("test-status").unwrap();
    assert_eq!(updated.status, Status::Done);
}

#[test]
fn test_import_beads_format() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-test","title":"Beads issue","status":"open","priority":2,"issue_type":"bug","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent"]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-test").unwrap();
    assert_eq!(issue.title, "Beads issue");
    assert_eq!(issue.issue_type, IssueType::Bug);
    assert_eq!(issue.status, Status::Todo); // "open" -> todo

    let labels = db.get_labels("bd-test").unwrap();
    assert!(labels.contains(&"urgent".to_string()));
}

#[test]
fn test_import_empty_file() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(&import_file, "").unwrap();

    let result = run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    );

    assert!(result.is_ok());
}

#[test]
fn test_import_skips_empty_lines() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-1","issue_type":"task","title":"First","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}

{"id":"test-2","issue_type":"task","title":"Second","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    assert!(db.get_issue("test-1").is_ok());
    assert!(db.get_issue("test-2").is_ok());
}

#[test]
fn test_import_invalid_json_fails() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(&import_file, "not valid json").unwrap();

    let result = run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    );

    assert!(result.is_err());
}

#[test]
fn test_import_chore_type() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"test-chore","issue_type":"chore","title":"Update dependencies","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "wok",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("test-chore").unwrap();
    assert_eq!(issue.title, "Update dependencies");
    assert_eq!(issue.issue_type, IssueType::Chore);
}

#[test]
fn test_import_beads_chore_type() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-chore","title":"Cleanup code","status":"open","priority":3,"issue_type":"chore","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-chore").unwrap();
    assert_eq!(issue.title, "Cleanup code");
    assert_eq!(issue.issue_type, IssueType::Chore);
}

#[test]
fn test_import_beads_epic_preserved() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-epic","title":"New feature","status":"open","priority":1,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-epic").unwrap();
    assert_eq!(issue.title, "New feature");
    assert_eq!(issue.issue_type, IssueType::Epic);
}

// === Phase 4-8 Unit Tests ===

#[test]
fn test_convert_beads_dep_type_blocks() {
    assert_eq!(convert_beads_dep_type("blocks"), Relation::Blocks);
}

#[test]
fn test_convert_beads_dep_type_parent() {
    assert_eq!(convert_beads_dep_type("parent"), Relation::Tracks);
}

#[test]
fn test_convert_beads_dep_type_parent_child() {
    assert_eq!(convert_beads_dep_type("parent-child"), Relation::TrackedBy);
}

#[test]
fn test_convert_beads_dep_type_child_of() {
    assert_eq!(convert_beads_dep_type("child-of"), Relation::TrackedBy);
}

#[test]
fn test_convert_beads_dep_type_tracks() {
    assert_eq!(convert_beads_dep_type("tracks"), Relation::Tracks);
}

#[test]
fn test_convert_beads_dep_type_unknown() {
    assert_eq!(convert_beads_dep_type("unknown"), Relation::Blocks);
}

#[test]
fn test_is_failure_reason_true() {
    assert!(is_failure_reason("abandoned"));
    assert!(is_failure_reason("FAILED"));
    assert!(is_failure_reason("rejected by team"));
    assert!(is_failure_reason("wontfix - not needed"));
    assert!(is_failure_reason("won't fix"));
    assert!(is_failure_reason("canceled by user"));
    assert!(is_failure_reason("Cancelled"));
    assert!(is_failure_reason("blocked by dependency"));
    assert!(is_failure_reason("error occurred"));
    assert!(is_failure_reason("timeout reached"));
    assert!(is_failure_reason("aborted by CI"));
}

#[test]
fn test_is_failure_reason_false() {
    assert!(!is_failure_reason("Completed"));
    assert!(!is_failure_reason("done"));
    assert!(!is_failure_reason("fixed in v1.2"));
    assert!(!is_failure_reason("merged to main"));
    assert!(!is_failure_reason(""));
}

#[test]
fn test_convert_beads_status_with_failure_reason() {
    let failure_reason = Some("abandoned".to_string());
    assert_eq!(
        convert_beads_status("closed", &failure_reason, &None),
        Status::Closed
    );
}

#[test]
fn test_convert_beads_status_with_success_reason() {
    let success_reason = Some("Completed successfully".to_string());
    assert_eq!(
        convert_beads_status("closed", &success_reason, &None),
        Status::Done
    );
}

#[test]
fn test_convert_beads_status_closed_no_reason() {
    assert_eq!(convert_beads_status("closed", &None, &None), Status::Done);
}

#[test]
fn test_convert_beads_status_tombstone_basic() {
    // Tombstone status should always map to Closed
    assert_eq!(
        convert_beads_status("tombstone", &None, &None),
        Status::Closed
    );
}

#[test]
fn test_convert_beads_status_tombstone_with_delete_reason() {
    let delete_reason = Some("batch delete".to_string());
    assert_eq!(
        convert_beads_status("tombstone", &None, &delete_reason),
        Status::Closed
    );
}

#[test]
fn test_convert_beads_status_tombstone_with_failure_delete_reason() {
    let delete_reason = Some("abandoned by team".to_string());
    assert_eq!(
        convert_beads_status("tombstone", &None, &delete_reason),
        Status::Closed
    );
}

#[test]
fn test_import_beads_priority_as_label() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-prio","title":"Priority issue","status":"open","priority":1,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let labels = db.get_labels("bd-prio").unwrap();
    assert!(labels.contains(&"priority:1".to_string()));
}

#[test]
fn test_import_beads_priority_zero_not_labeled() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-prio0","title":"No priority","status":"open","priority":0,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let labels = db.get_labels("bd-prio0").unwrap();
    assert!(!labels.iter().any(|l| l.starts_with("priority:")));
}

#[test]
fn test_beads_comment_text_field() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-text","title":"Text field test","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","comments":[{"text":"Comment using text field","created_at":"2024-01-01T00:00:00Z"}]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let notes = db.get_notes("bd-text").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "Comment using text field"));
}

#[test]
fn test_beads_comment_content_alias() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-content","title":"Content alias test","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","comments":[{"content":"Comment using content field","created_at":"2024-01-01T00:00:00Z"}]}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let notes = db.get_notes("bd-content").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "Comment using content field"));
}

#[test]
fn test_import_beads_close_reason_failure() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-fail","title":"Failed issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"abandoned due to scope change"}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-fail").unwrap();
    assert_eq!(issue.status, Status::Closed);

    let notes = db.get_notes("bd-fail").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "abandoned due to scope change" && n.status == Status::Closed));
}

#[test]
fn test_import_beads_close_reason_success() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-success","title":"Successful issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"Completed as planned"}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-success").unwrap();
    assert_eq!(issue.status, Status::Done);

    // Close reason notes always use Closed status so they display under "Close Reason:"
    let notes = db.get_notes("bd-success").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "Completed as planned" && n.status == Status::Closed));
}

#[test]
fn test_import_beads_tombstone_status() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-tomb","title":"Tombstoned issue","status":"tombstone","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","delete_reason":"batch delete"}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-tomb").unwrap();
    assert_eq!(issue.status, Status::Closed); // tombstone -> closed

    let notes = db.get_notes("bd-tomb").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "batch delete" && n.status == Status::Closed));
}

#[test]
fn test_import_beads_tombstone_without_delete_reason() {
    let (mut db, _dir) = setup_test_db();
    let config = dummy_config();

    let import_file = _dir.path().join("import.jsonl");
    std::fs::write(
        &import_file,
        r#"{"id":"bd-tomb-no-reason","title":"Tombstoned without reason","status":"tombstone","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}"#,
    )
    .unwrap();

    run_impl(
        &mut db,
        &config,
        import_file.to_str().unwrap(),
        "bd",
        false,
        vec![],
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let issue = db.get_issue("bd-tomb-no-reason").unwrap();
    assert_eq!(issue.status, Status::Closed); // tombstone -> closed

    // Should have a default "deleted" note
    let notes = db.get_notes("bd-tomb-no-reason").unwrap();
    assert!(notes
        .iter()
        .any(|n| n.content == "deleted" && n.status == Status::Closed));
}
