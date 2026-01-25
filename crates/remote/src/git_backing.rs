// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Git backing for server durability.
//!
//! Periodically commits the oplog to a git repository for durability
//! and optional push to a remote for disaster recovery.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Configuration for git backing.
#[derive(Debug, Clone)]
pub struct GitBackingConfig {
    /// Path to the git repository (data directory).
    pub repo_path: PathBuf,
    /// Branch name for commits.
    pub branch: String,
    /// Commit interval (how often to commit pending changes).
    pub commit_interval: Duration,
    /// Remote name for pushing (None = no push).
    pub remote: Option<String>,
}

impl Default for GitBackingConfig {
    fn default() -> Self {
        GitBackingConfig {
            repo_path: PathBuf::from("."),
            branch: "wok/oplog".to_string(),
            commit_interval: Duration::from_secs(90), // 90 seconds
            remote: None,
        }
    }
}

/// Git backing state.
pub struct GitBacking {
    config: GitBackingConfig,
    /// Tracks whether there are uncommitted changes.
    has_uncommitted: Arc<Mutex<bool>>,
}

impl GitBacking {
    /// Creates a new git backing instance.
    ///
    /// Initializes the git repository if it doesn't exist.
    pub fn new(config: GitBackingConfig) -> std::io::Result<Self> {
        let backing = GitBacking {
            config,
            has_uncommitted: Arc::new(Mutex::new(false)),
        };

        // Initialize git repo if needed
        backing.init_repo()?;

        Ok(backing)
    }

    /// Initializes the git repository.
    fn init_repo(&self) -> std::io::Result<()> {
        let repo_path = &self.config.repo_path;

        // Check if .git exists
        if !repo_path.join(".git").exists() {
            info!("Initializing git repository at {}", repo_path.display());

            // git init
            let output = Command::new("git")
                .current_dir(repo_path)
                .args(["init"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(std::io::Error::other(format!(
                    "git init failed: {}",
                    stderr
                )));
            }

            // Create orphan branch for oplog
            let output = Command::new("git")
                .current_dir(repo_path)
                .args(["checkout", "--orphan", &self.config.branch])
                .output()?;

            if !output.status.success() {
                // Branch might already exist, try regular checkout
                let _ = Command::new("git")
                    .current_dir(repo_path)
                    .args(["checkout", &self.config.branch])
                    .output();
            }

            // Initial commit if oplog exists
            if repo_path.join("oplog.jsonl").exists() {
                let _ = self.commit("Initialize oplog");
            }
        }

        Ok(())
    }

    /// Marks that there are changes to commit.
    pub async fn mark_dirty(&self) {
        let mut has_uncommitted = self.has_uncommitted.lock().await;
        *has_uncommitted = true;
    }

    /// Commits pending changes if any.
    ///
    /// Returns Ok(true) if a commit was made, Ok(false) if nothing to commit.
    pub async fn commit_if_dirty(&self) -> std::io::Result<bool> {
        let mut has_uncommitted = self.has_uncommitted.lock().await;
        if !*has_uncommitted {
            return Ok(false);
        }

        match self.commit("wk-remote sync") {
            Ok(committed) => {
                if committed {
                    *has_uncommitted = false;
                }
                Ok(committed)
            }
            Err(e) => {
                warn!("Git commit failed: {}", e);
                Err(e)
            }
        }
    }

    /// Commits changes to git.
    fn commit(&self, message: &str) -> std::io::Result<bool> {
        let repo_path = &self.config.repo_path;

        // Only commit the oplog (not the sqlite database)
        if repo_path.join("oplog.jsonl").exists() {
            let output = Command::new("git")
                .current_dir(repo_path)
                .args(["add", "oplog.jsonl"])
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                debug!("git add oplog.jsonl output: {}", stderr);
            }
        }

        // Check if there are changes to commit
        let status_output = Command::new("git")
            .current_dir(repo_path)
            .args(["status", "--porcelain"])
            .output()?;

        if status_output.stdout.is_empty() {
            debug!("Nothing to commit");
            return Ok(false);
        }

        // git commit
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["commit", "-m", message])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // "nothing to commit" is not an error (can appear in either stream)
            if stderr.contains("nothing to commit") || stdout.contains("nothing to commit") {
                return Ok(false);
            }
            return Err(std::io::Error::other(format!(
                "git commit failed: stderr={} stdout={}",
                stderr, stdout
            )));
        }

        info!("Committed oplog changes");
        Ok(true)
    }

    /// Pushes to remote if configured.
    pub async fn push_if_configured(&self) -> std::io::Result<bool> {
        let remote = match &self.config.remote {
            Some(r) => r,
            None => return Ok(false),
        };

        self.push(remote).await
    }

    /// Pushes to the specified remote.
    async fn push(&self, remote: &str) -> std::io::Result<bool> {
        let repo_path = &self.config.repo_path;
        let branch = &self.config.branch;

        info!("Pushing to remote {} branch {}", remote, branch);

        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["push", remote, branch])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // "Everything up-to-date" is not an error
            if stderr.contains("Everything up-to-date") {
                return Ok(false);
            }
            return Err(std::io::Error::other(format!(
                "git push failed: {}",
                stderr
            )));
        }

        info!("Pushed to remote");
        Ok(true)
    }

    /// Starts the background commit/push tasks.
    ///
    /// Returns a join handle for the commit task.
    pub fn start_background_tasks(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        let backing = Arc::clone(&self);
        let commit_interval = self.config.commit_interval;

        tokio::spawn(async move {
            let mut interval = interval(commit_interval);

            loop {
                interval.tick().await;

                if let Err(e) = backing.commit_if_dirty().await {
                    error!("Background commit failed: {}", e);
                }

                if let Err(e) = backing.push_if_configured().await {
                    error!("Background push failed: {}", e);
                }
            }
        })
    }
}

#[cfg(test)]
#[path = "git_backing_tests.rs"]
mod tests;
