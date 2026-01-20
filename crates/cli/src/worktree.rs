// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Git worktree management for oplog sync.
//!
//! For same-repo mode (git:.), the worktree is placed at `.git/wk/oplog`.
//! This provides branch protection since git prevents deletion of branches
//! with active worktrees.
//!
//! For separate repos, the worktree location is:
//! - XDG data dir: `~/.local/share/wk/<repo-hash>/oplog/`
//! - Or `.wok/oplog/` if config specifies `worktree = true`

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sha2::{Digest, Sha256};

use crate::config::{RemoteConfig, RemoteType};
use crate::error::{Error, Result};

/// Name of the oplog file within the worktree.
pub const OPLOG_FILE: &str = "oplog.jsonl";

/// Information about an initialized oplog worktree.
#[derive(Debug, Clone)]
pub struct OplogWorktree {
    /// Path to the worktree directory.
    pub path: PathBuf,
    /// Path to the oplog.jsonl file.
    pub oplog_path: PathBuf,
    /// The branch name (e.g., "wk/oplog").
    pub branch: String,
    /// Whether this is a same-repo worktree (git:.) or separate repo.
    #[allow(dead_code)] // Used for future sync optimizations
    pub is_same_repo: bool,
}

/// Resolves the oplog worktree path based on config and environment.
///
/// # Arguments
/// * `work_dir` - The .work directory path
/// * `remote` - Remote configuration
///
/// # Returns
/// The path where the oplog worktree should be located.
pub fn resolve_oplog_path(work_dir: &Path, remote: &RemoteConfig) -> Result<PathBuf> {
    // Only git remotes use a worktree
    if remote.remote_type() != RemoteType::Git {
        return Err(Error::Config(
            "oplog worktree only applies to git remotes".to_string(),
        ));
    }

    // For same-repo mode, always use .git/wk/oplog
    if remote.is_same_repo() {
        let repo_root = work_dir
            .parent()
            .ok_or_else(|| Error::Config("work_dir has no parent".to_string()))?;
        let git_dir = find_git_dir(repo_root)?;
        return Ok(git_dir.join("wk").join("oplog"));
    }

    // For separate repos, use XDG or fallback to .wok/oplog
    if remote.worktree == Some(true) {
        return Ok(work_dir.join("oplog"));
    }
    if let Some(data_dir) = dirs::data_dir() {
        let repo_hash = compute_repo_hash(work_dir)?;
        let oplog_dir = data_dir.join("wk").join(repo_hash).join("oplog");
        return Ok(oplog_dir);
    }
    Ok(work_dir.join("oplog"))
}

