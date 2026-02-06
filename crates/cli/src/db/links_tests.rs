// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::db::Database;
use crate::models::{Issue, IssueType, Status};
use chrono::Utc;

fn setup_db() -> Database {
    Database::open_in_memory().unwrap()
}

fn create_issue(db: &Database, id: &str) {
    let issue = Issue {
        id: id.to_string(),
        issue_type: IssueType::Task,
        title: format!("Test issue {}", id),
        description: None,
        status: Status::Todo,
        assignee: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        closed_at: None,
    };
    db.create_issue(&issue).unwrap();
}

#[test]
fn test_add_link_basic() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link = new_link("test-1");
    link.url = Some("https://github.com/org/repo/issues/123".to_string());
    link.link_type = Some(LinkType::Github);
    link.external_id = Some("123".to_string());

    let id = db.add_link(&link).unwrap();
    assert!(id > 0);
}

#[test]
fn test_add_link_with_rel() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link = new_link("test-1");
    link.url = Some("https://company.atlassian.net/browse/PE-5555".to_string());
    link.link_type = Some(LinkType::Jira);
    link.external_id = Some("PE-5555".to_string());
    link.rel = Some(LinkRel::Import);

    let id = db.add_link(&link).unwrap();
    assert!(id > 0);
}

#[test]
fn test_get_links_empty() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let links = db.get_links("test-1").unwrap();
    assert!(links.is_empty());
}

#[test]
fn test_get_links_single() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link = new_link("test-1");
    link.url = Some("https://github.com/org/repo/issues/123".to_string());
    link.link_type = Some(LinkType::Github);
    link.external_id = Some("123".to_string());
    db.add_link(&link).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].url,
        Some("https://github.com/org/repo/issues/123".to_string())
    );
    assert_eq!(links[0].link_type, Some(LinkType::Github));
    assert_eq!(links[0].external_id, Some("123".to_string()));
}

#[test]
fn test_get_links_multiple() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link1 = new_link("test-1");
    link1.url = Some("https://github.com/org/repo/issues/123".to_string());
    link1.link_type = Some(LinkType::Github);
    db.add_link(&link1).unwrap();

    let mut link2 = new_link("test-1");
    link2.url = Some("jira://PE-5555".to_string());
    link2.link_type = Some(LinkType::Jira);
    link2.external_id = Some("PE-5555".to_string());
    db.add_link(&link2).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 2);
}

#[test]
fn test_get_links_ordered_by_created_at() {
    let db = setup_db();
    create_issue(&db, "test-1");

    // Add links with different timestamps
    let mut link1 = new_link("test-1");
    link1.url = Some("first".to_string());
    db.add_link(&link1).unwrap();

    let mut link2 = new_link("test-1");
    link2.url = Some("second".to_string());
    db.add_link(&link2).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].url, Some("first".to_string()));
    assert_eq!(links[1].url, Some("second".to_string()));
}

#[test]
fn test_remove_link() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link = new_link("test-1");
    link.url = Some("https://github.com/org/repo/issues/123".to_string());
    let id = db.add_link(&link).unwrap();

    db.remove_link(id).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert!(links.is_empty());
}

#[test]
fn test_remove_link_nonexistent() {
    let db = setup_db();
    // Should not error, just do nothing
    db.remove_link(999999).unwrap();
}

#[test]
fn test_links_different_issues() {
    let db = setup_db();
    create_issue(&db, "test-1");
    create_issue(&db, "test-2");

    let mut link1 = new_link("test-1");
    link1.url = Some("for-test-1".to_string());
    db.add_link(&link1).unwrap();

    let mut link2 = new_link("test-2");
    link2.url = Some("for-test-2".to_string());
    db.add_link(&link2).unwrap();

    let links1 = db.get_links("test-1").unwrap();
    assert_eq!(links1.len(), 1);
    assert_eq!(links1[0].url, Some("for-test-1".to_string()));

    let links2 = db.get_links("test-2").unwrap();
    assert_eq!(links2.len(), 1);
    assert_eq!(links2[0].url, Some("for-test-2".to_string()));
}

#[test]
fn test_link_with_all_optional_fields_none() {
    let db = setup_db();
    create_issue(&db, "test-1");

    // Link with only issue_id and created_at
    let link = new_link("test-1");
    let id = db.add_link(&link).unwrap();
    assert!(id > 0);

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, None);
    assert_eq!(links[0].url, None);
    assert_eq!(links[0].external_id, None);
    assert_eq!(links[0].rel, None);
}

#[test]
fn test_link_preserves_rel() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let mut link = new_link("test-1");
    link.url = Some("https://github.com/org/repo/issues/123".to_string());
    link.rel = Some(LinkRel::TrackedBy);
    db.add_link(&link).unwrap();

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links[0].rel, Some(LinkRel::TrackedBy));
}
