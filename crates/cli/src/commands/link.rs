// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! External link management command.

use crate::db::new_link;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{parse_link_url, Action, Event, LinkRel};

use super::{apply_mutation, open_db};

/// Add an external link to an issue.
pub fn add(id: &str, url: &str, reason: Option<String>) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
    add_impl_with_reason(&db, id, url, reason)
}

/// Internal implementation for adding a link with optional reason.
fn add_impl_with_reason(db: &Database, id: &str, url: &str, reason: Option<String>) -> Result<()> {
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;

    // Verify issue exists
    db.get_issue(&resolved_id)?;

    // Parse URL to detect link type and external ID
    let (link_type, external_id) = parse_link_url(url);

    // Parse relation if provided
    let rel = reason.map(|r| r.parse::<LinkRel>()).transpose()?;

    // Validate import relation requirements
    if rel == Some(LinkRel::Import) {
        if link_type.is_none() {
            return Err(Error::LinkRequires {
                requirement: "import",
                dependency: "a known provider type (github, jira, gitlab)",
            });
        }
        if external_id.is_none() {
            return Err(Error::LinkRequires {
                requirement: "import",
                dependency: "a detectable issue ID",
            });
        }
    }

    // Create link
    let mut link = new_link(&resolved_id);
    link.link_type = link_type;
    link.url = Some(url.to_string());
    link.external_id = external_id;
    link.rel = rel;

    db.add_link(&link)?;

    // Log event
    apply_mutation(
        db,
        Event::new(resolved_id.clone(), Action::Linked).with_values(None, Some(url.to_string())),
    )?;

    println!("Added link to {}", resolved_id);
    Ok(())
}

/// Remove an external link from an issue.
pub fn remove(id: &str, url: &str) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
    remove_impl(&db, id, url)
}

/// Internal implementation for removing a link.
fn remove_impl(db: &Database, id: &str, url: &str) -> Result<()> {
    // Resolve potentially partial ID
    let resolved_id = db.resolve_id(id)?;

    // Verify issue exists
    db.get_issue(&resolved_id)?;

    // Find the link by URL
    let links = db.get_links(&resolved_id)?;
    let link = links.iter().find(|l| l.url.as_deref() == Some(url));

    match link {
        Some(link) => {
            db.remove_link(link.id)?;

            // Log event
            apply_mutation(
                db,
                Event::new(resolved_id.clone(), Action::Unlinked)
                    .with_values(Some(url.to_string()), None),
            )?;

            println!("Removed link from {}", resolved_id);
            Ok(())
        }
        None => {
            println!("Link {} not found on {}", url, resolved_id);
            Ok(())
        }
    }
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
    apply_mutation(
        db,
        Event::new(issue_id.to_string(), Action::Linked).with_values(None, Some(url.to_string())),
    )?;

    Ok(())
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
