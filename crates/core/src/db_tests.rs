// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use crate::hlc::Hlc;
use crate::issue::Action;
use chrono::Utc;

fn test_issue(id: &str, title: &str) -> Issue {
    Issue::new(
        id.to_string(),
        IssueType::Task,
        title.to_string(),
        Utc::now(),
    )
}

#[test]
fn create_and_get_issue() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");

    db.create_issue(&issue).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();

    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.title, "Test issue");
    assert_eq!(retrieved.status, Status::Todo);
    assert!(retrieved.closed_at.is_none());
    assert!(retrieved.last_status_hlc.is_none());
}

#[test]
fn issue_exists() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");

    assert!(!db.issue_exists("test-1").unwrap());
    db.create_issue(&issue).unwrap();
    assert!(db.issue_exists("test-1").unwrap());
}

#[test]
fn update_issue_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_status("test-1", Status::InProgress)
        .unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::InProgress);
}

#[test]
fn update_issue_status_hlc() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let hlc = Hlc::new(1000, 0, 1);
    db.update_issue_status_hlc("test-1", hlc).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.last_status_hlc, Some(hlc));
}

#[test]
fn update_issue_status_sets_closed_at() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    // Transitioning to Done should set closed_at
    db.update_issue_status("test-1", Status::Done).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Done);
    assert!(retrieved.closed_at.is_some());

    // Reopening (back to Todo) should clear closed_at
    db.update_issue_status("test-1", Status::Todo).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Todo);
    assert!(retrieved.closed_at.is_none());

    // Transitioning to Closed should set closed_at
    db.update_issue_status("test-1", Status::Closed).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.status, Status::Closed);
    assert!(retrieved.closed_at.is_some());
}

#[test]
fn update_issue_title() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Old title");
    db.create_issue(&issue).unwrap();

    db.update_issue_title("test-1", "New title").unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.title, "New title");
}

#[test]
fn update_issue_type() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_type("test-1", IssueType::Bug).unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.issue_type, IssueType::Bug);
}

#[test]
fn list_issues_filter_status() {
    let db = Database::open_in_memory().unwrap();

    let mut issue1 = test_issue("test-1", "Issue 1");
    issue1.status = Status::Todo;
    let mut issue2 = test_issue("test-2", "Issue 2");
    issue2.status = Status::InProgress;

    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    let todos = db.list_issues(Some(Status::Todo), None, None).unwrap();
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, "test-1");

    let in_progress = db
        .list_issues(Some(Status::InProgress), None, None)
        .unwrap();
    assert_eq!(in_progress.len(), 1);
    assert_eq!(in_progress[0].id, "test-2");
}

#[test]
fn log_and_get_events() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let event = Event::new("test-1".to_string(), Action::Created);
    db.log_event(&event).unwrap();

    let events = db.get_events("test-1").unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, Action::Created);
}

#[test]
fn add_and_get_notes() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_note("test-1", Status::Todo, "First note").unwrap();
    db.add_note("test-1", Status::InProgress, "Second note")
        .unwrap();

    let notes = db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].content, "First note");
    assert_eq!(notes[1].content, "Second note");
}

#[test]
fn add_and_get_labels() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    db.add_label("test-1", "backend").unwrap();

    let labels = db.get_labels("test-1").unwrap();
    assert_eq!(labels.len(), 2);
    assert!(labels.contains(&"urgent".to_string()));
    assert!(labels.contains(&"backend".to_string()));
}

#[test]
fn remove_label() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    assert!(db.remove_label("test-1", "urgent").unwrap());
    assert!(!db.remove_label("test-1", "urgent").unwrap()); // Already removed

    let labels = db.get_labels("test-1").unwrap();
    assert!(labels.is_empty());
}

#[test]
fn add_and_get_dependencies() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();

    let deps = db.get_deps_from("test-1").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].from_id, "test-1");
    assert_eq!(deps[0].to_id, "test-2");
    assert_eq!(deps[0].relation, Relation::Blocks);
}

