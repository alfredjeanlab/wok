// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Hook configuration loading from .wok/hooks.toml and .wok/hooks.json.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::Result;

/// A single hook definition from configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookConfig {
    /// Unique identifier for this hook.
    pub name: String,
    /// Event patterns this hook responds to (e.g., "issue.created", "issue.*").
    pub events: Vec<String>,
    /// Optional filter string (e.g., "-t bug -l urgent").
    #[serde(default)]
    pub filter: Option<String>,
    /// Command to execute when hook is triggered.
    pub run: String,
}

/// Root configuration structure for hooks files.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HooksConfig {
    /// List of configured hooks.
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
}

impl HooksConfig {
    /// Merge another config into this one.
    pub fn merge(&mut self, other: HooksConfig) {
        self.hooks.extend(other.hooks);
    }
}

/// Load hooks config from .wok/hooks.toml and/or .wok/hooks.json.
///
/// Returns None if neither file exists.
/// If both files exist, hooks from both are merged.
pub fn load_hooks_config(work_dir: &Path) -> Result<Option<HooksConfig>> {
    let toml_path = work_dir.join("hooks.toml");
    let json_path = work_dir.join("hooks.json");

    let toml_exists = toml_path.exists();
    let json_exists = json_path.exists();

    if !toml_exists && !json_exists {
        return Ok(None);
    }

    let mut config = HooksConfig::default();

    if toml_exists {
        let content = fs::read_to_string(&toml_path)?;
        let toml_config: HooksConfig = toml::from_str(&content).map_err(|e| {
            crate::error::Error::Config(format!("failed to parse hooks.toml: {}", e))
        })?;
        config.merge(toml_config);
    }

    if json_exists {
        let content = fs::read_to_string(&json_path)?;
        let json_config: HooksConfig = serde_json::from_str(&content).map_err(|e| {
            crate::error::Error::Config(format!("failed to parse hooks.json: {}", e))
        })?;
        config.merge(json_config);
    }

    if config.hooks.is_empty() {
        Ok(None)
    } else {
        Ok(Some(config))
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
