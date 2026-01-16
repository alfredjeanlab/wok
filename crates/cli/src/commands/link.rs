// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! External link management command.

use crate::db::links::new_link;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{parse_link_url, Action, Event, LinkRel};

use super::open_db;

/// Add an external link to an issue.
pub fn add(id: &str, url: &str, reason: Option<String>) -> Result<()> {
    let (db, _) = open_db()?;
    add_impl(&db, id, url, reason)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn add_impl(db: &Database, id: &str, url: &str, reason: Option<String>) -> Result<()> {
    // Verify issue exists
    db.get_issue(id)?;

    // Parse URL to detect link type and external ID
    let (link_type, external_id) = parse_link_url(url);

    // Parse relation if provided
    let rel = reason.map(|r| r.parse::<LinkRel>()).transpose()?;

    // Validate import relation requirements
    if rel == Some(LinkRel::Import) {
        if link_type.is_none() {
            return Err(Error::InvalidInput(
                "import requires a known provider type (github, jira, gitlab)".to_string(),
            ));
        }
        if external_id.is_none() {
            return Err(Error::InvalidInput(
                "import requires a detectable issue ID".to_string(),
            ));
        }
    }

    // Create link
    let mut link = new_link(id);
    link.link_type = link_type;
    link.url = Some(url.to_string());
    link.external_id = external_id;
    link.rel = rel;

    db.add_link(&link)?;

    // Log event
    let event = Event::new(id.to_string(), Action::Linked).with_values(None, Some(url.to_string()));
    db.log_event(&event)?;

    println!("Added link to {}", id);
    Ok(())
}

/// Add a link to an issue (for use by new command).
///
/// This is a helper function used by the `new` command to add links
/// during issue creation.
pub(crate) fn add_link_impl(db: &Database, issue_id: &str, url: &str) -> Result<()> {
    let (link_type, external_id) = parse_link_url(url);

    let mut link = new_link(issue_id);
    link.link_type = link_type;
    link.url = Some(url.to_string());
    link.external_id = external_id;

    db.add_link(&link)?;

    // Log event
    let event =
        Event::new(issue_id.to_string(), Action::Linked).with_values(None, Some(url.to_string()));
    db.log_event(&event)?;

    Ok(())
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