#[test]
fn self_dependency_error() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Issue 1");
    db.create_issue(&issue).unwrap();

    let result = db.add_dependency("test-1", "test-1", Relation::Blocks);
    assert!(matches!(result, Err(Error::SelfDependency)));
}

#[test]
fn cycle_detection() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    let issue3 = test_issue("test-3", "Issue 3");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();
    db.create_issue(&issue3).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();
    db.add_dependency("test-2", "test-3", Relation::Blocks)
        .unwrap();

    // This would create a cycle: test-3 -> test-1 -> test-2 -> test-3
    let result = db.add_dependency("test-3", "test-1", Relation::Blocks);
    assert!(matches!(result, Err(Error::CycleDetected)));
}

#[test]
fn get_blockers() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_dependency("test-1", "test-2", Relation::Blocks)
        .unwrap();

    let blockers = db.get_blockers("test-2").unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0], "test-1");
}

#[test]
fn issue_not_found() {
    let db = Database::open_in_memory().unwrap();
    let result = db.get_issue("nonexistent");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn get_all_labels() {
    let db = Database::open_in_memory().unwrap();
    let issue1 = test_issue("test-1", "Issue 1");
    let issue2 = test_issue("test-2", "Issue 2");
    db.create_issue(&issue1).unwrap();
    db.create_issue(&issue2).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    db.add_label("test-2", "backend").unwrap();

    let all_labels = db.get_all_labels().unwrap();
    assert_eq!(all_labels.len(), 2);
}

#[test]
fn resolve_id_exact_match() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("proj-abc123", "Test");
    db.create_issue(&issue).unwrap();

    let resolved = db.resolve_id("proj-abc123").unwrap();
    assert_eq!(resolved, "proj-abc123");
}

#[test]
fn resolve_id_prefix_match() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("proj-abc123", "Test");
    db.create_issue(&issue).unwrap();

    let resolved = db.resolve_id("proj-abc").unwrap();
    assert_eq!(resolved, "proj-abc123");
}

#[test]
fn resolve_id_too_short() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("proj-abc123", "Test");
    db.create_issue(&issue).unwrap();

    let result = db.resolve_id("pr");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn resolve_id_ambiguous() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("proj-abc1", "Issue 1"))
        .unwrap();
    db.create_issue(&test_issue("proj-abc2", "Issue 2"))
        .unwrap();

    let result = db.resolve_id("proj-abc");
    assert!(matches!(result, Err(Error::AmbiguousId { .. })));
}

#[test]
fn resolve_id_not_found() {
    let db = Database::open_in_memory().unwrap();
    let result = db.resolve_id("nonexistent");
    assert!(matches!(result, Err(Error::IssueNotFound(_))));
}

#[test]
fn search_issues_by_title() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("test-1", "Fix login bug"))
        .unwrap();
    db.create_issue(&test_issue("test-2", "Add dashboard"))
        .unwrap();

    let results = db.search_issues("login").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "test-1");
}

#[test]
fn search_issues_case_insensitive() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("test-1", "Fix Login Bug"))
        .unwrap();

    let results = db.search_issues("login").unwrap();
    assert_eq!(results.len(), 1);
}

#[test]
fn update_issue_description() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_description("test-1", "New description")
        .unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.description.as_deref(), Some("New description"));
}

#[test]
fn set_and_clear_assignee() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.set_assignee("test-1", "alice").unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert_eq!(retrieved.assignee.as_deref(), Some("alice"));

    db.clear_assignee("test-1").unwrap();
    let retrieved = db.get_issue("test-1").unwrap();
    assert!(retrieved.assignee.is_none());
}

#[test]
fn get_labels_batch() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("test-1", "Issue 1")).unwrap();
    db.create_issue(&test_issue("test-2", "Issue 2")).unwrap();

    db.add_label("test-1", "urgent").unwrap();
    db.add_label("test-1", "backend").unwrap();
    db.add_label("test-2", "frontend").unwrap();

    let batch = db.get_labels_batch(&["test-1", "test-2"]).unwrap();
    assert_eq!(batch.get("test-1").map(|v| v.len()), Some(2));
    assert_eq!(batch.get("test-2").map(|v| v.len()), Some(1));
}

