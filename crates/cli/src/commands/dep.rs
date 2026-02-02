// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use crate::db::Database;

use super::{apply_mutation, open_db};
use crate::error::Result;
use crate::models::{Action, Event, Relation, UserRelation};

pub fn add(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
    add_impl(&db, from_id, rel, to_ids)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn add_impl(db: &Database, from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    // Resolve and verify source issue exists (fail fast on ambiguity)
    let resolved_from = db.resolve_id(from_id)?;
    db.get_issue(&resolved_from)?;

    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        // Resolve and verify target issue exists
        let resolved_to = db.resolve_id(to_id)?;
        db.get_issue(&resolved_to)?;

        match user_rel {
            UserRelation::Blocks => {
                db.add_dependency(&resolved_from, &resolved_to, Relation::Blocks)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Related)
                        .with_values(None, Some(format!("blocks {}", resolved_to))),
                )?;

                println!("{} blocks {}", resolved_from, resolved_to);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.add_dependency(&resolved_to, &resolved_from, Relation::Blocks)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Related)
                        .with_values(None, Some(format!("blocked by {}", resolved_to))),
                )?;

                println!("{} blocked by {}", resolved_from, resolved_to);
            }
            UserRelation::Tracks => {
                // A tracks B means:
                // - A tracks B
                // - B tracked-by A
                db.add_dependency(&resolved_from, &resolved_to, Relation::Tracks)?;
                db.add_dependency(&resolved_to, &resolved_from, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Related)
                        .with_values(None, Some(format!("tracks {}", resolved_to))),
                )?;

                println!("{} tracks {}", resolved_from, resolved_to);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.add_dependency(&resolved_to, &resolved_from, Relation::Tracks)?;
                db.add_dependency(&resolved_from, &resolved_to, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Related)
                        .with_values(None, Some(format!("tracked by {}", resolved_to))),
                )?;

                println!("{} tracked by {}", resolved_from, resolved_to);
            }
        }
    }

    Ok(())
}

pub fn remove(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, _config, _work_dir) = open_db()?;
    remove_impl(&db, from_id, rel, to_ids)
}

/// Internal implementation that accepts db for testing.
pub(crate) fn remove_impl(
    db: &Database,
    from_id: &str,
    rel: &str,
    to_ids: &[String],
) -> Result<()> {
    // Resolve source ID (fail fast on ambiguity)
    let resolved_from = db.resolve_id(from_id)?;

    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        // Resolve target ID
        let resolved_to = db.resolve_id(to_id)?;

        match user_rel {
            UserRelation::Blocks => {
                db.remove_dependency(&resolved_from, &resolved_to, Relation::Blocks)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Unrelated)
                        .with_values(None, Some(format!("blocks {}", resolved_to))),
                )?;

                println!("Removed: {} blocks {}", resolved_from, resolved_to);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.remove_dependency(&resolved_to, &resolved_from, Relation::Blocks)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Unrelated)
                        .with_values(None, Some(format!("blocked by {}", resolved_to))),
                )?;

                println!("Removed: {} blocked by {}", resolved_from, resolved_to);
            }
            UserRelation::Tracks => {
                db.remove_dependency(&resolved_from, &resolved_to, Relation::Tracks)?;
                db.remove_dependency(&resolved_to, &resolved_from, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Unrelated)
                        .with_values(None, Some(format!("tracks {}", resolved_to))),
                )?;

                println!("Removed: {} tracks {}", resolved_from, resolved_to);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.remove_dependency(&resolved_to, &resolved_from, Relation::Tracks)?;
                db.remove_dependency(&resolved_from, &resolved_to, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    Event::new(resolved_from.clone(), Action::Unrelated)
                        .with_values(None, Some(format!("tracked by {}", resolved_to))),
                )?;

                println!("Removed: {} tracked by {}", resolved_from, resolved_to);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "dep_tests.rs"]
mod tests;
