// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::db::Database;
use crate::models::{Issue, IssueType, LinkRel, LinkType, Status};
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
fn test_add_link_github() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    );
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Github));
    assert_eq!(links[0].external_id, Some("123".to_string()));
}

#[test]
fn test_add_link_jira_shorthand() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(&db, "test-1", "jira://PE-5555", None);
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Jira));
    assert_eq!(links[0].external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_add_link_jira_atlassian() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://company.atlassian.net/browse/PE-5555",
        None,
    );
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Jira));
    assert_eq!(links[0].external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_add_link_confluence() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://company.atlassian.net/wiki/spaces/DOC/pages/123",
        None,
    );
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Confluence));
    // Confluence doesn't have extractable issue IDs
    assert_eq!(links[0].external_id, None);
}

#[test]
fn test_add_link_unknown_url() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(&db, "test-1", "https://example.com/issue/123", None);
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, None);
    assert_eq!(links[0].external_id, None);
}

#[test]
fn test_add_link_with_reason() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://github.com/org/repo/issues/456",
        Some("tracks".to_string()),
    );
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].rel, Some(LinkRel::Tracks));
}

#[test]
fn test_add_link_import_requires_known_provider() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://example.com/issue/123",
        Some("import".to_string()),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("requires a known provider"));
}

#[test]
fn test_add_link_import_requires_detectable_id() {
    let db = setup_db();
    create_issue(&db, "test-1");

    // Confluence URLs don't have extractable IDs
    let result = add_impl(
        &db,
        "test-1",
        "https://company.atlassian.net/wiki/spaces/DOC/pages/123",
        Some("import".to_string()),
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("requires a detectable issue ID"));
}

#[test]
fn test_add_link_import_success() {
    let db = setup_db();
    create_issue(&db, "test-1");

    // GitHub URL has both known provider and detectable ID
    let result = add_impl(
        &db,
        "test-1",
        "https://github.com/org/repo/issues/789",
        Some("import".to_string()),
    );
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links[0].rel, Some(LinkRel::Import));
}

#[test]
fn test_add_link_nonexistent_issue() {
    let db = setup_db();

    let result = add_impl(
        &db,
        "nonexistent",
        "https://github.com/org/repo/issues/123",
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_add_link_invalid_reason() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_impl(
        &db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        Some("invalid".to_string()),
    );
    assert!(result.is_err());
}

#[test]
fn test_add_link_logs_event() {
    let db = setup_db();
    create_issue(&db, "test-1");

    add_impl(
        &db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    let events = db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Linked));
}

#[test]
fn test_add_link_impl_basic() {
    let db = setup_db();
    create_issue(&db, "test-1");

    let result = add_link_impl(&db, "test-1", "https://github.com/org/repo/issues/999");
    assert!(result.is_ok());

    let links = db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Github));
}
