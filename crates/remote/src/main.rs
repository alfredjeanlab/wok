// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! wk-remote: WebSocket relay server for distributed wk issue tracking.
//!
//! This server maintains the canonical database state and broadcasts operations
//! to all connected clients. Optionally backs up the oplog to a git repository.

mod git_backing;
mod server;
#[cfg(test)]
mod server_tests;
mod state;

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use git_backing::{GitBacking, GitBackingConfig};

/// wk-remote: Distributed issue tracker relay server
#[derive(Parser, Debug)]
#[command(name = "wk-remote")]
#[command(about = "WebSocket relay server for distributed wk issue tracking")]
struct Args {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:7890")]
    bind: SocketAddr,

    /// Directory for database and oplog storage
    #[arg(short, long, default_value = ".")]
    data: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable git backing for durability
    #[arg(long)]
    git: bool,

    /// Git branch for oplog commits
    #[arg(long, default_value = "wok/oplog")]
    git_branch: String,

    /// Git commit interval in seconds
    #[arg(long, default_value = "90")]
    git_commit_interval: u64,

    /// Git remote name for pushing (enables push)
    #[arg(long)]
    git_remote: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting wk-remote server");
    info!("  Bind address: {}", args.bind);
    info!("  Data directory: {}", args.data.display());

    // Initialize git backing if enabled
    let git_backing = if args.git {
        info!("  Git backing: enabled");
        info!("    Branch: {}", args.git_branch);
        info!("    Commit interval: {}s", args.git_commit_interval);

        let config = GitBackingConfig {
            repo_path: args.data.clone(),
            branch: args.git_branch,
            commit_interval: Duration::from_secs(args.git_commit_interval),
            remote: args.git_remote.clone(),
        };

        if let Some(ref remote) = config.remote {
            info!("    Remote: {}", remote);
        }

        let backing = Arc::new(GitBacking::new(config)?);

        // Start background commit/push tasks
        let _handle = backing.clone().start_background_tasks();

        Some(backing)
    } else {
        None
    };

    // Initialize state
    let state = state::ServerState::new(&args.data, git_backing)?;

    // Run server
    server::run(args.bind, state).await?;

    Ok(())
}