#[test]
fn get_labels_batch_empty() {
    let db = Database::open_in_memory().unwrap();
    let batch = db.get_labels_batch(&[]).unwrap();
    assert!(batch.is_empty());
}

#[test]
fn add_and_get_links() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let link = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: Some(LinkType::Github),
        url: Some("https://github.com/org/repo/issues/1".to_string()),
        external_id: Some("1".to_string()),
        rel: None,
        created_at: Utc::now(),
    };
    db.add_link(&link).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Github));
}

#[test]
fn get_link_by_url() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let link = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: None,
        url: Some("https://example.com".to_string()),
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    };
    db.add_link(&link).unwrap();

    let found = db.get_link_by_url("test-1", "https://example.com").unwrap();
    assert!(found.is_some());

    let not_found = db.get_link_by_url("test-1", "https://other.com").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn remove_link() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let link = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: Some(LinkType::Github),
        url: Some("https://github.com/org/repo/issues/1".to_string()),
        external_id: Some("1".to_string()),
        rel: None,
        created_at: Utc::now(),
    };
    let link_id = db.add_link(&link).unwrap();

    assert_eq!(db.get_links("test-1").unwrap().len(), 1);
    db.remove_link(link_id).unwrap();
    assert_eq!(db.get_links("test-1").unwrap().len(), 0);
}

#[test]
fn replace_note() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_note("test-1", Status::Todo, "Original note")
        .unwrap();

    let result = db.replace_note("test-1", Status::Todo, "Replaced note");
    assert!(result.is_ok());

    let notes = db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].content, "Replaced note");
}

#[test]
fn replace_note_no_existing_notes() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let result = db.replace_note("test-1", Status::Todo, "New content");
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("no notes to replace"));
    }
}

#[test]
fn replace_note_updates_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_note("test-1", Status::Todo, "Original note")
        .unwrap();

    let result = db.replace_note("test-1", Status::InProgress, "Updated note");
    assert!(result.is_ok());

    let notes = db.get_notes("test-1").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].status, Status::InProgress);
    assert_eq!(notes[0].content, "Updated note");
}

#[test]
fn get_notes_by_status() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    db.add_note("test-1", Status::Todo, "Todo note 1").unwrap();
    db.add_note("test-1", Status::Todo, "Todo note 2").unwrap();
    db.add_note("test-1", Status::InProgress, "Progress note")
        .unwrap();

    let grouped = db.get_notes_by_status("test-1").unwrap();
    assert_eq!(grouped.len(), 2);

    let (status, notes) = &grouped[0];
    assert_eq!(*status, Status::Todo);
    assert_eq!(notes.len(), 2);
}

#[test]
fn get_notes_by_status_empty() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let grouped = db.get_notes_by_status("test-1").unwrap();
    assert!(grouped.is_empty());
}

