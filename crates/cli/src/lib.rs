// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! wkrs - A lightweight, git-friendly issue tracker library.
//!
//! This crate provides the core functionality for the `wk` CLI tool,
//! a local issue tracker that stores data in a SQLite database.
//!
//! # Main Components
//!
//! - [`Database`] - SQLite-backed storage for issues, events, notes, and dependencies
//! - [`Config`] - Project configuration (prefix, private mode)
//! - [`models`] - Core data types ([`Issue`](models::Issue), [`Event`](models::Event), etc.)
//! - [`Error`] - Error types for all operations
//!
//! # Initialization
//!
//! Use [`init_work_dir`] to create a new `.work/` directory, then open the database:
//!
//! ```rust,ignore
//! use wkrs::{init_work_dir, find_work_dir, get_db_path, Config, Database};
//!
//! // Initialize a new project
//! let work_dir = init_work_dir(Path::new("."), "proj")?;
//!
//! // Later, find and open an existing project
//! let work_dir = find_work_dir()?;
//! let config = Config::load(&work_dir)?;
//! let db_path = get_db_path(&work_dir, &config);
//! let db = Database::open(&db_path)?;
//! ```

mod cli;
pub mod colors;
mod commands;
mod completions;
pub mod daemon;
mod display;
pub mod env;
pub mod filter;
pub mod help;
mod normalize;
mod schema;
pub mod timings;
mod validate;

pub mod config;
pub mod db;
pub mod error;
pub mod id;
pub mod models;

pub use cli::{
    AssigneeArgs, Cli, Command, ConfigCommand, DaemonCommand, HooksCommand, LimitArgs,
    OutputFormat, SchemaCommand, TypeLabelArgs,
};
pub use config::{find_work_dir, get_db_path, init_work_dir, Config};
pub use db::Database;
pub use error::{Error, Result};

use clap::CommandFactory;
use clap_complete::generate;

/// Split label command arguments into (ids, labels) by trying to resolve each argument as an issue ID.
/// Once an argument fails to resolve as an issue ID, treat it and all subsequent arguments as labels.
fn split_ids_and_labels(db: &Database, args: &[String]) -> Result<(Vec<String>, Vec<String>)> {
    if args.len() < 2 {
        return Err(Error::FieldRequired {
            field: "At least one ID and a label",
        });
    }

    let mut ids = Vec::new();
    let mut labels_start = args.len();

    for (i, arg) in args.iter().enumerate() {
        match db.resolve_id(arg) {
            Ok(resolved_id) => ids.push(resolved_id),
            Err(_) => {
                // This arg doesn't resolve to an issue ID, treat it and rest as labels
                labels_start = i;
                break;
            }
        }
    }

    let labels: Vec<String> = args[labels_start..].to_vec();

    if ids.is_empty() {
        return Err(Error::FieldRequired {
            field: "At least one valid issue ID",
        });
    }
    if labels.is_empty() {
        return Err(Error::FieldRequired {
            field: "At least one label",
        });
    }

    Ok((ids, labels))
}

