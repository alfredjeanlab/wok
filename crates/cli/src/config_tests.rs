// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use super::*;
use tempfile::TempDir;

#[test]
fn test_init_and_load_config() {
    let temp = TempDir::new().unwrap();
    let work_dir = init_work_dir(temp.path(), "test").unwrap();

    let config = Config::load(&work_dir).unwrap();
    assert_eq!(config.prefix, "test");
    assert!(!config.private);
}

#[test]
fn test_db_path_private_mode() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config::new_private("prj".to_string()).unwrap();
    let db_path = get_db_path(&work_dir, &config);
    assert_eq!(db_path, PathBuf::from("/project/.wok/issues.db"));
}

#[test]
fn test_db_path_user_level_mode() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config::new("prj".to_string()).unwrap();
    let db_path = get_db_path(&work_dir, &config);
    // User-level mode: database stored in state directory
    assert_eq!(db_path, wok_state_dir().join("issues.db"));
}

#[test]
fn test_invalid_prefix() {
    assert!(Config::new("a".to_string()).is_err()); // too short
    assert!(Config::new("AB".to_string()).is_err()); // uppercase
}

#[test]
fn test_already_initialized() {
    let temp = TempDir::new().unwrap();
    init_work_dir(temp.path(), "test").unwrap();

    // Second init should fail
    let result = init_work_dir(temp.path(), "test");
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(e.to_string().contains("already initialized"));
    }
}

#[test]
fn test_init_succeeds_with_empty_wok_dir() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    // Init should succeed when .wok exists but has no config.toml
    let result = init_work_dir(temp.path(), "test");
    assert!(result.is_ok());
    assert!(work_dir.join("config.toml").exists());
}

#[test]
fn test_config_load_missing_file() {
    let temp = TempDir::new().unwrap();
    let result = Config::load(temp.path());
    assert!(result.is_err());
}

#[test]
fn test_config_save_and_reload() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    let config = Config {
        prefix: "myproj".to_string(),
        private: true,
    };
    config.save(&work_dir).unwrap();

    let loaded = Config::load(&work_dir).unwrap();
    assert_eq!(loaded.prefix, "myproj");
    assert!(loaded.private);
}

#[test]
fn test_config_load_invalid_toml() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();
    std::fs::write(work_dir.join("config.toml"), "invalid toml {{{").unwrap();

    let result = Config::load(&work_dir);
    assert!(result.is_err());
}

#[test]
fn test_user_level_mode_by_default() {
    let config = Config::new("proj".to_string()).unwrap();
    assert!(!config.private);
}

#[test]
fn test_private_mode() {
    let config = Config::new_private("proj".to_string()).unwrap();
    assert!(config.private);
}

#[test]
fn test_daemon_dir_user_level() {
    let config = Config::new("prj".to_string()).unwrap();
    let daemon_dir = get_daemon_dir(&config);
    assert_eq!(daemon_dir, wok_state_dir());
}

#[test]
fn test_daemon_dir_private() {
    let config = Config::new_private("prj".to_string()).unwrap();
    let daemon_dir = get_daemon_dir(&config);
    // Private mode returns "." as sane default (daemon not used)
    assert_eq!(daemon_dir, PathBuf::from("."));
}

#[test]
fn test_write_gitignore_private_mode() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    write_gitignore(&work_dir, true).unwrap();

    let gitignore_path = work_dir.join(".gitignore");
    assert!(gitignore_path.exists());

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("issues.db"));
    assert!(content.contains("config.toml"));
}

#[test]
fn test_write_gitignore_user_level_mode() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    write_gitignore(&work_dir, false).unwrap();

    let gitignore_path = work_dir.join(".gitignore");
    assert!(gitignore_path.exists());

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("config.toml"));
}

#[test]
fn test_init_work_dir_private() {
    let temp = TempDir::new().unwrap();
    let work_dir = init_work_dir_private(temp.path(), "test").unwrap();

    let config = Config::load(&work_dir).unwrap();
    assert_eq!(config.prefix, "test");
    assert!(config.private);
}

#[test]
fn test_init_work_dir_private_already_initialized() {
    let temp = TempDir::new().unwrap();
    init_work_dir_private(temp.path(), "test").unwrap();

    let result = init_work_dir_private(temp.path(), "test");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already initialized"));
}

#[test]
fn test_parse_config_from_toml() {
    let toml_content = r#"
prefix = "proj"
private = true
"#;

    let config: Config = toml::from_str(toml_content).unwrap();
    assert_eq!(config.prefix, "proj");
    assert!(config.private);
}

#[test]
fn test_parse_config_defaults() {
    let toml_content = r#"
prefix = "proj"
"#;

    let config: Config = toml::from_str(toml_content).unwrap();
    assert_eq!(config.prefix, "proj");
    assert!(!config.private); // default
}

#[test]
fn test_config_serialization() {
    let config = Config::new("myproj".to_string()).unwrap();
    let toml = toml::to_string(&config).unwrap();
    assert!(
        toml.contains("prefix = \"myproj\""),
        "Serialized TOML should contain prefix"
    );
}
