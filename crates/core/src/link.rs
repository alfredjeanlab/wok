// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Link types for external issue tracker integration.
//!
//! This module contains types for linking wok issues to external trackers
//! like GitHub, Jira, GitLab, and Confluence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Type of external link (auto-detected from URL).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Github,
    Jira,
    Gitlab,
    Confluence,
}

impl LinkType {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkType::Github => "github",
            LinkType::Jira => "jira",
            LinkType::Gitlab => "gitlab",
            LinkType::Confluence => "confluence",
        }
    }
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LinkType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "github" => Ok(LinkType::Github),
            "jira" => Ok(LinkType::Jira),
            "gitlab" => Ok(LinkType::Gitlab),
            "confluence" => Ok(LinkType::Confluence),
            _ => Err(Error::InvalidLinkType(s.to_string())),
        }
    }
}

/// Relationship of external link to issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkRel {
    /// Issue was imported from this external source.
    Import,
    /// External issue blocks this issue.
    Blocks,
    /// This issue tracks the external issue.
    Tracks,
    /// This issue is tracked by the external issue.
    TrackedBy,
}

impl LinkRel {
    /// Returns the string representation used in storage and display.
    pub fn as_str(&self) -> &'static str {
        match self {
            LinkRel::Import => "import",
            LinkRel::Blocks => "blocks",
            LinkRel::Tracks => "tracks",
            LinkRel::TrackedBy => "tracked-by",
        }
    }
}

impl fmt::Display for LinkRel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LinkRel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "import" => Ok(LinkRel::Import),
            "blocks" => Ok(LinkRel::Blocks),
            "tracks" => Ok(LinkRel::Tracks),
            "tracked-by" => Ok(LinkRel::TrackedBy),
            _ => Err(Error::InvalidLinkRel(s.to_string())),
        }
    }
}

/// An external link attached to an issue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// Database-assigned identifier.
    pub id: i64,
    /// The issue this link belongs to.
    pub issue_id: String,
    /// Type of external link (auto-detected from URL).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_type: Option<LinkType>,
    /// Full URL (may be None for shorthand like jira://PE-5555).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// External issue ID (e.g., "PE-5555" for Jira, "123" for GitHub).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// Relationship to the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rel: Option<LinkRel>,
    /// When the link was created.
    pub created_at: DateTime<Utc>,
}

impl Link {
    /// Creates a new link with the current timestamp.
    pub fn new(issue_id: String) -> Self {
        Link {
            id: 0, // Will be set by database
            issue_id,
            link_type: None,
            url: None,
            external_id: None,
            rel: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the link type (builder pattern).
    pub fn with_type(mut self, link_type: LinkType) -> Self {
        self.link_type = Some(link_type);
        self
    }

    /// Sets the URL (builder pattern).
    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    /// Sets the external ID (builder pattern).
    pub fn with_external_id(mut self, external_id: String) -> Self {
        self.external_id = Some(external_id);
        self
    }

    /// Sets the relationship (builder pattern).
    pub fn with_rel(mut self, rel: LinkRel) -> Self {
        self.rel = Some(rel);
        self
    }
}

/// Information about a prefix in the issue tracker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrefixInfo {
    /// The prefix string (e.g., "proj", "api").
    pub prefix: String,
    /// Number of issues with this prefix.
    pub issue_count: i64,
    /// When this prefix was first used.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
