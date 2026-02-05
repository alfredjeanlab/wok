// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use yare::parameterized;

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
