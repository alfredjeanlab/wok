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
    assert!(config.workspace.is_none());
}

#[test]
fn test_db_path_default() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config::new("prj".to_string()).unwrap();
    let db_path = get_db_path(&work_dir, &config);
    assert_eq!(db_path, PathBuf::from("/project/.wok/issues.db"));
}

#[test]
fn test_db_path_with_workspace() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config {
        prefix: "prj".to_string(),
        workspace: Some("../shared".to_string()),
        remote: None,
    };
    let db_path = get_db_path(&work_dir, &config);
    assert_eq!(db_path, PathBuf::from("/project/../shared/issues.db"));
}

#[test]
fn test_invalid_prefix() {
    assert!(Config::new("a".to_string()).is_err()); // too short
    assert!(Config::new("AB".to_string()).is_err()); // uppercase
}

#[test]
fn test_db_path_with_absolute_workspace() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config {
        prefix: "prj".to_string(),
        workspace: Some("/absolute/path".to_string()),
        remote: None,
    };
    let db_path = get_db_path(&work_dir, &config);
    assert_eq!(db_path, PathBuf::from("/absolute/path/issues.db"));
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
        workspace: Some("shared".to_string()),
        remote: None,
    };
    config.save(&work_dir).unwrap();

    let loaded = Config::load(&work_dir).unwrap();
    assert_eq!(loaded.prefix, "myproj");
    assert_eq!(loaded.workspace, Some("shared".to_string()));
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
fn test_local_mode_by_default() {
    let config = Config::new("proj".to_string()).unwrap();
    assert!(!config.is_remote_mode());
    assert!(config.remote_url().is_none());
}

#[test]
fn test_remote_mode() {
    let config = Config {
        prefix: "proj".to_string(),
        workspace: None,
        remote: Some(RemoteConfig {
            url: "ws://remote:7890".to_string(),
            branch: "wk/oplog".to_string(),
            worktree: None,
            reconnect_max_retries: 10,
            reconnect_max_delay_secs: 30,
            heartbeat_interval_ms: 30_000,
            heartbeat_timeout_ms: 10_000,
            connect_timeout_secs: 2,
        }),
    };
    assert!(config.is_remote_mode());
    assert_eq!(config.remote_url(), Some("ws://remote:7890"));
}

#[test]
fn test_parse_remote_config_from_toml() {
    let toml_content = r#"
prefix = "proj"

[remote]
url = "ws://remote.example.com:7890"
"#;

    let config: Config = toml::from_str(toml_content).unwrap();
    assert_eq!(config.prefix, "proj");
    assert!(config.is_remote_mode());

    let remote = config.remote.unwrap();
    assert_eq!(remote.url, "ws://remote.example.com:7890");
    assert_eq!(remote.reconnect_max_retries, 10); // default
    assert_eq!(remote.reconnect_max_delay_secs, 30); // default
}

#[test]
fn test_parse_remote_config_with_overrides() {
    let toml_content = r#"
prefix = "proj"

[remote]
url = "ws://custom:1234"
reconnect_max_retries = 5
reconnect_max_delay_secs = 60
"#;

    let config: Config = toml::from_str(toml_content).unwrap();
    let remote = config.remote.unwrap();
    assert_eq!(remote.url, "ws://custom:1234");
    assert_eq!(remote.reconnect_max_retries, 5);
    assert_eq!(remote.reconnect_max_delay_secs, 60);
}

#[test]
fn test_save_and_load_remote_config() {
    let temp = TempDir::new().unwrap();
    let work_dir = init_work_dir(temp.path(), "test").unwrap();

    // Modify config to add remote
    let mut config = Config::load(&work_dir).unwrap();
    config.remote = Some(RemoteConfig {
        url: "ws://test:7890".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 15,
        reconnect_max_delay_secs: 45,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    });
    config.save(&work_dir).unwrap();

    // Reload and verify
    let reloaded = Config::load(&work_dir).unwrap();
    assert!(reloaded.is_remote_mode());
    let remote = reloaded.remote.unwrap();
    assert_eq!(remote.url, "ws://test:7890");
    assert_eq!(remote.reconnect_max_retries, 15);
    assert_eq!(remote.reconnect_max_delay_secs, 45);
}

