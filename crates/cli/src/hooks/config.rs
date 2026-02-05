// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook configuration loading from TOML and JSON files.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

const HOOKS_TOML_FILE: &str = "hooks.toml";
const HOOKS_JSON_FILE: &str = "hooks.json";

/// A single hook definition from configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Identifier for the hook.
    pub name: String,
    /// Event patterns to trigger on (e.g., "issue.created", "issue.*").
    pub events: Vec<String>,
    /// Optional filter string using CLI filter syntax (e.g., "-t bug -l urgent").
    #[serde(default)]
    pub filter: Option<String>,
    /// Command to execute when the hook triggers.
    pub run: String,
}

/// Root configuration structure for hooks.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HooksConfig {
    /// List of configured hooks.
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

/// Load hooks configuration from `.wok/hooks.toml` and/or `.wok/hooks.json`.
///
/// If both files exist, hooks from both are merged. Returns `None` if neither
/// file exists.
pub fn load_hooks_config(work_dir: &Path) -> Result<Option<HooksConfig>> {
    let toml_path = work_dir.join(HOOKS_TOML_FILE);
    let json_path = work_dir.join(HOOKS_JSON_FILE);

    let toml_exists = toml_path.exists();
    let json_exists = json_path.exists();

    if !toml_exists && !json_exists {
        return Ok(None);
    }

    let mut all_hooks = Vec::new();

    // Load TOML config if present
    if toml_exists {
        let content = fs::read_to_string(&toml_path)
            .map_err(|e| Error::Config(format!("failed to read hooks.toml: {}", e)))?;
        let config: HooksConfig = toml::from_str(&content)
            .map_err(|e| Error::Config(format!("failed to parse hooks.toml: {}", e)))?;
        all_hooks.extend(config.hooks);
    }

    // Load JSON config if present
    if json_exists {
        let content = fs::read_to_string(&json_path)
            .map_err(|e| Error::Config(format!("failed to read hooks.json: {}", e)))?;
        let config: HooksConfig = serde_json::from_str(&content)
            .map_err(|e| Error::Config(format!("failed to parse hooks.json: {}", e)))?;
        all_hooks.extend(config.hooks);
    }

    Ok(Some(HooksConfig { hooks: all_hooks }))
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
