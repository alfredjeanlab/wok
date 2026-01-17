// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Core data models for the issue tracker.
//!
//! This module contains all the domain types used throughout the application:
//!
//! - [`Issue`] - The primary entity representing a task, bug, or epic
//! - [`IssueType`] - Classification of issues (Task, Bug, Epic)
//! - [`Status`] - Workflow states (Todo, InProgress, Done, Closed)
//! - [`Event`] - Audit log entries tracking issue changes
//! - [`Action`] - Types of actions recorded in events
//! - [`Note`] - User notes attached to issues
//! - [`Dependency`] - Relationships between issues
//! - [`Relation`] - Types of dependencies (Blocks, Tracks, TrackedBy)
//! - [`Link`] - External links to issue trackers (GitHub, Jira, etc.)
//! - [`LinkType`] - Type of external link provider
//! - [`LinkRel`] - Relationship of link to issue

mod dependency;
mod event;
mod issue;
mod link;
mod note;

pub use dependency::{Dependency, Relation, UserRelation};
pub use event::{Action, Event};
pub use issue::{Issue, Status};
pub use link::{parse_link_url, Link, LinkRel, LinkType};
pub use note::Note;
pub use wk_core::IssueType;