#[test]
fn test_daemon_dir_default() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config::new("prj".to_string()).unwrap();
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    assert_eq!(daemon_dir, PathBuf::from("/project/.wok"));
}

#[test]
fn test_daemon_dir_with_workspace() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config {
        prefix: "prj".to_string(),
        workspace: Some("../shared".to_string()),
        remote: None,
    };
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    assert_eq!(daemon_dir, PathBuf::from("/project/../shared"));
}

#[test]
fn test_daemon_dir_with_absolute_workspace() {
    let work_dir = PathBuf::from("/project/.wok");
    let config = Config {
        prefix: "prj".to_string(),
        workspace: Some("/absolute/path".to_string()),
        remote: None,
    };
    let daemon_dir = get_daemon_dir(&work_dir, &config);
    assert_eq!(daemon_dir, PathBuf::from("/absolute/path"));
}

#[test]
fn test_daemon_dir_is_parent_of_db_path() {
    // Verify that daemon_dir is the parent directory of db_path
    let work_dir = PathBuf::from("/project/.wok");

    // Test with no workspace
    let config1 = Config::new("prj".to_string()).unwrap();
    let db_path1 = get_db_path(&work_dir, &config1);
    let daemon_dir1 = get_daemon_dir(&work_dir, &config1);
    assert_eq!(db_path1.parent().unwrap(), daemon_dir1.as_path());

    // Test with relative workspace
    let config2 = Config {
        prefix: "prj".to_string(),
        workspace: Some("../shared".to_string()),
        remote: None,
    };
    let db_path2 = get_db_path(&work_dir, &config2);
    let daemon_dir2 = get_daemon_dir(&work_dir, &config2);
    assert_eq!(db_path2.parent().unwrap(), daemon_dir2.as_path());

    // Test with absolute workspace
    let config3 = Config {
        prefix: "prj".to_string(),
        workspace: Some("/absolute/path".to_string()),
        remote: None,
    };
    let db_path3 = get_db_path(&work_dir, &config3);
    let daemon_dir3 = get_daemon_dir(&work_dir, &config3);
    assert_eq!(db_path3.parent().unwrap(), daemon_dir3.as_path());
}

#[test]
fn test_config_new_with_workspace_no_prefix() {
    let config = Config::new_with_workspace(None, "/path/to/ws".to_string()).unwrap();
    assert!(config.prefix.is_empty());
    assert_eq!(config.workspace, Some("/path/to/ws".to_string()));
}

#[test]
fn test_config_new_with_workspace_with_prefix() {
    let config =
        Config::new_with_workspace(Some("prj".to_string()), "/path/to/ws".to_string()).unwrap();
    assert_eq!(config.prefix, "prj");
    assert_eq!(config.workspace, Some("/path/to/ws".to_string()));
}

#[test]
fn test_config_new_with_workspace_invalid_prefix() {
    let result = Config::new_with_workspace(
        Some("AB".to_string()), // Invalid: uppercase
        "/path/to/ws".to_string(),
    );
    assert!(result.is_err());
}

#[test]
fn test_config_empty_prefix_not_serialized() {
    let config = Config {
        prefix: String::new(),
        workspace: Some("/ws/path".to_string()),
        remote: None,
    };
    let toml = toml::to_string(&config).unwrap();
    assert!(!toml.contains("prefix ="));
    assert!(toml.contains(r#"workspace = "/ws/path""#));
}

#[test]
fn test_config_deserialize_without_prefix() {
    let toml = r#"workspace = "/path/to/workspace""#;
    let config: Config = toml::from_str(toml).unwrap();
    assert!(config.prefix.is_empty());
    assert_eq!(config.workspace, Some("/path/to/workspace".to_string()));
}

#[test]
fn test_init_workspace_link() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let work_dir = init_workspace_link(temp.path(), ws_dir.to_str().unwrap(), None).unwrap();

    assert!(work_dir.exists());
    assert!(work_dir.join("config.toml").exists());
    assert!(!work_dir.join("issues.db").exists());

    let config = Config::load(&work_dir).unwrap();
    assert!(config.prefix.is_empty());
    assert_eq!(config.workspace, Some(ws_dir.to_str().unwrap().to_string()));
}

#[test]
fn test_init_workspace_link_with_prefix() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let work_dir = init_workspace_link(temp.path(), ws_dir.to_str().unwrap(), Some("prj")).unwrap();

    let config = Config::load(&work_dir).unwrap();
    assert_eq!(config.prefix, "prj");
    assert_eq!(config.workspace, Some(ws_dir.to_str().unwrap().to_string()));
}