/// Computes a stable hash of the repository root for XDG path uniqueness.
fn compute_repo_hash(work_dir: &Path) -> Result<String> {
    // Get the repo root (parent of .work)
    let repo_root = work_dir
        .parent()
        .ok_or_else(|| Error::Config("work_dir has no parent".to_string()))?;

    // Canonicalize to get absolute path
    let canonical = repo_root
        .canonicalize()
        .map_err(|e| Error::Config(format!("failed to canonicalize path: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(&result[..8])) // Use first 8 bytes (16 hex chars)
}

/// Finds the .git directory for a repository.
/// Handles both regular repos (.git directory) and worktrees (.git file).
fn find_git_dir(repo_root: &Path) -> Result<PathBuf> {
    let git_path = repo_root.join(".git");

    if git_path.is_dir() {
        // Regular repository
        return Ok(git_path);
    }

    if git_path.is_file() {
        // Worktree - .git is a file containing "gitdir: /path/to/real/git/dir"
        let content = fs::read_to_string(&git_path)
            .map_err(|e| Error::Config(format!("failed to read .git file: {}", e)))?;
        if let Some(path) = content.strip_prefix("gitdir: ") {
            let gitdir = PathBuf::from(path.trim());
            // Navigate up from .git/worktrees/<name> to .git
            if let Some(parent) = gitdir.parent().and_then(|p| p.parent()) {
                return Ok(parent.to_path_buf());
            }
        }
        return Err(Error::Config("invalid .git file format".to_string()));
    }

    Err(Error::Config(format!(
        "not a git repository: {}",
        repo_root.display()
    )))
}

/// Initializes the oplog worktree for a git remote.
///
/// This sets up the orphan branch and worktree if they don't exist.
///
/// # Arguments
/// * `work_dir` - The .work directory path
/// * `remote` - Remote configuration (must be a git remote)
///
/// # Returns
/// Information about the initialized worktree.
pub fn init_oplog_worktree(work_dir: &Path, remote: &RemoteConfig) -> Result<OplogWorktree> {
    if remote.remote_type() != RemoteType::Git {
        return Err(Error::Config(
            "init_oplog_worktree only applies to git remotes".to_string(),
        ));
    }

    let worktree_path = resolve_oplog_path(work_dir, remote)?;
    let branch = &remote.branch;
    let is_same_repo = remote.is_same_repo();

    // Get the repo root
    let repo_root = work_dir
        .parent()
        .ok_or_else(|| Error::Config("work_dir has no parent".to_string()))?;

    // For same-repo mode, work with the current repository
    // For separate repo mode, we'd clone/init the separate repo
    if is_same_repo {
        init_same_repo_worktree(repo_root, &worktree_path, branch)?;
    } else {
        // For separate repos, we need the URL
        let git_url = remote
            .git_url()
            .ok_or_else(|| Error::Config("no git URL for separate repo".to_string()))?;
        init_separate_repo_worktree(&worktree_path, git_url, branch)?;
    }

    let oplog_path = worktree_path.join(OPLOG_FILE);

    Ok(OplogWorktree {
        path: worktree_path,
        oplog_path,
        branch: branch.clone(),
        is_same_repo,
    })
}

/// Initializes a same-repo worktree (git:.).
fn init_same_repo_worktree(repo_root: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    // Ensure the worktree parent directory exists (.git/wk/)
    if let Some(parent) = worktree_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Check if the orphan branch exists
    if !branch_exists(repo_root, branch)? {
        create_orphan_branch(repo_root, branch)?;
    }

    // Check if worktree already exists and is valid
    if worktree_path.exists() {
        if is_valid_worktree(worktree_path) {
            return Ok(());
        }
        // Remove invalid directory and recreate
        fs::remove_dir_all(worktree_path)?;
    }

    // Add the worktree
    add_worktree(repo_root, worktree_path, branch)?;

    Ok(())
}

/// Initializes a separate-repo worktree.
fn init_separate_repo_worktree(worktree_path: &Path, git_url: &str, branch: &str) -> Result<()> {
    // Ensure the worktree parent directory exists
    if let Some(parent) = worktree_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // If the worktree doesn't exist, clone the repo
    if !worktree_path.exists() {
        // Clone just the oplog branch (shallow clone for efficiency)
        let output = Command::new("git")
            .args([
                "clone",
                "--single-branch",
                "--branch",
                branch,
                "--depth",
                "1",
                git_url,
            ])
            .arg(worktree_path)
            .output()
            .map_err(Error::Io)?;

        // If clone fails (branch doesn't exist), init empty repo and create branch
        if !output.status.success() {
            init_empty_oplog_repo(worktree_path, git_url, branch)?;
        }
    }

    Ok(())
}

/// Creates an empty oplog repo when cloning fails (branch doesn't exist yet).
fn init_empty_oplog_repo(worktree_path: &Path, git_url: &str, branch: &str) -> Result<()> {
    // Create directory
    fs::create_dir_all(worktree_path)?;

    // Init git repo
    run_git(worktree_path, &["init"])?;

    // Add remote
    run_git(worktree_path, &["remote", "add", "origin", git_url])?;

    // Create and checkout the orphan branch
    run_git(worktree_path, &["checkout", "--orphan", branch])?;

    // Create empty oplog file
    let oplog_path = worktree_path.join(OPLOG_FILE);
    fs::write(&oplog_path, "")?;

    // Initial commit
    run_git(worktree_path, &["add", OPLOG_FILE])?;
    run_git(worktree_path, &["commit", "-m", "Initialize oplog"])?;

    Ok(())
}

/// Checks if a branch exists in the repository.
fn branch_exists(repo_path: &Path, branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch)])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(Error::Io)?;

    Ok(output.success())
}

/// Creates an orphan branch with an empty oplog using git plumbing commands.
///
/// This approach avoids touching the main working directory by using low-level
/// git commands (hash-object, mktree, commit-tree, update-ref) instead of
/// checkout-based workflows.
fn create_orphan_branch(repo_path: &Path, branch: &str) -> Result<()> {
    // 1. Create a blob for the empty oplog file
    let blob_hash = run_git_trimmed(repo_path, &["hash-object", "-w", "--stdin"], Some(""))?;

    // 2. Create a tree containing the oplog file
    // Format: "mode type hash\tfilename"
    let tree_input = format!("100644 blob {}\t{}", blob_hash, OPLOG_FILE);
    let tree_hash = run_git_trimmed(repo_path, &["mktree"], Some(&tree_input))?;

    // 3. Create an orphan commit (no parents) with the tree
    let commit_hash = run_git_trimmed(
        repo_path,
        &["commit-tree", &tree_hash, "-m", "Initialize oplog"],
        None,
    )?;

    // 4. Create the branch ref pointing to the commit
    run_git(
        repo_path,
        &[
            "update-ref",
            &format!("refs/heads/{}", branch),
            &commit_hash,
        ],
    )?;

    Ok(())
}

