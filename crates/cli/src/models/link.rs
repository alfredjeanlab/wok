// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Pre-compiled regexes for URL parsing.
// These are compile-time constant patterns that are verified at test time.
// Using match with unreachable! since these patterns are hard-coded and known-valid.
static GITHUB_RE: LazyLock<Regex> =
    LazyLock::new(
        || match Regex::new(r"https://github\.com/[^/]+/[^/]+/issues/(\d+)") {
            Ok(re) => re,
            Err(_) => unreachable!("static regex pattern"),
        },
    );
static GITLAB_RE: LazyLock<Regex> =
    LazyLock::new(
        || match Regex::new(r"https://(?:[^/]+\.)?gitlab\.com/.+/issues/(\d+)") {
            Ok(re) => re,
            Err(_) => unreachable!("static regex pattern"),
        },
    );
static JIRA_RE: LazyLock<Regex> =
    LazyLock::new(
        || match Regex::new(r"https://[^/]+\.atlassian\.net/browse/([A-Z]+-\d+)") {
            Ok(re) => re,
            Err(_) => unreachable!("static regex pattern"),
        },
    );

/// Parse URL and extract link type and issue ID.
///
/// Returns `(link_type, external_id)` where either or both may be None
/// if the URL cannot be recognized or doesn't contain an extractable ID.
///
/// Priority order for URL detection:
/// 1. jira://ID shorthand (explicit)
/// 2. Confluence (must contain /wiki/ in path, before Jira check)
/// 3. GitHub
/// 4. GitLab (supports custom domains with gitlab in name)
/// 5. Jira (atlassian.net/browse/...)
/// 6. Unknown (valid, just no type detection)
pub fn parse_link_url(url: &str) -> (Option<LinkType>, Option<String>) {
    // jira://PE-5555 shorthand
    if let Some(id) = url.strip_prefix("jira://") {
        return (Some(LinkType::Jira), Some(id.to_string()));
    }

    // Confluence: has /wiki/ in path and is atlassian.net (check before Jira)
    if url.contains("/wiki/") && url.contains("atlassian.net") {
        return (Some(LinkType::Confluence), None);
    }

    // GitHub: https://github.com/{owner}/{repo}/issues/{id}
    if let Some(caps) = GITHUB_RE.captures(url) {
        return (
            Some(LinkType::Github),
            caps.get(1).map(|m| m.as_str().to_string()),
        );
    }

    // GitLab: https://(*.)?gitlab.com/{path}/issues/{id}
    if let Some(caps) = GITLAB_RE.captures(url) {
        return (
            Some(LinkType::Gitlab),
            caps.get(1).map(|m| m.as_str().to_string()),
        );
    }

    // Jira: https://*.atlassian.net/browse/{PROJECT-ID}
    if let Some(caps) = JIRA_RE.captures(url) {
        return (
            Some(LinkType::Jira),
            caps.get(1).map(|m| m.as_str().to_string()),
        );
    }

    // Unknown - still valid, just no type/id detection
    (None, None)
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
