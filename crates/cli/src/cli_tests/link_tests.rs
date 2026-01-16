// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;

// Link command examples test
#[test]
fn test_link_examples_cover_all_url_formats() {
    // Verify 'wk link' help has exactly one example for each known URL format
    use clap::CommandFactory;
    let cmd = Cli::command();
    let link_cmd = cmd
        .get_subcommands()
        .find(|c| c.get_name() == "link")
        .expect("link subcommand should exist");

    let after_help = link_cmd
        .get_after_help()
        .expect("link should have after_help")
        .to_string();

    // 5 known URL formats that must each have exactly one example:
    // 1. GitHub: github.com
    // 2. GitLab: gitlab.com
    // 3. JIRA shorthand: jira://
    // 4. JIRA (Atlassian): atlassian.net/browse
    // 5. Confluence: atlassian.net/wiki

    let github_count = after_help.matches("github.com").count();
    let gitlab_count = after_help.matches("gitlab.com").count();
    let jira_shorthand_count = after_help.matches("jira://").count();
    let jira_atlassian_count = after_help.matches("atlassian.net/browse").count();
    let confluence_count = after_help.matches("atlassian.net/wiki").count();

    assert_eq!(github_count, 1, "Expected exactly 1 GitHub example");
    assert_eq!(gitlab_count, 1, "Expected exactly 1 GitLab example");
    assert_eq!(
        jira_shorthand_count, 1,
        "Expected exactly 1 JIRA shorthand example"
    );
    assert_eq!(
        jira_atlassian_count, 1,
        "Expected exactly 1 JIRA (Atlassian) example"
    );
    assert_eq!(confluence_count, 1, "Expected exactly 1 Confluence example");

    // Total should be exactly 5 examples (one per format)
    let total = github_count
        + gitlab_count
        + jira_shorthand_count
        + jira_atlassian_count
        + confluence_count;
    assert_eq!(total, 5, "Expected exactly 5 URL format examples total");
}
