// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Project configuration management.
//!
//! Configuration is stored in `.wok/config.toml` and includes:
//! - `prefix`: The project-specific prefix for issue IDs (e.g., "proj" → "proj-a1b2")
//! - `workspace`: Optional path to store the database in a different location

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::id::validate_prefix;

const WORK_DIR_NAME: &str = ".wok";
const CONFIG_FILE_NAME: &str = "config.toml";
const DB_FILE_NAME: &str = "issues.db";
const GITIGNORE_FILE_NAME: &str = ".gitignore";

/// Project configuration stored in `.wok/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project-specific prefix for issue IDs (2+ lowercase alphanumeric with at least one letter).
    /// Empty when linking to workspace without local prefix.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prefix: String,
    /// Optional path for the database (relative to project root or absolute).
    pub workspace: Option<String>,
    /// Remote sync configuration (optional - if absent, runs in local-only mode).
    pub remote: Option<RemoteConfig>,
}

/// Remote sync configuration.
///
/// Supports three remote types:
/// - Git (same repo): `git:.` - orphan branch in current repo
/// - Git (separate repo): `git:~/tracker` or `git:git@github.com:...`
/// - WebSocket: `ws://...` or `wss://...`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Remote URL. Formats:
    /// - `git:.` - current repo orphan branch
    /// - `git:<path>` - local path to git repo
    /// - `git:<ssh-url>` - SSH URL to git repo (e.g., `git:git@github.com:org/repo.git`)
    /// - `ws://...` or `wss://...` - WebSocket server
    pub url: String,
    /// Branch name for git remotes (default: "wok/oplog").
    #[serde(default = "default_branch")]
    pub branch: String,
    /// If true, store oplog worktree in `.wok/oplog/` instead of XDG data dir.
    /// Only relevant for git remotes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worktree: Option<bool>,
    /// Maximum reconnection attempts before giving up (default: 10).
    /// Only relevant for WebSocket remotes.
    #[serde(default = "default_reconnect_max_retries")]
    pub reconnect_max_retries: u32,
    /// Maximum delay between reconnection attempts in seconds (default: 30).
    /// Only relevant for WebSocket remotes.
    #[serde(default = "default_reconnect_max_delay_secs")]
    pub reconnect_max_delay_secs: u64,
    /// Heartbeat ping interval in milliseconds (default: 30000). 0 = disabled.
    /// Only relevant for WebSocket remotes.
    #[serde(default = "default_heartbeat_interval_ms")]
    pub heartbeat_interval_ms: u64,
    /// Max time to wait for pong response in milliseconds (default: 10000).
    /// Only relevant for WebSocket remotes.
    #[serde(default = "default_heartbeat_timeout_ms")]
    pub heartbeat_timeout_ms: u64,
    /// Max time to wait for initial connection in seconds (default: 5).
    /// Used when starting a new daemon to wait for WebSocket connection.
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
}

fn default_branch() -> String {
    "wok/oplog".to_string()
}

/// The type of remote backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteType {
    /// Git remote (same repo or separate repo).
    Git,
    /// WebSocket server.
    WebSocket,
}

impl RemoteConfig {
    /// Determines the remote type from the URL.
    pub fn remote_type(&self) -> RemoteType {
        if self.url.starts_with("ws://") || self.url.starts_with("wss://") {
            RemoteType::WebSocket
        } else {
            // Everything else is git (git:., git:path, git:ssh-url, or bare ssh)
            RemoteType::Git
        }
    }

    /// Validates that the URL is in a recognized format.
    ///
    /// Valid formats:
    /// - WebSocket: `ws://...` or `wss://...`
    /// - Git same-repo: `git:.` or `.`
    /// - Git separate repo: `git:<path>`, `git:<ssh-url>`, `git@...`, `ssh://...`
    ///
    /// Returns an error message if the URL is invalid.
    pub fn validate_url(&self) -> Option<String> {
        let url = &self.url;

        // WebSocket URLs
        if url.starts_with("ws://") || url.starts_with("wss://") {
            return None; // Valid
        }

        // Git same-repo
        if url == "git:." || url == "." {
            return None; // Valid
        }

        // Git separate repo with git: prefix
        if let Some(path) = url.strip_prefix("git:") {
            if path.is_empty() {
                return Some("git: URL requires a path or SSH URL".to_string());
            }
            return None; // Valid (path or SSH URL after git:)
        }

        // Bare SSH URLs
        if url.starts_with("git@") || url.starts_with("ssh://") {
            return None; // Valid
        }

        // Anything else is invalid
        Some(format!(
            "invalid remote URL '{}': must be ws://, wss://, git:., git:<path>, git@..., or ssh://",
            url
        ))
    }

    /// Returns true if this is a same-repo git remote (git:.).
    pub fn is_same_repo(&self) -> bool {
        self.url == "git:." || self.url == "."
    }

    /// Returns the git URL to use for operations.
    /// For same-repo, returns None (use current repo).
    /// For separate repos, strips the `git:` prefix.
    pub fn git_url(&self) -> Option<&str> {
        if self.is_same_repo() {
            None
        } else if let Some(stripped) = self.url.strip_prefix("git:") {
            Some(stripped)
        } else if self.url.starts_with("git@") || self.url.starts_with("ssh://") {
            Some(&self.url)
        } else {
            // Treat as path
            Some(&self.url)
        }
    }
}

fn default_reconnect_max_retries() -> u32 {
    10
}

fn default_reconnect_max_delay_secs() -> u64 {
    30
}

