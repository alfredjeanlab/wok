// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Core data models for the issue tracker.
//!
//! Model types are defined in the shared `wk_ipc` crate and re-exported here.
//! CLI-specific additions (UserRelation, parse_link_url) are defined locally.

mod dependency;
mod link;

pub use dependency::UserRelation;
pub use link::parse_link_url;
pub use wk_ipc::{
    Action, Dependency, Event, Issue, IssueType, Link, LinkRel, LinkType, Note, PrefixInfo,
    Relation, Status,
};