/// Adds a git worktree.
fn add_worktree(repo_path: &Path, worktree_path: &Path, branch: &str) -> Result<()> {
    run_git(
        repo_path,
        &["worktree", "add", &worktree_path.to_string_lossy(), branch],
    )?;
    Ok(())
}

/// Sets up default git author/committer environment variables.
/// This ensures git commands work even when no user is configured
/// (e.g., in CI environments or when HOME points to an empty directory).
fn setup_git_env(cmd: &mut Command) {
    // Set defaults if not present or empty in environment
    if std::env::var("GIT_AUTHOR_NAME")
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        cmd.env("GIT_AUTHOR_NAME", "wk");
    }
    if std::env::var("GIT_AUTHOR_EMAIL")
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        cmd.env("GIT_AUTHOR_EMAIL", "wk@localhost");
    }
    if std::env::var("GIT_COMMITTER_NAME")
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        cmd.env("GIT_COMMITTER_NAME", "wk");
    }
    if std::env::var("GIT_COMMITTER_EMAIL")
        .map(|v| v.is_empty())
        .unwrap_or(true)
    {
        cmd.env("GIT_COMMITTER_EMAIL", "wk@localhost");
    }
}

/// Runs a git command in the given directory.
fn run_git(dir: &Path, args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(dir).args(args);
    setup_git_env(&mut cmd);

    let output = cmd.output().map_err(Error::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::Config(format!("git {} failed: {}", args[0], stderr)))
    }
}

/// Runs a git command with optional stdin, returning trimmed output.
fn run_git_trimmed(dir: &Path, args: &[&str], stdin_data: Option<&str>) -> Result<String> {
    use std::io::Write;

    let mut cmd = Command::new("git");
    cmd.current_dir(dir)
        .args(args)
        .stdin(if stdin_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    setup_git_env(&mut cmd);

    let mut child = cmd.spawn().map_err(Error::Io)?;

    if let Some(data) = stdin_data {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data.as_bytes()).map_err(Error::Io)?;
        }
    }

    let output = child.wait_with_output().map_err(Error::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::Config(format!("git {} failed: {}", args[0], stderr)))
    }
}

/// Checks if a path is a valid git worktree.
fn is_valid_worktree(path: &Path) -> bool {
    let git_file = path.join(".git");
    if git_file.is_file() {
        if let Ok(content) = fs::read_to_string(&git_file) {
            return content.starts_with("gitdir:");
        }
    }
    false
}

/// Checks if a path is inside a git worktree.
#[allow(dead_code)] // Will be used for worktree validation
pub fn is_worktree(path: &Path) -> bool {
    let git_dir = path.join(".git");
    if git_dir.is_file() {
        // Worktrees have a .git file pointing to the main repo
        if let Ok(content) = fs::read_to_string(&git_dir) {
            return content.starts_with("gitdir:");
        }
    }
    false
}

/// Gets the oplog path for an existing worktree, verifying it exists.
#[allow(dead_code)] // Part of public worktree API
pub fn get_oplog_path(worktree: &OplogWorktree) -> Result<&Path> {
    if !worktree.oplog_path.exists() {
        // Create empty oplog file if it doesn't exist
        fs::write(&worktree.oplog_path, "")?;
    }
    Ok(&worktree.oplog_path)
}

/// Reads all operations from the oplog file.
pub fn read_oplog(oplog_path: &Path) -> Result<Vec<wk_core::Op>> {
    if !oplog_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(oplog_path)?;
    let reader = BufReader::new(file);
    let mut ops = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let op: wk_core::Op = serde_json::from_str(&line)?;
        ops.push(op);
    }

    Ok(ops)
}

/// Appends operations to the oplog file.
pub fn append_oplog(oplog_path: &Path, ops: &[wk_core::Op]) -> Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(oplog_path)?;

    for op in ops {
        let json = serde_json::to_string(op)?;
        writeln!(file, "{}", json)?;
    }

    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
#[path = "worktree_tests.rs"]
mod tests;