#[test]
fn priority_from_tags_numeric() {
    assert_eq!(Database::priority_from_tags(&["priority:0".into()]), 0);
    assert_eq!(Database::priority_from_tags(&["priority:1".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["priority:2".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:3".into()]), 3);
    assert_eq!(Database::priority_from_tags(&["priority:4".into()]), 4);
}

#[test]
fn priority_from_tags_named() {
    assert_eq!(
        Database::priority_from_tags(&["priority:highest".into()]),
        0
    );
    assert_eq!(Database::priority_from_tags(&["priority:high".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["priority:medium".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:med".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:low".into()]), 3);
    assert_eq!(Database::priority_from_tags(&["priority:lowest".into()]), 4);
}

#[test]
fn priority_from_tags_prefers_priority_over_p() {
    let tags = vec!["p:0".into(), "priority:4".into()];
    assert_eq!(Database::priority_from_tags(&tags), 4);

    let tags2 = vec!["priority:3".into(), "p:1".into()];
    assert_eq!(Database::priority_from_tags(&tags2), 3);
}

#[test]
fn priority_from_tags_default() {
    assert_eq!(Database::priority_from_tags(&[]), 2);
    assert_eq!(Database::priority_from_tags(&["unrelated".into()]), 2);
    assert_eq!(
        Database::priority_from_tags(&["team:backend".into(), "urgent".into()]),
        2
    );
}

#[test]
fn priority_from_tags_p_fallback() {
    assert_eq!(Database::priority_from_tags(&["p:0".into()]), 0);
    assert_eq!(Database::priority_from_tags(&["p:1".into()]), 1);
    assert_eq!(Database::priority_from_tags(&["p:4".into()]), 4);
}

#[test]
fn priority_from_tags_invalid_values_ignored() {
    assert_eq!(
        Database::priority_from_tags(&["priority:invalid".into()]),
        2
    );
    assert_eq!(Database::priority_from_tags(&["priority:5".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:-1".into()]), 2);
    assert_eq!(Database::priority_from_tags(&["priority:".into()]), 2);
}

#[test]
fn priority_from_tags_first_valid_wins() {
    let tags = vec!["priority:1".into(), "priority:4".into()];
    assert_eq!(Database::priority_from_tags(&tags), 1);
}

#[test]
fn priority_from_tags_with_other_tags() {
    let tags = vec!["team:backend".into(), "priority:0".into(), "urgent".into()];
    assert_eq!(Database::priority_from_tags(&tags), 0);
}

#[test]
fn remove_all_links() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Test issue");
    db.create_issue(&issue).unwrap();

    let link1 = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: None,
        url: Some("https://a.com".to_string()),
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    };
    let link2 = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: None,
        url: Some("https://b.com".to_string()),
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    };
    db.add_link(&link1).unwrap();
    db.add_link(&link2).unwrap();

    assert_eq!(db.get_links("test-1").unwrap().len(), 2);
    db.remove_all_links("test-1").unwrap();
    assert_eq!(db.get_links("test-1").unwrap().len(), 0);
}

#[test]
fn ensure_prefix_creates_new() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("proj").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 1);
    assert_eq!(prefixes[0].prefix, "proj");
    assert_eq!(prefixes[0].issue_count, 0);
}

#[test]
fn ensure_prefix_idempotent() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("proj").unwrap();
    db.ensure_prefix("proj").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 1);
}

#[test]
fn increment_and_decrement_prefix_count() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("proj").unwrap();

    db.increment_prefix_count("proj").unwrap();
    db.increment_prefix_count("proj").unwrap();
    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes[0].issue_count, 2);

    db.decrement_prefix_count("proj").unwrap();
    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes[0].issue_count, 1);
}

#[test]
fn list_prefixes_ordered_by_count() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("alpha").unwrap();
    db.ensure_prefix("beta").unwrap();

    db.increment_prefix_count("alpha").unwrap();
    db.increment_prefix_count("beta").unwrap();
    db.increment_prefix_count("beta").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 2);
    assert_eq!(prefixes[0].prefix, "beta");
    assert_eq!(prefixes[0].issue_count, 2);
    assert_eq!(prefixes[1].prefix, "alpha");
    assert_eq!(prefixes[1].issue_count, 1);
}

#[test]
fn rename_prefix_no_conflict() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("old").unwrap();
    db.increment_prefix_count("old").unwrap();

    db.rename_prefix("old", "new").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 1);
    assert_eq!(prefixes[0].prefix, "new");
    assert_eq!(prefixes[0].issue_count, 1);
}

#[test]
fn rename_prefix_merges_counts() {
    let db = Database::open_in_memory().unwrap();
    db.ensure_prefix("old").unwrap();
    db.ensure_prefix("new").unwrap();

    db.increment_prefix_count("old").unwrap();
    db.increment_prefix_count("old").unwrap();
    db.increment_prefix_count("new").unwrap();

    db.rename_prefix("old", "new").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 1);
    assert_eq!(prefixes[0].prefix, "new");
    assert_eq!(prefixes[0].issue_count, 3);
}

#[test]
fn rename_nonexistent_prefix_is_noop() {
    let db = Database::open_in_memory().unwrap();
    db.rename_prefix("ghost", "new").unwrap();

    let prefixes = db.list_prefixes().unwrap();
    assert!(prefixes.is_empty());
}

