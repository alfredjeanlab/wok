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
//! - [`Config`] - Project configuration (prefix, workspace location)
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
mod commands;
mod completions;
mod daemon;
mod display;
pub mod filter;
mod git_hooks;
mod mode;
mod normalize;
mod schema;
pub mod timings;
mod validate;
mod wal;
mod worktree;

pub mod config;
pub mod db;
pub mod error;
pub mod id;
pub mod models;
pub mod sync;

pub use cli::{
    Cli, Command, ConfigCommand, HooksCommand, OutputFormat, RemoteCommand, SchemaCommand,
};
pub use config::{find_work_dir, get_db_path, init_work_dir, init_workspace_link, Config};
pub use db::Database;
pub use error::{Error, Result};

use clap::CommandFactory;
use clap_complete::generate;

/// Split label command arguments into (ids, label).
/// The label is always the last argument, with all preceding arguments being IDs.
fn split_ids_and_label(args: &[String]) -> Result<(Vec<String>, String)> {
    if args.len() < 2 {
        return Err(Error::FieldRequired {
            field: "At least one ID and a label",
        });
    }
    let label = args
        .last()
        .ok_or_else(|| Error::FieldRequired { field: "Label" })?
        .clone();
    let ids = args[..args.len() - 1].to_vec();
    Ok((ids, label))
}

/// Execute a CLI command. This is the main entry point for library users
/// and provides a testable way to run commands without process execution.
pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init {
            prefix,
            path,
            workspace,
            remote,
            local,
        } => commands::init::run(prefix, path, workspace, remote, local),
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
        ),
        Command::Start { ids } => commands::lifecycle::start(&ids),
        Command::Done { ids, reason } => commands::lifecycle::done(&ids, reason.as_deref()),
        Command::Close { ids, reason } => commands::lifecycle::close(&ids, reason.as_deref()),
        Command::Reopen { ids, reason } => commands::lifecycle::reopen(&ids, reason.as_deref()),
        Command::Edit { id, attr, value } => commands::edit::run(&id, &attr, &value),
        Command::List {
            status,
            r#type,
            label,
            assignee,
            unassigned,
            filter,
            limit,
            blocked,
            all,
            output,
        } => commands::list::run(
            status, r#type, label, assignee, unassigned, filter, limit, blocked, all, output,
        ),
        Command::Show { id, output } => commands::show::run(&id, &output),
        Command::Tree { id } => commands::tree::run(&id),
        Command::Link { id, url, reason } => commands::link::add(&id, &url, reason),
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
            let (ids, label) = split_ids_and_label(&args)?;
            commands::label::add(&ids, &label)
        }
        Command::Unlabel { args } => {
            let (ids, label) = split_ids_and_label(&args)?;
            commands::label::remove(&ids, &label)
        }
        Command::Note {
            id,
            content,
            replace,
        } => commands::note::run(&id, &content, replace),
        Command::Log { id, limit } => commands::log::run(id, limit),
        Command::Export { filepath } => commands::export::run(&filepath),
        Command::Import {
            file,
            input,
            format,
            dry_run,
            status,
            r#type,
            label,
            prefix,
        } => commands::import::run(file, input, &format, dry_run, status, r#type, label, prefix),
        Command::Ready {
            r#type,
            label,
            assignee,
            unassigned,
            all_assignees,
            output,
        } => commands::ready::run(r#type, label, assignee, unassigned, all_assignees, output),
        Command::Search {
            query,
            status,
            r#type,
            label,
            assignee,
            unassigned,
            filter,
            limit,
            output,
        } => commands::search::run(
            &query, status, r#type, label, assignee, unassigned, filter, limit, output,
        ),
        Command::Completion { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "wk", &mut std::io::stdout());
            Ok(())
        }
        Command::Prime => commands::prime::run(),
        Command::Remote(cmd) => match cmd {
            RemoteCommand::Status => commands::remote::status(),
            RemoteCommand::Sync { force, quiet } => commands::remote::sync(force, quiet),
            RemoteCommand::Stop => commands::remote::stop(),
            RemoteCommand::Run {
                daemon_dir,
                work_dir,
            } => {
                let cfg = config::Config::load(&work_dir)?;
                daemon::run_daemon(&daemon_dir, &cfg)
            }
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
