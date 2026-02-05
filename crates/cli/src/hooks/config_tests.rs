// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_hooks_config_no_files() {
    let tmp = TempDir::new().unwrap();
    let result = load_hooks_config(tmp.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_load_hooks_config_toml() {
    let tmp = TempDir::new().unwrap();
    let toml_content = r#"
[[hooks]]
name = "test-hook"
events = ["issue.created"]
run = "./test.sh"
"#;
    fs::write(tmp.path().join("hooks.toml"), toml_content).unwrap();

    let result = load_hooks_config(tmp.path()).unwrap();
    assert!(result.is_some());
    let config = result.unwrap();
    assert_eq!(config.hooks.len(), 1);
    assert_eq!(config.hooks[0].name, "test-hook");
    assert_eq!(config.hooks[0].events, vec!["issue.created"]);
    assert_eq!(config.hooks[0].run, "./test.sh");
    assert!(config.hooks[0].filter.is_none());
}

#[test]
fn test_load_hooks_config_json() {
    let tmp = TempDir::new().unwrap();
    let json_content = r#"{
        "hooks": [{
            "name": "json-hook",
            "events": ["issue.done", "issue.closed"],
            "filter": "-t bug",
            "run": "./notify.sh"
        }]
    }"#;
    fs::write(tmp.path().join("hooks.json"), json_content).unwrap();

    let result = load_hooks_config(tmp.path()).unwrap();
    assert!(result.is_some());
    let config = result.unwrap();
    assert_eq!(config.hooks.len(), 1);
    assert_eq!(config.hooks[0].name, "json-hook");
    assert_eq!(config.hooks[0].events, vec!["issue.done", "issue.closed"]);
    assert_eq!(config.hooks[0].filter, Some("-t bug".to_string()));
    assert_eq!(config.hooks[0].run, "./notify.sh");
}

#[test]
fn test_load_hooks_config_merge() {
    let tmp = TempDir::new().unwrap();

    let toml_content = r#"
[[hooks]]
name = "toml-hook"
events = ["issue.created"]
run = "./toml.sh"
"#;
    fs::write(tmp.path().join("hooks.toml"), toml_content).unwrap();

    let json_content = r#"{
        "hooks": [{
            "name": "json-hook",
            "events": ["issue.done"],
            "run": "./json.sh"
        }]
    }"#;
    fs::write(tmp.path().join("hooks.json"), json_content).unwrap();

    let result = load_hooks_config(tmp.path()).unwrap();
    assert!(result.is_some());
    let config = result.unwrap();
    assert_eq!(config.hooks.len(), 2);
    assert_eq!(config.hooks[0].name, "toml-hook");
    assert_eq!(config.hooks[1].name, "json-hook");
}

#[test]
fn test_load_hooks_config_empty_hooks() {
    let tmp = TempDir::new().unwrap();
    // Empty hooks array
    let toml_content = "hooks = []\n";
    fs::write(tmp.path().join("hooks.toml"), toml_content).unwrap();

    let result = load_hooks_config(tmp.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_hooks_config_merge() {
    let mut config = HooksConfig::default();
    let other = HooksConfig {
        hooks: vec![HookConfig {
            name: "test".to_string(),
            events: vec!["issue.created".to_string()],
            filter: None,
            run: "./test.sh".to_string(),
        }],
    };

    config.merge(other);
    assert_eq!(config.hooks.len(), 1);
}
