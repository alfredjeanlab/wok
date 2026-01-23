// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::path::Path;

use wk_core::OpPayload;

use crate::config::Config;
use crate::db::Database;

use super::{apply_mutation, open_db, queue_op};
use crate::error::Result;
use crate::models::{Action, Event, Relation, UserRelation};

pub fn add(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    add_impl(&db, &config, &work_dir, from_id, rel, to_ids)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn add_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    from_id: &str,
    rel: &str,
    to_ids: &[String],
) -> Result<()> {
    // Verify source issue exists
    db.get_issue(from_id)?;

    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        // Verify target issue exists
        db.get_issue(to_id)?;

        match user_rel {
            UserRelation::Blocks => {
                db.add_dependency(from_id, to_id, Relation::Blocks)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Related)
                        .with_values(None, Some(format!("blocks {}", to_id))),
                    Some(OpPayload::add_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::Blocks,
                    )),
                )?;

                println!("{} blocks {}", from_id, to_id);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.add_dependency(to_id, from_id, Relation::Blocks)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Related)
                        .with_values(None, Some(format!("blocked by {}", to_id))),
                    Some(OpPayload::add_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::Blocks,
                    )),
                )?;

                println!("{} blocked by {}", from_id, to_id);
            }
            UserRelation::Tracks => {
                // A tracks B means:
                // - A tracks B
                // - B tracked-by A
                db.add_dependency(from_id, to_id, Relation::Tracks)?;
                db.add_dependency(to_id, from_id, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Related)
                        .with_values(None, Some(format!("tracks {}", to_id))),
                    Some(OpPayload::add_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::Tracks,
                    )),
                )?;
                // Second queue_op for reverse direction
                queue_op(
                    work_dir,
                    config,
                    OpPayload::add_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::TrackedBy,
                    ),
                )?;

                println!("{} tracks {}", from_id, to_id);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.add_dependency(to_id, from_id, Relation::Tracks)?;
                db.add_dependency(from_id, to_id, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Related)
                        .with_values(None, Some(format!("tracked by {}", to_id))),
                    Some(OpPayload::add_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::Tracks,
                    )),
                )?;
                // Second queue_op for reverse direction
                queue_op(
                    work_dir,
                    config,
                    OpPayload::add_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::TrackedBy,
                    ),
                )?;

                println!("{} tracked by {}", from_id, to_id);
            }
        }
    }

    Ok(())
}

pub fn remove(from_id: &str, rel: &str, to_ids: &[String]) -> Result<()> {
    let (db, config, work_dir) = open_db()?;
    remove_impl(&db, &config, &work_dir, from_id, rel, to_ids)
}

/// Internal implementation that accepts db/config for testing.
pub(crate) fn remove_impl(
    db: &Database,
    config: &Config,
    work_dir: &Path,
    from_id: &str,
    rel: &str,
    to_ids: &[String],
) -> Result<()> {
    let user_rel: UserRelation = rel.parse()?;

    for to_id in to_ids {
        match user_rel {
            UserRelation::Blocks => {
                db.remove_dependency(from_id, to_id, Relation::Blocks)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Unrelated)
                        .with_values(None, Some(format!("blocks {}", to_id))),
                    Some(OpPayload::remove_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::Blocks,
                    )),
                )?;

                println!("Removed: {} blocks {}", from_id, to_id);
            }
            UserRelation::BlockedBy => {
                // "A blocked by B" means "B blocks A"
                db.remove_dependency(to_id, from_id, Relation::Blocks)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Unrelated)
                        .with_values(None, Some(format!("blocked by {}", to_id))),
                    Some(OpPayload::remove_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::Blocks,
                    )),
                )?;

                println!("Removed: {} blocked by {}", from_id, to_id);
            }
            UserRelation::Tracks => {
                db.remove_dependency(from_id, to_id, Relation::Tracks)?;
                db.remove_dependency(to_id, from_id, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Unrelated)
                        .with_values(None, Some(format!("tracks {}", to_id))),
                    Some(OpPayload::remove_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::Tracks,
                    )),
                )?;
                // Second queue_op for reverse direction
                queue_op(
                    work_dir,
                    config,
                    OpPayload::remove_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::TrackedBy,
                    ),
                )?;

                println!("Removed: {} tracks {}", from_id, to_id);
            }
            UserRelation::TrackedBy => {
                // "A tracked by B" means "B tracks A"
                db.remove_dependency(to_id, from_id, Relation::Tracks)?;
                db.remove_dependency(from_id, to_id, Relation::TrackedBy)?;

                apply_mutation(
                    db,
                    work_dir,
                    config,
                    Event::new(from_id.to_string(), Action::Unrelated)
                        .with_values(None, Some(format!("tracked by {}", to_id))),
                    Some(OpPayload::remove_dep(
                        to_id.to_string(),
                        from_id.to_string(),
                        wk_core::issue::Relation::Tracks,
                    )),
                )?;
                // Second queue_op for reverse direction
                queue_op(
                    work_dir,
                    config,
                    OpPayload::remove_dep(
                        from_id.to_string(),
                        to_id.to_string(),
                        wk_core::issue::Relation::TrackedBy,
                    ),
                )?;

                println!("Removed: {} tracked by {}", from_id, to_id);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "dep_tests.rs"]
mod tests;