#[test]
fn closed_at_none_for_open_issue() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Open issue");
    db.create_issue(&issue).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert!(retrieved.closed_at.is_none());
}

#[test]
fn closed_at_set_when_done() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Done issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_status("test-1", Status::Done).unwrap();
    let event = Event::new("test-1".to_string(), Action::Done)
        .with_values(Some("todo".to_string()), Some("done".to_string()));
    db.log_event(&event).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert!(retrieved.closed_at.is_some());
}

#[test]
fn closed_at_set_when_closed() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Closed issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_status("test-1", Status::Closed).unwrap();
    let event = Event::new("test-1".to_string(), Action::Closed)
        .with_values(Some("todo".to_string()), Some("closed".to_string()));
    db.log_event(&event).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert!(retrieved.closed_at.is_some());
}

#[test]
fn closed_at_none_after_reopen() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Reopened issue");
    db.create_issue(&issue).unwrap();

    // Close the issue
    db.update_issue_status("test-1", Status::Done).unwrap();
    let event = Event::new("test-1".to_string(), Action::Done)
        .with_values(Some("todo".to_string()), Some("done".to_string()));
    db.log_event(&event).unwrap();

    // Reopen the issue
    db.update_issue_status("test-1", Status::InProgress)
        .unwrap();
    let event = Event::new("test-1".to_string(), Action::Reopened)
        .with_values(Some("done".to_string()), Some("in_progress".to_string()));
    db.log_event(&event).unwrap();

    let retrieved = db.get_issue("test-1").unwrap();
    assert!(retrieved.closed_at.is_none());
}

#[test]
fn closed_at_in_list_issues() {
    let db = Database::open_in_memory().unwrap();
    let issue = test_issue("test-1", "Done issue");
    db.create_issue(&issue).unwrap();

    db.update_issue_status("test-1", Status::Done).unwrap();
    let event = Event::new("test-1".to_string(), Action::Done)
        .with_values(Some("todo".to_string()), Some("done".to_string()));
    db.log_event(&event).unwrap();

    let issues = db.list_issues(Some(Status::Done), None, None).unwrap();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].closed_at.is_some());
}

/// Create a database with a minimal old-style schema (no assignee, no HLC columns,
/// no closed_at, no prefixes table), insert data, then run migrations and verify
/// everything works.
#[test]
fn migrate_old_schema_adds_missing_columns_and_backfills() {
    use rusqlite::Connection;

    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    // Old-style schema: no assignee, no HLC columns, no closed_at, no prefixes table
    conn.execute_batch(
        "CREATE TABLE issues (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'todo',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE deps (
            from_id TEXT NOT NULL,
            to_id TEXT NOT NULL,
            rel TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (from_id, to_id, rel)
        );
        CREATE TABLE labels (
            issue_id TEXT NOT NULL,
            label TEXT NOT NULL,
            PRIMARY KEY (issue_id, label)
        );
        CREATE TABLE notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id TEXT NOT NULL,
            status TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id TEXT NOT NULL,
            action TEXT NOT NULL,
            old_value TEXT,
            new_value TEXT,
            reason TEXT,
            created_at TEXT NOT NULL
        );
        CREATE TABLE links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id TEXT NOT NULL,
            link_type TEXT,
            url TEXT,
            external_id TEXT,
            rel TEXT,
            created_at TEXT NOT NULL
        );",
    )
    .unwrap();

    // Insert issues using old schema (no assignee / HLC / closed_at)
    conn.execute(
        "INSERT INTO issues (id, type, title, status, created_at, updated_at)
         VALUES ('proj-abc1', 'task', 'First task', 'done', '2026-01-01T00:00:00Z', '2026-01-02T00:00:00Z')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO issues (id, type, title, status, created_at, updated_at)
         VALUES ('proj-def2', 'bug', 'A bug', 'todo', '2026-01-03T00:00:00Z', '2026-01-03T00:00:00Z')",
        [],
    )
    .unwrap();

    // Add a done event for the first issue (for closed_at backfill)
    conn.execute(
        "INSERT INTO events (issue_id, action, created_at)
         VALUES ('proj-abc1', 'done', '2026-01-02T00:00:00Z')",
        [],
    )
    .unwrap();

    // Add an old-style tracked_by dependency
    conn.execute(
        "INSERT INTO deps (from_id, to_id, rel, created_at)
         VALUES ('proj-abc1', 'proj-def2', 'tracked_by', '2026-01-01T00:00:00Z')",
        [],
    )
    .unwrap();

    // Now wrap with Database and run migrations via free function
    let db = Database { conn };
    run_migrations(&db.conn).unwrap();

    // Verify assignee column exists and is readable
    let issue = db.get_issue("proj-abc1").unwrap();
    assert!(issue.assignee.is_none());

    // Verify HLC columns exist and are NULL
    assert!(issue.last_status_hlc.is_none());
    assert!(issue.last_title_hlc.is_none());
    assert!(issue.last_type_hlc.is_none());
    assert!(issue.last_description_hlc.is_none());
    assert!(issue.last_assignee_hlc.is_none());

    // Verify closed_at was backfilled for the done issue
    assert!(issue.closed_at.is_some());
    assert_eq!(
        issue.closed_at.unwrap().to_rfc3339(),
        "2026-01-02T00:00:00+00:00"
    );

    // Verify the todo issue has no closed_at
    let bug = db.get_issue("proj-def2").unwrap();
    assert!(bug.closed_at.is_none());

    // Verify prefixes were backfilled
    let prefixes = db.list_prefixes().unwrap();
    assert_eq!(prefixes.len(), 1);
    assert_eq!(prefixes[0].prefix, "proj");
    assert_eq!(prefixes[0].issue_count, 2);

    // Verify tracked_by was migrated to tracked-by
    let deps = db.get_deps_from("proj-abc1").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].relation, Relation::TrackedBy);
}

