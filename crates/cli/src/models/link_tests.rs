// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use wk_ipc::{LinkRel, LinkType};
use yare::parameterized;

// LinkType tests
#[parameterized(
    github = { LinkType::Github, "github" },
    jira = { LinkType::Jira, "jira" },
    gitlab = { LinkType::Gitlab, "gitlab" },
    confluence = { LinkType::Confluence, "confluence" },
)]
fn test_link_type_roundtrip(link_type: LinkType, expected: &str) {
    assert_eq!(link_type.as_str(), expected);
    assert_eq!(link_type.to_string(), expected);
    assert_eq!(expected.parse::<LinkType>().unwrap(), link_type);
}

#[parameterized(
    github_upper = { "GITHUB", LinkType::Github },
    jira_mixed = { "Jira", LinkType::Jira },
)]
fn test_link_type_from_str_case_insensitive(input: &str, expected: LinkType) {
    assert_eq!(input.parse::<LinkType>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    unknown = { "unknown" },
)]
fn test_link_type_from_str_invalid(input: &str) {
    assert!(input.parse::<LinkType>().is_err());
}

// LinkRel tests
#[parameterized(
    import = { LinkRel::Import, "import" },
    blocks = { LinkRel::Blocks, "blocks" },
    tracks = { LinkRel::Tracks, "tracks" },
    tracked_by = { LinkRel::TrackedBy, "tracked-by" },
)]
fn test_link_rel_roundtrip(link_rel: LinkRel, expected: &str) {
    assert_eq!(link_rel.as_str(), expected);
    assert_eq!(link_rel.to_string(), expected);
    assert_eq!(expected.parse::<LinkRel>().unwrap(), link_rel);
}

#[parameterized(
    import_upper = { "IMPORT", LinkRel::Import },
    blocks_mixed = { "Blocks", LinkRel::Blocks },
    tracked_by_mixed = { "Tracked-By", LinkRel::TrackedBy },
)]
fn test_link_rel_from_str_case_insensitive(input: &str, expected: LinkRel) {
    assert_eq!(input.parse::<LinkRel>().unwrap(), expected);
}

#[parameterized(
    invalid = { "invalid" },
    empty = { "" },
    relates = { "relates" },
)]
fn test_link_rel_from_str_invalid(input: &str) {
    assert!(input.parse::<LinkRel>().is_err());
}

// parse_link_url tests

#[test]
fn test_parse_github_url() {
    let url = "https://github.com/org/repo/issues/123";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Github));
    assert_eq!(external_id, Some("123".to_string()));
}

#[test]
fn test_parse_github_url_large_number() {
    let url = "https://github.com/anthropics/claude/issues/999999";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Github));
    assert_eq!(external_id, Some("999999".to_string()));
}

#[test]
fn test_parse_jira_atlassian_url() {
    let url = "https://company.atlassian.net/browse/PE-5555";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Jira));
    assert_eq!(external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_parse_jira_shorthand() {
    let url = "jira://PE-5555";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Jira));
    assert_eq!(external_id, Some("PE-5555".to_string()));
}

#[test]
fn test_parse_jira_shorthand_any_project() {
    let url = "jira://PROJ-123";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Jira));
    assert_eq!(external_id, Some("PROJ-123".to_string()));
}

#[test]
fn test_parse_gitlab_url() {
    let url = "https://gitlab.com/org/project/issues/456";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Gitlab));
    assert_eq!(external_id, Some("456".to_string()));
}

#[test]
fn test_parse_gitlab_nested_group() {
    let url = "https://gitlab.com/org/subgroup/project/issues/789";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Gitlab));
    assert_eq!(external_id, Some("789".to_string()));
}

#[test]
fn test_parse_confluence_url() {
    let url = "https://company.atlassian.net/wiki/spaces/DOC/pages/123";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Confluence));
    // Confluence pages don't have extractable issue IDs
    assert_eq!(external_id, None);
}

#[test]
fn test_confluence_before_jira() {
    // Confluence URLs contain atlassian.net but should NOT be detected as Jira
    let url = "https://company.atlassian.net/wiki/spaces/TEAM/pages/456789";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, Some(LinkType::Confluence));
    assert_ne!(link_type, Some(LinkType::Jira));
    assert_eq!(external_id, None);
}

#[test]
fn test_parse_unknown_url() {
    let url = "https://example.com/issue/123";
    let (link_type, external_id) = parse_link_url(url);
    assert_eq!(link_type, None);
    assert_eq!(external_id, None);
}

#[test]
fn test_parse_empty_string() {
    let (link_type, external_id) = parse_link_url("");
    assert_eq!(link_type, None);
    assert_eq!(external_id, None);
}

// Link serialization tests
#[test]
fn test_link_type_serde() {
    let json = serde_json::to_string(&LinkType::Github).unwrap();
    assert_eq!(json, "\"github\"");

    let parsed: LinkType = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, LinkType::Github);
}

#[test]
fn test_link_rel_serde() {
    let json = serde_json::to_string(&LinkRel::TrackedBy).unwrap();
    assert_eq!(json, "\"tracked-by\"");

    let parsed: LinkRel = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, LinkRel::TrackedBy);
}
