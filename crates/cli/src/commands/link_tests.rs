// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use crate::commands::testing::TestContext;
use crate::models::{IssueType, LinkRel, LinkType};

#[test]
fn test_add_link_github() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    );
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Github));
    assert_eq!(links[0].external_id, Some("123".to_string()));
}

#[test]
fn test_add_link_jira_shorthand() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(&ctx.db, "test-1", "jira://PE-5555", None);
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Jira));
    assert_eq!(links[0].external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_add_link_jira_atlassian() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://company.atlassian.net/browse/PE-5555",
        None,
    );
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Jira));
    assert_eq!(links[0].external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_add_link_confluence() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://company.atlassian.net/wiki/spaces/DOC/pages/123",
        None,
    );
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Confluence));
    // Confluence doesn't have extractable issue IDs
    assert_eq!(links[0].external_id, None);
}

#[test]
fn test_add_link_unknown_url() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(&ctx.db, "test-1", "https://example.com/issue/123", None);
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, None);
    assert_eq!(links[0].external_id, None);
}

#[test]
fn test_add_link_with_reason() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/456",
        Some("tracks".to_string()),
    );
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].rel, Some(LinkRel::Tracks));
}

#[test]
fn test_add_link_import_requires_known_provider() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
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
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Confluence URLs don't have extractable IDs
    let result = add_impl_with_reason(
        &ctx.db,
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
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // GitHub URL has both known provider and detectable ID
    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/789",
        Some("import".to_string()),
    );
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links[0].rel, Some(LinkRel::Import));
}

#[test]
fn test_add_link_nonexistent_issue() {
    let mut ctx = TestContext::new();

    let result = add_impl_with_reason(
        &ctx.db,
        "nonexistent",
        "https://github.com/org/repo/issues/123",
        None,
    );
    assert!(result.is_err());
}

#[test]
fn test_add_link_invalid_reason() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        Some("invalid".to_string()),
    );
    assert!(result.is_err());
}

#[test]
fn test_add_link_logs_event() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Linked));
}

#[test]
fn test_add_link_impl_basic() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    let result = add_link_impl(&ctx.db, "test-1", "https://github.com/org/repo/issues/999");
    assert!(result.is_ok());

    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].link_type, Some(LinkType::Github));
}

#[test]
fn test_remove_link() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Add a link first
    add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    // Verify link exists
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);

    // Remove the link
    let result = remove_impl(&ctx.db, "test-1", "https://github.com/org/repo/issues/123");
    assert!(result.is_ok());

    // Verify link is gone
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 0);
}

#[test]
fn test_remove_link_nonexistent_url() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Try to remove a link that doesn't exist (should succeed with message)
    let result = remove_impl(&ctx.db, "test-1", "https://example.com/not-linked");
    assert!(result.is_ok());
}

#[test]
fn test_remove_link_nonexistent_issue() {
    let mut ctx = TestContext::new();

    let result = remove_impl(
        &ctx.db,
        "nonexistent",
        "https://github.com/org/repo/issues/123",
    );
    assert!(result.is_err());
}

#[test]
fn test_remove_link_logs_event() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/123",
        None,
    )
    .unwrap();

    remove_impl(&ctx.db, "test-1", "https://github.com/org/repo/issues/123").unwrap();

    let events = ctx.db.get_events("test-1").unwrap();
    assert!(events.iter().any(|e| e.action == Action::Unlinked));
}

#[test]
fn test_remove_link_multiple_links() {
    let mut ctx = TestContext::new();
    ctx.create_issue("test-1", IssueType::Task, "Test");

    // Add multiple links
    add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/1",
        None,
    )
    .unwrap();
    add_impl_with_reason(
        &ctx.db,
        "test-1",
        "https://github.com/org/repo/issues/2",
        None,
    )
    .unwrap();

    // Remove only one
    remove_impl(&ctx.db, "test-1", "https://github.com/org/repo/issues/1").unwrap();

    // Verify only one remains
    let links = ctx.db.get_links("test-1").unwrap();
    assert_eq!(links.len(), 1);
    assert_eq!(
        links[0].url,
        Some("https://github.com/org/repo/issues/2".to_string())
    );
}
