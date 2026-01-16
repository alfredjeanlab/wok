// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

// Relation tests - parameterized
#[parameterized(
    blocks = { Relation::Blocks, "blocks" },
    tracked_by = { Relation::TrackedBy, "tracked-by" },
    tracks = { Relation::Tracks, "tracks" },
)]
fn test_relation_roundtrip(relation: Relation, expected: &str) {
    assert_eq!(relation.as_str(), expected);
    assert_eq!(relation.to_string(), expected);
    assert_eq!(expected.parse::<Relation>().unwrap(), relation);
}

#[parameterized(
    blocks_upper = { "BLOCKS", Relation::Blocks },
    tracked_by_mixed = { "Tracked-By", Relation::TrackedBy },
)]
fn test_relation_from_str_case_insensitive(input: &str, expected: Relation) {
    assert_eq!(input.parse::<Relation>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    child_of = { "child-of" },
)]
fn test_relation_from_str_invalid(input: &str) {
    assert!(input.parse::<Relation>().is_err());
}

// UserRelation tests - parameterized
#[parameterized(
    blocks = { "blocks", UserRelation::Blocks },
    blocks_upper = { "BLOCKS", UserRelation::Blocks },
    blocked_by_hyphen = { "blocked-by", UserRelation::BlockedBy },
    blocked_by_underscore = { "blocked_by", UserRelation::BlockedBy },
    blockedby = { "blockedby", UserRelation::BlockedBy },
    blockedby_upper = { "BLOCKED-BY", UserRelation::BlockedBy },
    tracks = { "tracks", UserRelation::Tracks },
    tracks_upper = { "TRACKS", UserRelation::Tracks },
    contains = { "contains", UserRelation::Tracks },
    tracked_by_hyphen = { "tracked-by", UserRelation::TrackedBy },
    tracked_by_underscore = { "tracked_by", UserRelation::TrackedBy },
    trackedby = { "trackedby", UserRelation::TrackedBy },
    trackedby_upper = { "TRACKED-BY", UserRelation::TrackedBy },
)]
fn test_user_relation_from_str_valid(input: &str, expected: UserRelation) {
    assert_eq!(input.parse::<UserRelation>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    child_of = { "child-of" },
)]
fn test_user_relation_from_str_invalid(input: &str) {
    assert!(input.parse::<UserRelation>().is_err());
}

// Dependency tests - parameterized
#[parameterized(
    blocks = { Relation::Blocks },
    tracked_by = { Relation::TrackedBy },
    tracks = { Relation::Tracks },
)]
fn test_dependency_new(relation: Relation) {
    let dep = Dependency::new("a".to_string(), "b".to_string(), relation);
    assert_eq!(dep.from_id, "a");
    assert_eq!(dep.to_id, "b");
    assert_eq!(dep.relation, relation);
}
