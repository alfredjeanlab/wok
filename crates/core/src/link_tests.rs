// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use super::*;

#[test]
fn link_type_as_str() {
    assert_eq!(LinkType::Github.as_str(), "github");
    assert_eq!(LinkType::Jira.as_str(), "jira");
    assert_eq!(LinkType::Gitlab.as_str(), "gitlab");
    assert_eq!(LinkType::Confluence.as_str(), "confluence");
}

#[test]
fn link_type_from_str() {
    assert_eq!("github".parse::<LinkType>().unwrap(), LinkType::Github);
    assert_eq!("GITHUB".parse::<LinkType>().unwrap(), LinkType::Github);
    assert_eq!("jira".parse::<LinkType>().unwrap(), LinkType::Jira);
    assert_eq!("gitlab".parse::<LinkType>().unwrap(), LinkType::Gitlab);
    assert_eq!("confluence".parse::<LinkType>().unwrap(), LinkType::Confluence);
    assert!("invalid".parse::<LinkType>().is_err());
}

#[test]
fn link_rel_as_str() {
    assert_eq!(LinkRel::Import.as_str(), "import");
    assert_eq!(LinkRel::Blocks.as_str(), "blocks");
    assert_eq!(LinkRel::Tracks.as_str(), "tracks");
    assert_eq!(LinkRel::TrackedBy.as_str(), "tracked-by");
}

#[test]
fn link_rel_from_str() {
    assert_eq!("import".parse::<LinkRel>().unwrap(), LinkRel::Import);
    assert_eq!("IMPORT".parse::<LinkRel>().unwrap(), LinkRel::Import);
    assert_eq!("blocks".parse::<LinkRel>().unwrap(), LinkRel::Blocks);
    assert_eq!("tracks".parse::<LinkRel>().unwrap(), LinkRel::Tracks);
    assert_eq!("tracked-by".parse::<LinkRel>().unwrap(), LinkRel::TrackedBy);
    assert!("invalid".parse::<LinkRel>().is_err());
}

#[test]
fn link_builder_pattern() {
    let link = Link::new("test-123".to_string())
        .with_type(LinkType::Github)
        .with_url("https://github.com/org/repo/issues/1".to_string())
        .with_external_id("1".to_string())
        .with_rel(LinkRel::Tracks);

    assert_eq!(link.issue_id, "test-123");
    assert_eq!(link.link_type, Some(LinkType::Github));
    assert_eq!(link.url, Some("https://github.com/org/repo/issues/1".to_string()));
    assert_eq!(link.external_id, Some("1".to_string()));
    assert_eq!(link.rel, Some(LinkRel::Tracks));
}