/// Execute a CLI command. This is the main entry point for library users
/// and provides a testable way to run commands without process execution.
pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init {
            prefix,
            path,
            private,
        } => commands::init::run(prefix, path, private),
        Command::New {
            type_or_title,
            title,
            label,
            note,
            link,
            assignee,
            priority,
            description,
            blocks,
            blocked_by,
            tracks,
            tracked_by,
            output,
            prefix,
        } => commands::new::run(
            type_or_title,
            title,
            label,
            note,
            link,
            assignee,
            priority,
            description,
            blocks,
            blocked_by,
            tracks,
            tracked_by,
            output,
            prefix,
        ),
        Command::Start { ids } => commands::lifecycle::start(&ids),
        Command::Done { ids, reason } => commands::lifecycle::done(&ids, reason.as_deref()),
        Command::Close { ids, reason } => commands::lifecycle::close(&ids, reason.as_deref()),
        Command::Reopen { ids, reason } => commands::lifecycle::reopen(&ids, reason.as_deref()),
        Command::Edit {
            id,
            attr,
            value,
            flag_title,
            flag_description,
            flag_type,
            flag_assignee,
        } => {
            let (resolved_attr, resolved_value) = if let Some(v) = flag_title {
                ("title".to_string(), v)
            } else if let Some(v) = flag_description {
                ("description".to_string(), v)
            } else if let Some(v) = flag_type {
                ("type".to_string(), v)
            } else if let Some(v) = flag_assignee {
                ("assignee".to_string(), v)
            } else if let (Some(a), Some(v)) = (attr, value) {
                (a, v)
            } else {
                return Err(Error::FieldRequired {
                    field: "attribute and value",
                });
            };
            commands::edit::run(&id, &resolved_attr, &resolved_value)
        }
        Command::List {
            status,
            type_label,
            assignee_args,
            filter,
            limits,
            blocked,
            all,
            output,
        } => commands::list::run(
            status,
            type_label.r#type,
            type_label.label,
            type_label.prefix,
            assignee_args.assignee,
            assignee_args.unassigned,
            filter,
            limits.limit,
            limits.no_limit,
            blocked,
            all,
            output,
        ),
        Command::Show { ids, output } => commands::show::run(&ids, &output),
        Command::Tree { ids } => commands::tree::run(&ids),
        Command::Link { id, url, reason } => commands::link::add(&id, &url, reason),
        Command::Unlink { id, url } => commands::link::remove(&id, &url),
        Command::Dep {
            from_id,
            rel,
            to_ids,
        } => commands::dep::add(&from_id, &rel, &to_ids),
        Command::Undep {
            from_id,
            rel,
            to_ids,
        } => commands::dep::remove(&from_id, &rel, &to_ids),
        Command::Label { args } => {
            let (db, _config, _work_dir) = commands::open_db()?;
            let (ids, labels) = split_ids_and_labels(&db, &args)?;
            commands::label::add_with_db(&db, &ids, &labels)
        }
        Command::Unlabel { args } => {
            let (db, _config, _work_dir) = commands::open_db()?;
            let (ids, labels) = split_ids_and_labels(&db, &args)?;
            commands::label::remove_with_db(&db, &ids, &labels)
        }
        Command::Note {
            id,
            content,
            replace,
        } => commands::note::run(&id, &content, replace),
        Command::Log { id, limits } => commands::log::run(id, limits.limit, limits.no_limit),
        Command::Export { filepath } => commands::export::run(&filepath),
        Command::Import {
            file,
            input,
            format,
            dry_run,
            status,
            type_label,
        } => commands::import::run(
            file,
            input,
            &format,
            dry_run,
            status,
            type_label.r#type,
            type_label.label,
            type_label.prefix,
        ),
        Command::Ready {
            type_label,
            assignee,
            unassigned,
            all_assignees,
            output,
        } => commands::ready::run(
            type_label.r#type,
            type_label.label,
            type_label.prefix,
            assignee,
            unassigned,
            all_assignees,
            output,
        ),
        Command::Search {
            query,
            status,
            type_label,
            assignee_args,
            filter,
            limits,
            output,
        } => commands::search::run(
            &query,
            status,
            type_label.r#type,
            type_label.label,
            type_label.prefix,
            assignee_args.assignee,
            assignee_args.unassigned,
            filter,
            limits.limit,
            limits.no_limit,
            output,
        ),
        Command::Completion { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "wok", &mut std::io::stdout());
            Ok(())
        }
        Command::Prime => commands::prime::run(),
        Command::Daemon(cmd) => match cmd {
            DaemonCommand::Status => commands::daemon::status(),
            DaemonCommand::Stop => commands::daemon::stop(),
            DaemonCommand::Start { foreground } => commands::daemon::start(foreground),
            DaemonCommand::Logs { follow } => commands::daemon::logs(follow),
        },
        Command::Hooks(cmd) => match cmd {
            HooksCommand::Install {
                scope,
                interactive,
                yes,
            } => commands::hooks::install(scope, interactive, yes),
            HooksCommand::Uninstall { scope } => commands::hooks::uninstall(scope),
            HooksCommand::Status => commands::hooks::status(),
        },
        Command::Config(cmd) => commands::config::run(cmd),
        Command::Schema(cmd) => commands::schema::run(cmd),
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
