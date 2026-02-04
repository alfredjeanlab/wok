// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Project configuration management.
//!
//! Configuration is stored in `.wok/config.toml` and includes:
//! - `prefix`: The project-specific prefix for issue IDs (e.g., "proj" → "proj-a1b2")
//! - `private`: Whether to use private mode (direct SQLite) vs user-level (daemon)

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
    /// If true, use private mode (direct SQLite at .wok/issues.db, no daemon).
    /// If false (default), use user-level mode (daemon at ~/.local/state/wok/).
    #[serde(default)]
    pub private: bool,
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
            private: false,
        })
    }

    /// Creates a config with private mode enabled.
    pub fn new_private(prefix: String) -> Result<Self> {
        if !validate_prefix(&prefix) {
            return Err(Error::InvalidPrefix);
        }
        Ok(Config {
            prefix,
            private: true,
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
    if config.private {
        // Private mode: database stored in .wok/issues.db
        work_dir.join(DB_FILE_NAME)
    } else {
        // User-level mode: database stored in state directory
        wok_state_dir().join(DB_FILE_NAME)
    }
}

/// Resolve the XDG state directory for wok.
///
/// Precedence:
/// 1. `WOK_STATE_DIR` environment variable
/// 2. `XDG_STATE_HOME/wok`
/// 3. `~/.local/state/wok`
pub fn wok_state_dir() -> PathBuf {
    if let Some(dir) = crate::env::state_dir() {
        return dir;
    }
    if let Some(dir) = crate::env::xdg_state_home() {
        return dir.join("wok");
    }
    dirs::home_dir()
        .map(|h| h.join(".local/state/wok"))
        .unwrap_or_else(|| PathBuf::from(".local/state/wok"))
}

/// Initialize a new .wok directory at the given path
pub fn init_work_dir(path: &Path, prefix: &str) -> Result<PathBuf> {
    let work_dir = path.join(WORK_DIR_NAME);

    if work_dir.join(CONFIG_FILE_NAME).exists() {
        return Err(Error::AlreadyInitialized(work_dir.display().to_string()));
    }

    fs::create_dir_all(&work_dir)?;

    let config = Config::new(prefix.to_string())?;
    config.save(&work_dir)?;

    Ok(work_dir)
}

/// Initialize a new .wok directory in private mode
pub fn init_work_dir_private(path: &Path, prefix: &str) -> Result<PathBuf> {
    let work_dir = path.join(WORK_DIR_NAME);

    if work_dir.join(CONFIG_FILE_NAME).exists() {
        return Err(Error::AlreadyInitialized(work_dir.display().to_string()));
    }

    fs::create_dir_all(&work_dir)?;

    let config = Config::new_private(prefix.to_string())?;
    config.save(&work_dir)?;

    Ok(work_dir)
}

/// Write a .gitignore file to the work directory.
///
/// Private mode: ignores issues.db, config.toml
/// User-level mode: ignores config.toml only (no local db)
pub fn write_gitignore(work_dir: &Path, private: bool) -> Result<()> {
    let gitignore_path = work_dir.join(GITIGNORE_FILE_NAME);

    let content = if private {
        "# Local configuration\nconfig.toml\n\n# Database (private mode)\nissues.db\n"
    } else {
        "# Local configuration\nconfig.toml\n"
    };

    fs::write(&gitignore_path, content)?;
    Ok(())
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
