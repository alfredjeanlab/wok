// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;
use crate::error::Result;
use crate::models::{Action, Event, Relation, UserRelation};

use super::open_db;

pub fn add(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, _) = open_db()?;
    add_impl(&db, from_id, rel, to_ids)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn add_impl(db: &Database, from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    // Verify source issue exists
    db.get_issue(from_id)?;

    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        // Verify target issue exists
        db.get_issue(to_id)?;

        match user_rel {
            UserRelation::Blocks => {
                db.add_dependency(from_id, to_id, Relation::Blocks)?;

                let event = Event::new(from_id.to_string(), Action::Related)
                    .with_values(None, Some(format!("blocks {}", to_id)));
                db.log_event(&event)?;

                println!("{} blocks {}", from_id, to_id);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.add_dependency(to_id, from_id, Relation::Blocks)?;

                let event = Event::new(from_id.to_string(), Action::Related)
                    .with_values(None, Some(format!("blocked by {}", to_id)));
                db.log_event(&event)?;

                println!("{} blocked by {}", from_id, to_id);
            }
            UserRelation::Tracks => {
                // A tracks B means:
                // - A tracks B
                // - B tracked-by A
                db.add_dependency(from_id, to_id, Relation::Tracks)?;
                db.add_dependency(to_id, from_id, Relation::TrackedBy)?;

                let event = Event::new(from_id.to_string(), Action::Related)
                    .with_values(None, Some(format!("tracks {}", to_id)));
                db.log_event(&event)?;

                println!("{} tracks {}", from_id, to_id);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.add_dependency(to_id, from_id, Relation::Tracks)?;
                db.add_dependency(from_id, to_id, Relation::TrackedBy)?;

                let event = Event::new(from_id.to_string(), Action::Related)
                    .with_values(None, Some(format!("tracked by {}", to_id)));
                db.log_event(&event)?;

                println!("{} tracked by {}", from_id, to_id);
            }
        }
    }

    Ok(())
}

pub fn remove(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, _) = open_db()?;
    remove_impl(&db, from_id, rel, to_ids)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn remove_impl(
    db: &Database,
    from_id: &str,
    rel: &str,
    to_ids: &[String],
) -> Result<()> {
    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        match user_rel {
            UserRelation::Blocks => {
                db.remove_dependency(from_id, to_id, Relation::Blocks)?;

                let event = Event::new(from_id.to_string(), Action::Unrelated)
                    .with_values(None, Some(format!("blocks {}", to_id)));
                db.log_event(&event)?;

                println!("Removed: {} blocks {}", from_id, to_id);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.remove_dependency(to_id, from_id, Relation::Blocks)?;

                let event = Event::new(from_id.to_string(), Action::Unrelated)
                    .with_values(None, Some(format!("blocked by {}", to_id)));
                db.log_event(&event)?;

                println!("Removed: {} blocked by {}", from_id, to_id);
            }
            UserRelation::Tracks => {
                db.remove_dependency(from_id, to_id, Relation::Tracks)?;
                db.remove_dependency(to_id, from_id, Relation::TrackedBy)?;

                let event = Event::new(from_id.to_string(), Action::Unrelated)
                    .with_values(None, Some(format!("tracks {}", to_id)));
                db.log_event(&event)?;

                println!("Removed: {} tracks {}", from_id, to_id);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.remove_dependency(to_id, from_id, Relation::Tracks)?;
                db.remove_dependency(from_id, to_id, Relation::TrackedBy)?;

                let event = Event::new(from_id.to_string(), Action::Unrelated)
                    .with_values(None, Some(format!("tracked by {}", to_id)));
                db.log_event(&event)?;

                println!("Removed: {} tracked by {}", from_id, to_id);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "dep_tests.rs"]
mod tests;
