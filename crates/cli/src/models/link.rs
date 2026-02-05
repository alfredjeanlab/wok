// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use regex::Regex;
use std::sync::LazyLock;

use wk_ipc::LinkType;

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