#[test]
fn test_init_workspace_link_already_initialized() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    init_workspace_link(temp.path(), ws_dir.to_str().unwrap(), None).unwrap();

    let result = init_workspace_link(temp.path(), ws_dir.to_str().unwrap(), None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("already initialized"));
}

#[test]
fn test_init_workspace_link_invalid_prefix() {
    let temp = TempDir::new().unwrap();
    let ws_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let result = init_workspace_link(temp.path(), ws_dir.to_str().unwrap(), Some("A")); // Too short
    assert!(result.is_err());
}

#[test]
fn test_init_workspace_link_workspace_not_found() {
    let temp = TempDir::new().unwrap();
    let result = init_workspace_link(temp.path(), "/nonexistent/path", None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("workspace not found"));
}

#[test]
fn test_write_gitignore_remote_mode() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    write_gitignore(&work_dir, false).unwrap();

    let gitignore_path = work_dir.join(".gitignore");
    assert!(gitignore_path.exists());

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("current/"));
    assert!(content.contains("issues.db"));
    assert!(!content.contains("config.toml"));
}

#[test]
fn test_write_gitignore_local_mode() {
    let temp = TempDir::new().unwrap();
    let work_dir = temp.path().join(".wok");
    std::fs::create_dir_all(&work_dir).unwrap();

    write_gitignore(&work_dir, true).unwrap();

    let gitignore_path = work_dir.join(".gitignore");
    assert!(gitignore_path.exists());

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("current/"));
    assert!(content.contains("issues.db"));
    assert!(content.contains("config.toml"));
}

#[test]
fn test_remote_config_validate_url_websocket() {
    let config = RemoteConfig {
        url: "ws://localhost:7890".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());

    let config = RemoteConfig {
        url: "wss://secure.example.com:443".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());
}

#[test]
fn test_remote_config_validate_url_git_same_repo() {
    let config = RemoteConfig {
        url: "git:.".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());

    let config = RemoteConfig {
        url: ".".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());
}

#[test]
fn test_remote_config_validate_url_git_separate_repo() {
    // Git prefix with path
    let config = RemoteConfig {
        url: "git:~/repos/tracker".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());

    // Git prefix with SSH URL
    let config = RemoteConfig {
        url: "git:git@github.com:org/repo.git".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());
}

#[test]
fn test_remote_config_validate_url_bare_ssh() {
    let config = RemoteConfig {
        url: "git@github.com:org/repo.git".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());

    let config = RemoteConfig {
        url: "ssh://git@github.com/org/repo.git".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_none());
}

#[test]
fn test_remote_config_validate_url_invalid() {
    // Not a recognized URL format
    let config = RemoteConfig {
        url: "not-a-valid-url".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    let error = config.validate_url();
    assert!(error.is_some());
    assert!(error.unwrap().contains("invalid remote URL"));

    // HTTP is not supported
    let config = RemoteConfig {
        url: "http://example.com".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    assert!(config.validate_url().is_some());

    // Empty git: URL
    let config = RemoteConfig {
        url: "git:".to_string(),
        branch: "wk/oplog".to_string(),
        worktree: None,
        reconnect_max_retries: 10,
        reconnect_max_delay_secs: 30,
        heartbeat_interval_ms: 30_000,
        heartbeat_timeout_ms: 10_000,
        connect_timeout_secs: 2,
    };
    let error = config.validate_url();
    assert!(error.is_some());
    assert!(error.unwrap().contains("requires a path"));
}