#[test]
fn get_deps_to() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("a", "A")).unwrap();
    db.create_issue(&test_issue("b", "B")).unwrap();
    db.create_issue(&test_issue("c", "C")).unwrap();

    db.add_dependency("a", "b", Relation::Blocks).unwrap();
    db.add_dependency("c", "b", Relation::Tracks).unwrap();

    let deps = db.get_deps_to("b").unwrap();
    assert_eq!(deps.len(), 2);

    let from_ids: Vec<&str> = deps.iter().map(|d| d.from_id.as_str()).collect();
    assert!(from_ids.contains(&"a"));
    assert!(from_ids.contains(&"c"));
}

#[test]
fn get_transitive_blocker_deps() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("a", "A")).unwrap();
    db.create_issue(&test_issue("b", "B")).unwrap();
    db.create_issue(&test_issue("c", "C")).unwrap();

    // a blocks b, b blocks c
    db.add_dependency("a", "b", Relation::Blocks).unwrap();
    db.add_dependency("b", "c", Relation::Blocks).unwrap();

    let deps = db.get_transitive_blocker_deps("c").unwrap();
    assert_eq!(deps.len(), 2);

    let blocker_ids: Vec<&str> = deps.iter().map(|d| d.from_id.as_str()).collect();
    assert!(blocker_ids.contains(&"a"));
    assert!(blocker_ids.contains(&"b"));

    // Mark b as done; only a should remain as transitive blocker
    db.update_issue_status("b", Status::Done).unwrap();
    let deps = db.get_transitive_blocker_deps("c").unwrap();
    // b is done so filtered out, but a (still todo) is reachable through done b
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].from_id, "a");
}

#[test]
fn remove_link_by_url() {
    let db = Database::open_in_memory().unwrap();
    db.create_issue(&test_issue("test-1", "Test")).unwrap();

    let link = Link {
        id: 0,
        issue_id: "test-1".to_string(),
        link_type: None,
        url: Some("https://example.com".to_string()),
        external_id: None,
        rel: None,
        created_at: Utc::now(),
    };
    db.add_link(&link).unwrap();
    assert_eq!(db.get_links("test-1").unwrap().len(), 1);

    db.remove_link_by_url("test-1", "https://example.com")
        .unwrap();
    assert_eq!(db.get_links("test-1").unwrap().len(), 0);
}
