// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]

use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn load_returns_none_when_no_config() {
    let dir = TempDir::new().unwrap();
    let result = load_hooks_config(dir.path()).unwrap();
    assert!(result.is_none());
}

#[test]
fn load_toml_config() {
    let dir = TempDir::new().unwrap();
    let config_content = r#"
[[hooks]]
name = "test-hook"
events = ["issue.created"]
run = "./test.sh"
"#;
    fs::write(dir.path().join("hooks.toml"), config_content).unwrap();

    let result = load_hooks_config(dir.path()).unwrap().unwrap();
    assert_eq!(result.hooks.len(), 1);
    assert_eq!(result.hooks[0].name, "test-hook");
    assert_eq!(result.hooks[0].events, vec!["issue.created"]);
    assert_eq!(result.hooks[0].run, "./test.sh");
    assert!(result.hooks[0].filter.is_none());
}

#[test]
fn load_json_config() {
    let dir = TempDir::new().unwrap();
    let config_content = r#"{
        "hooks": [{
            "name": "json-hook",
            "events": ["issue.done"],
            "filter": "-t bug",
            "run": "./notify.sh"
        }]
    }"#;
    fs::write(dir.path().join("hooks.json"), config_content).unwrap();

    let result = load_hooks_config(dir.path()).unwrap().unwrap();
    assert_eq!(result.hooks.len(), 1);
    assert_eq!(result.hooks[0].name, "json-hook");
    assert_eq!(result.hooks[0].filter, Some("-t bug".to_string()));
}

#[test]
fn load_merges_toml_and_json() {
    let dir = TempDir::new().unwrap();

    let toml_content = r#"
[[hooks]]
name = "toml-hook"
events = ["issue.created"]
run = "./toml.sh"
"#;
    fs::write(dir.path().join("hooks.toml"), toml_content).unwrap();

    let json_content =
        r#"{"hooks": [{"name": "json-hook", "events": ["issue.done"], "run": "./json.sh"}]}"#;
    fs::write(dir.path().join("hooks.json"), json_content).unwrap();

    let result = load_hooks_config(dir.path()).unwrap().unwrap();
    assert_eq!(result.hooks.len(), 2);

    let names: Vec<_> = result.hooks.iter().map(|h| &h.name).collect();
    assert!(names.contains(&&"toml-hook".to_string()));
    assert!(names.contains(&&"json-hook".to_string()));
}

#[test]
fn load_toml_with_filter() {
    let dir = TempDir::new().unwrap();
    let config_content = r#"
[[hooks]]
name = "filtered"
events = ["issue.created"]
filter = "-t bug -l urgent"
run = "./alert.sh"
"#;
    fs::write(dir.path().join("hooks.toml"), config_content).unwrap();

    let result = load_hooks_config(dir.path()).unwrap().unwrap();
    assert_eq!(result.hooks[0].filter, Some("-t bug -l urgent".to_string()));
}

#[test]
fn load_wildcard_event() {
    let dir = TempDir::new().unwrap();
    let config_content = r#"
[[hooks]]
name = "all-events"
events = ["issue.*"]
run = "./audit.sh"
"#;
    fs::write(dir.path().join("hooks.toml"), config_content).unwrap();

    let result = load_hooks_config(dir.path()).unwrap().unwrap();
    assert_eq!(result.hooks[0].events, vec!["issue.*"]);
}

#[test]
fn load_invalid_toml_returns_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hooks.toml"), "invalid toml {{{{").unwrap();

    let result = load_hooks_config(dir.path());
    assert!(result.is_err());
}

#[test]
fn load_invalid_json_returns_error() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("hooks.json"), "{invalid json").unwrap();

    let result = load_hooks_config(dir.path());
    assert!(result.is_err());
}
