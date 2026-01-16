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
fn test_user_relation_parsing() {
    assert_eq!(
        "blocks".parse::<UserRelation>().unwrap(),
        UserRelation::Blocks
    );
    assert_eq!(
        "tracks".parse::<UserRelation>().unwrap(),
        UserRelation::Tracks
    );
    assert!("depends".parse::<UserRelation>().is_err());
}

#[test]
fn test_blocks_creates_single_dependency() {
    let db = setup_db();
    create_issue(&db, "issue-a");
    create_issue(&db, "issue-b");

    // Add blocks dependency
    db.add_dependency("issue-a", "issue-b", Relation::Blocks)
        .unwrap();

    // Verify only one dependency exists
    let deps = db.get_deps_from("issue-a").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].relation, Relation::Blocks);
}

#[test]
fn test_tracks_creates_bidirectional_dependencies() {
    let db = setup_db();
    create_issue(&db, "parent");
    create_issue(&db, "child");

    // Add tracks relationship (parent tracks child)
    // This creates: parent -> child (tracks) and child -> parent (tracked-by)
    db.add_dependency("parent", "child", Relation::Tracks)
        .unwrap();
    db.add_dependency("child", "parent", Relation::TrackedBy)
        .unwrap();

    // Verify tracks dependency
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps
        .iter()
        .any(|d| d.relation == Relation::Tracks && d.to_id == "child"));

    // Verify tracked-by dependency
    let child_deps = db.get_deps_from("child").unwrap();
    assert!(child_deps
        .iter()
        .any(|d| d.relation == Relation::TrackedBy && d.to_id == "parent"));
}

#[test]
fn test_remove_tracks_removes_both_directions() {
    let db = setup_db();
    create_issue(&db, "parent");
    create_issue(&db, "child");

    // Add tracks relationship
    db.add_dependency("parent", "child", Relation::Tracks)
        .unwrap();
    db.add_dependency("child", "parent", Relation::TrackedBy)
        .unwrap();

    // Remove both directions
    db.remove_dependency("parent", "child", Relation::Tracks)
        .unwrap();
    db.remove_dependency("child", "parent", Relation::TrackedBy)
        .unwrap();

    // Verify no dependencies remain
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps.is_empty());
    let child_deps = db.get_deps_from("child").unwrap();
    assert!(child_deps.is_empty());
}

// Tests for run_impl

use crate::commands::dep::{add_impl, remove_impl};

#[test]
fn test_add_impl_blocks() {
    let db = setup_db();
    create_issue(&db, "blocker");
    create_issue(&db, "blocked");

    let result = add_impl(&db, "blocker", "blocks", &["blocked".to_string()]);
    assert!(result.is_ok());

    let deps = db.get_deps_from("blocker").unwrap();
    assert!(deps.iter().any(|d| d.relation == Relation::Blocks));
}

#[test]
fn test_add_impl_tracks() {
    let db = setup_db();
    create_issue(&db, "parent");
    create_issue(&db, "child");

    let result = add_impl(&db, "parent", "tracks", &["child".to_string()]);
    assert!(result.is_ok());

    // tracks creates tracks and tracked-by
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps.iter().any(|d| d.relation == Relation::Tracks));
}

#[test]
fn test_add_impl_multiple_targets() {
    let db = setup_db();
    create_issue(&db, "blocker");
    create_issue(&db, "blocked1");
    create_issue(&db, "blocked2");

    let result = add_impl(
        &db,
        "blocker",
        "blocks",
        &["blocked1".to_string(), "blocked2".to_string()],
    );
    assert!(result.is_ok());

    let deps = db.get_deps_from("blocker").unwrap();
    assert_eq!(deps.len(), 2);
}