fn default_heartbeat_interval_ms() -> u64 {
    30_000
}

fn default_heartbeat_timeout_ms() -> u64 {
    10_000
}

fn default_connect_timeout_secs() -> u64 {
    2
}

impl Config {
    /// Creates a new config with the given prefix.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPrefix`] if prefix is not 2+ lowercase alphanumeric with at least one letter.
    pub fn new(prefix: String) -> Result<Self> {
        if !validate_prefix(&prefix) {
            return Err(Error::InvalidPrefix);
        }
        Ok(Config {
            prefix,
            workspace: None,
            remote: None,
        })
    }

    /// Creates a config with workspace and optional prefix.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidPrefix`] if prefix is provided but invalid.
    pub fn new_with_workspace(prefix: Option<String>, workspace: String) -> Result<Self> {
        if let Some(ref p) = prefix {
            if !validate_prefix(p) {
                return Err(Error::InvalidPrefix);
            }
        }
        Ok(Config {
            prefix: prefix.unwrap_or_default(),
            workspace: Some(workspace),
            remote: None,
        })
    }

    /// Loads configuration from the given `.wok/` directory.
    pub fn load(work_dir: &Path) -> Result<Self> {
        let config_path = work_dir.join(CONFIG_FILE_NAME);
        let content = fs::read_to_string(&config_path)
            .map_err(|e| Error::Config(format!("failed to read config: {}", e)))?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Returns true if remote sync is configured.
    pub fn is_remote_mode(&self) -> bool {
        self.remote.is_some()
    }

    /// Returns the remote URL if configured.
    pub fn remote_url(&self) -> Option<&str> {
        self.remote.as_ref().map(|r| r.url.as_str())
    }

    /// Saves configuration to the given `.wok/` directory.
    pub fn save(&self, work_dir: &Path) -> Result<()> {
        let config_path = work_dir.join(CONFIG_FILE_NAME);
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("failed to serialize config: {}", e)))?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

/// Find the .wok directory by walking up from the current directory
pub fn find_work_dir() -> Result<PathBuf> {
    let mut current = std::env::current_dir()?;
    loop {
        let work_dir = current.join(WORK_DIR_NAME);
        if work_dir.is_dir() {
            return Ok(work_dir);
        }
        if !current.pop() {
            return Err(Error::NotInitialized);
        }
    }
}

/// Get the database path from config
pub fn get_db_path(work_dir: &Path, config: &Config) -> PathBuf {
    match &config.workspace {
        Some(workspace) => {
            let workspace_path = Path::new(workspace);
            if workspace_path.is_absolute() {
                workspace_path.join(DB_FILE_NAME)
            } else {
                // Relative to work_dir's parent (the project root)
                work_dir
                    .parent()
                    .unwrap_or(work_dir)
                    .join(workspace)
                    .join(DB_FILE_NAME)
            }
        }
        None => work_dir.join(DB_FILE_NAME),
    }
}

/// Get the directory for daemon files (socket, pid, lock).
/// This is the same directory where the database is stored.
pub fn get_daemon_dir(work_dir: &Path, config: &Config) -> PathBuf {
    match &config.workspace {
        Some(workspace) => {
            let workspace_path = Path::new(workspace);
            if workspace_path.is_absolute() {
                workspace_path.to_path_buf()
            } else {
                // Relative to work_dir's parent (the project root)
                work_dir.parent().unwrap_or(work_dir).join(workspace)
            }
        }
        None => work_dir.to_path_buf(),
    }
}

/// Initialize a new .wok directory at the given path
pub fn init_work_dir(path: &Path, prefix: &str) -> Result<PathBuf> {
    let work_dir = path.join(WORK_DIR_NAME);

    if work_dir.exists() {
        return Err(Error::AlreadyInitialized(work_dir.display().to_string()));
    }

    fs::create_dir_all(&work_dir)?;

    let config = Config::new(prefix.to_string())?;
    config.save(&work_dir)?;

    Ok(work_dir)
}

/// Initialize a workspace-link .wok directory (no database)
pub fn init_workspace_link(path: &Path, workspace: &str, prefix: Option<&str>) -> Result<PathBuf> {
    let work_dir = path.join(WORK_DIR_NAME);

    if work_dir.exists() {
        return Err(Error::AlreadyInitialized(work_dir.display().to_string()));
    }

    // Validate workspace path exists
    let workspace_path = Path::new(workspace);
    let resolved_path = if workspace_path.is_absolute() {
        workspace_path.to_path_buf()
    } else {
        path.join(workspace)
    };
    if !resolved_path.is_dir() {
        return Err(Error::WorkspaceNotFound(workspace.to_string()));
    }

    fs::create_dir_all(&work_dir)?;

    let config = Config::new_with_workspace(prefix.map(String::from), workspace.to_string())?;
    config.save(&work_dir)?;

    Ok(work_dir)
}

/// Write a .gitignore file to the work directory.
///
/// Always ignores `current/` and `issues.db`.
/// In local mode (no remote sync), also ignores `config.toml`.
pub fn write_gitignore(work_dir: &Path, local: bool) -> Result<()> {
    let gitignore_path = work_dir.join(GITIGNORE_FILE_NAME);

    let content = if local {
        "# User-specific runtime state\ncurrent/\n\n# Database (local-only mode)\nissues.db\n\n# Local configuration\nconfig.toml\n"
    } else {
        "# User-specific runtime state\ncurrent/\n\n# Database (synced via remote)\nissues.db\n"
    };

    fs::write(&gitignore_path, content)?;
    Ok(())
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