#[test]
fn test_add_impl_nonexistent_source() {
    let db = setup_db();
    create_issue(&db, "target");

    let result = add_impl(&db, "nonexistent", "blocks", &["target".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_add_impl_nonexistent_target() {
    let db = setup_db();
    create_issue(&db, "source");

    let result = add_impl(&db, "source", "blocks", &["nonexistent".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_add_impl_invalid_relation() {
    let db = setup_db();
    create_issue(&db, "a");
    create_issue(&db, "b");

    let result = add_impl(&db, "a", "invalid", &["b".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_remove_impl_blocks() {
    let db = setup_db();
    create_issue(&db, "blocker");
    create_issue(&db, "blocked");
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    let result = remove_impl(&db, "blocker", "blocks", &["blocked".to_string()]);
    assert!(result.is_ok());

    let deps = db.get_deps_from("blocker").unwrap();
    assert!(deps.is_empty());
}

#[test]
fn test_remove_impl_tracks() {
    let db = setup_db();
    create_issue(&db, "parent");
    create_issue(&db, "child");
    db.add_dependency("parent", "child", Relation::Tracks)
        .unwrap();
    db.add_dependency("child", "parent", Relation::TrackedBy)
        .unwrap();

    let result = remove_impl(&db, "parent", "tracks", &["child".to_string()]);
    assert!(result.is_ok());

    // Both tracks and tracked-by should be removed
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps.is_empty());
}

// Tests for blocked-by and tracked-by relations

#[test]
fn test_add_impl_blocked_by() {
    let db = setup_db();
    create_issue(&db, "blocked");
    create_issue(&db, "blocker");

    // "blocked blocked-by blocker" means "blocker blocks blocked"
    let result = add_impl(&db, "blocked", "blocked-by", &["blocker".to_string()]);
    assert!(result.is_ok());

    // The dependency should be stored as "blocker blocks blocked"
    let deps = db.get_deps_from("blocker").unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].relation, Relation::Blocks);
    assert_eq!(deps[0].to_id, "blocked");
}

#[test]
fn test_add_impl_blocked_by_multiple_targets() {
    let db = setup_db();
    create_issue(&db, "blocked");
    create_issue(&db, "blocker1");
    create_issue(&db, "blocker2");
    create_issue(&db, "blocker3");

    // "blocked blocked-by blocker1 blocker2 blocker3"
    let result = add_impl(
        &db,
        "blocked",
        "blocked-by",
        &[
            "blocker1".to_string(),
            "blocker2".to_string(),
            "blocker3".to_string(),
        ],
    );
    assert!(result.is_ok());

    // Each blocker should have a blocks dependency to the blocked issue
    for blocker in &["blocker1", "blocker2", "blocker3"] {
        let deps = db.get_deps_from(blocker).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].relation, Relation::Blocks);
        assert_eq!(deps[0].to_id, "blocked");
    }
}

#[test]
fn test_add_impl_tracked_by() {
    let db = setup_db();
    create_issue(&db, "child");
    create_issue(&db, "parent");

    // "child tracked-by parent" means "parent tracks child"
    let result = add_impl(&db, "child", "tracked-by", &["parent".to_string()]);
    assert!(result.is_ok());

    // Parent should have tracks dependency to child
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps
        .iter()
        .any(|d| d.relation == Relation::Tracks && d.to_id == "child"));

    // Child should have tracked-by dependency to parent
    let child_deps = db.get_deps_from("child").unwrap();
    assert!(child_deps
        .iter()
        .any(|d| d.relation == Relation::TrackedBy && d.to_id == "parent"));
}

#[test]
fn test_remove_impl_blocked_by() {
    let db = setup_db();
    create_issue(&db, "blocked");
    create_issue(&db, "blocker");

    // Set up the dependency (blocker blocks blocked)
    db.add_dependency("blocker", "blocked", Relation::Blocks)
        .unwrap();

    // Remove using blocked-by syntax
    let result = remove_impl(&db, "blocked", "blocked-by", &["blocker".to_string()]);
    assert!(result.is_ok());

    let deps = db.get_deps_from("blocker").unwrap();
    assert!(deps.is_empty());
}

#[test]
fn test_remove_impl_tracked_by() {
    let db = setup_db();
    create_issue(&db, "child");
    create_issue(&db, "parent");

    // Set up the bidirectional dependency
    db.add_dependency("parent", "child", Relation::Tracks)
        .unwrap();
    db.add_dependency("child", "parent", Relation::TrackedBy)
        .unwrap();

    // Remove using tracked-by syntax
    let result = remove_impl(&db, "child", "tracked-by", &["parent".to_string()]);
    assert!(result.is_ok());

    // Both tracks and tracked-by should be removed
    let parent_deps = db.get_deps_from("parent").unwrap();
    assert!(parent_deps.is_empty());
    let child_deps = db.get_deps_from("child").unwrap();
    assert!(child_deps.is_empty());
}

#[test]
fn test_add_impl_blocked_by_alternate_spellings() {
    let db = setup_db();
    create_issue(&db, "a");
    create_issue(&db, "b");
    create_issue(&db, "c");
    create_issue(&db, "d");

    // Test blocked_by (underscore)
    let result = add_impl(&db, "a", "blocked_by", &["b".to_string()]);
    assert!(result.is_ok());

    // Test blockedby (no separator)
    let result = add_impl(&db, "c", "blockedby", &["d".to_string()]);
    assert!(result.is_ok());

    // Verify both created correct dependencies
    let b_deps = db.get_deps_from("b").unwrap();
    assert!(b_deps
        .iter()
        .any(|dep| dep.relation == Relation::Blocks && dep.to_id == "a"));

    let d_deps = db.get_deps_from("d").unwrap();
    assert!(d_deps
        .iter()
        .any(|dep| dep.relation == Relation::Blocks && dep.to_id == "c"));
}
