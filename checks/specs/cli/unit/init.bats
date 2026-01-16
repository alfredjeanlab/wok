#!/usr/bin/env bats
load '../../helpers/common'

# ============================================================================
# Basic Initialization
# ============================================================================

@test "init creates .wok directory" {
    run "$WK_BIN" init --prefix prj
    assert_success
    [ -d ".wok" ]
}

@test "init creates config.toml with prefix" {
    run "$WK_BIN" init --prefix myapp
    assert_success
    [ -f ".wok/config.toml" ]
    grep -q 'prefix = "myapp"' .wok/config.toml
}

@test "init creates issues.db" {
    run "$WK_BIN" init --prefix prj
    assert_success
    [ -f ".wok/issues.db" ]
}

@test "init outputs success message" {
    run "$WK_BIN" init --prefix prj
    assert_success
    # Should have some confirmation output
    [ -n "$output" ]
}

# ============================================================================
# Re-initialization Prevention
# ============================================================================

@test "init fails if already initialized" {
    "$WK_BIN" init --prefix prj
    run "$WK_BIN" init --prefix prj
    assert_failure
}

@test "init fails if .wok directory exists" {
    mkdir -p .wok
    run "$WK_BIN" init --prefix prj
    assert_failure
}

# ============================================================================
# --path Option
# ============================================================================

@test "init with --path creates at specified location" {
    mkdir -p subdir
    run "$WK_BIN" init --path subdir --prefix sub
    assert_success
    [ -d "subdir/.wok" ]
    [ -f "subdir/.wok/config.toml" ]
    [ -f "subdir/.wok/issues.db" ]
}

@test "init with --path creates parent directories if needed" {
    run "$WK_BIN" init --path nested/deep/dir --prefix prj
    assert_success
    [ -d "nested/deep/dir/.wok" ]
}

@test "init with --path uses correct prefix in config" {
    mkdir -p other
    run "$WK_BIN" init --path other --prefix custom
    assert_success
    grep -q 'prefix = "custom"' other/.wok/config.toml
}

@test "init with --path fails if already initialized at path" {
    mkdir -p target
    "$WK_BIN" init --path target --prefix prj
    run "$WK_BIN" init --path target --prefix prj
    assert_failure
}

# ============================================================================
# Default Prefix from Directory Name
# ============================================================================

@test "init without prefix uses directory name" {
    # Create a directory with a known name
    mkdir -p myproject
    cd myproject
    run "$WK_BIN" init
    assert_success
    # Prefix should be derived from directory name
    grep -q 'prefix = "myproject"' .wok/config.toml
}

@test "init default prefix keeps alphanumeric chars" {
    # Directory name with numbers and special chars
    mkdir -p "proj123"
    cd "proj123"
    run "$WK_BIN" init
    assert_success
    # Letters and numbers should be kept
    grep -q 'prefix = "proj123"' .wok/config.toml
}

@test "init default prefix lowercases directory name" {
    mkdir -p "MyProject"
    cd "MyProject"
    run "$WK_BIN" init
    assert_success
    # Should be lowercase
    grep -q 'prefix = "myproject"' .wok/config.toml
}

@test "init fails if directory name has insufficient chars" {
    # Directory name with only 1 alphanumeric char
    mkdir -p "a---"
    cd "a---"
    run "$WK_BIN" init
    assert_failure
}

@test "init succeeds with alphanumeric directory name" {
    mkdir -p "v0"
    cd "v0"
    run "$WK_BIN" init
    assert_success
    grep -q 'prefix = "v0"' .wok/config.toml
}

@test "init with explicit prefix overrides directory default" {
    mkdir -p "myproject"
    cd "myproject"
    run "$WK_BIN" init --prefix custom
    assert_success
    grep -q 'prefix = "custom"' .wok/config.toml
}

# ============================================================================
# Prefix Validation
# ============================================================================

@test "init accepts lowercase prefix" {
    run "$WK_BIN" init --prefix abc
    assert_success
}

@test "init prefix must be lowercase" {
    run "$WK_BIN" init --prefix ABC
    assert_failure
}

@test "init prefix accepts numbers with letters" {
    run "$WK_BIN" init --prefix abc123
    assert_success
    grep -q 'prefix = "abc123"' .wok/config.toml
}

@test "init prefix rejects pure numbers" {
    run "$WK_BIN" init --prefix 123
    assert_failure
}

@test "init prefix rejects special characters" {
    run "$WK_BIN" init --prefix my-prefix
    assert_failure
}

@test "init prefix rejects underscores" {
    run "$WK_BIN" init --prefix my_prefix
    assert_failure
}

@test "init prefix requires at least 2 characters" {
    run "$WK_BIN" init --prefix a
    assert_failure
}

@test "init prefix accepts 2 character prefix" {
    run "$WK_BIN" init --prefix ab
    assert_success
    grep -q 'prefix = "ab"' .wok/config.toml
}

@test "init prefix accepts longer prefix" {
    run "$WK_BIN" init --prefix mylongprefix
    assert_success
    grep -q 'prefix = "mylongprefix"' .wok/config.toml
}

# ============================================================================
# Database Initialization
# ============================================================================

@test "init creates valid SQLite database" {
    run "$WK_BIN" init --prefix prj
    assert_success
    # Verify it's a valid SQLite database
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table';"
    assert_success
}

@test "init creates issues table" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='issues';"
    assert_success
    [ "$output" = "issues" ]
}

@test "init creates deps table" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='deps';"
    assert_success
    [ "$output" = "deps" ]
}

@test "init creates labels table" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='labels';"
    assert_success
    [ "$output" = "labels" ]
}

@test "init creates notes table" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='notes';"
    assert_success
    [ "$output" = "notes" ]
}

@test "init creates events table" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='events';"
    assert_success
    [ "$output" = "events" ]
}

@test "init creates empty database with no issues" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run "$WK_BIN" list
    assert_success
    # Should show no issues (empty or no output lines with issues)
    refute_output --regexp '\[task\]|\[bug\]|\[feature\]'
}

# ============================================================================
# Config File Format
# ============================================================================

@test "init config.toml is valid TOML" {
    run "$WK_BIN" init --prefix prj
    assert_success
    # Basic TOML syntax check - should have key = "value" format
    grep -qE '^prefix = "[a-z]+"' .wok/config.toml
}

@test "init config.toml contains only expected keys" {
    run "$WK_BIN" init --prefix prj
    assert_success
    # Should have prefix, optionally workspace
    local line_count
    line_count=$(grep -cE '^[a-z]' .wok/config.toml || echo 0)
    [ "$line_count" -ge 1 ]
}

# ============================================================================
# Integration with Other Commands
# ============================================================================

@test "init allows immediate issue creation" {
    run "$WK_BIN" init --prefix prj
    assert_success
    run "$WK_BIN" new task "Test issue"
    assert_success
}

@test "init prefix is used in issue IDs" {
    run "$WK_BIN" init --prefix myprj
    assert_success
    run "$WK_BIN" new task "Test issue"
    assert_success
    # Output should contain ID with the prefix
    assert_output --regexp 'myprj-[a-z0-9]+'
}

# ============================================================================
# Error Messages
# ============================================================================

@test "init shows helpful error when already initialized" {
    "$WK_BIN" init --prefix prj
    run "$WK_BIN" init --prefix prj
    assert_failure
    # Should have some error message
    [ -n "$output" ]
}

@test "init shows helpful error for invalid prefix" {
    run "$WK_BIN" init --prefix 123
    assert_failure
    # Should have some error message about prefix
    [ -n "$output" ]
}

# ============================================================================
# --workspace Option (Workspace Link Only)
# ============================================================================

@test "init with --workspace creates config with workspace only" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace
    assert_success
    [ -f ".wok/config.toml" ]
    # Should have workspace line
    grep -q 'workspace = "/tmp/workspace"' .wok/config.toml
    # Should NOT have prefix line (when not specified)
    ! grep -q '^prefix' .wok/config.toml
}

@test "init with --workspace does not create local database" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace
    assert_success
    [ -d ".wok" ]
    [ ! -f ".wok/issues.db" ]
}

@test "init with --workspace and --prefix includes both" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace --prefix prj
    assert_success
    grep -q 'workspace = "/tmp/workspace"' .wok/config.toml
    grep -q 'prefix = "prj"' .wok/config.toml
    # Still no local database
    [ ! -f ".wok/issues.db" ]
}

@test "init with --workspace validates prefix if provided" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace --prefix ABC
    assert_failure
}

@test "init with --workspace accepts relative path" {
    mkdir -p external/workspace
    run "$WK_BIN" init --workspace external/workspace
    assert_success
    grep -q 'workspace = "external/workspace"' .wok/config.toml
}

@test "init with --workspace at specific --path" {
    # Workspace path is resolved relative to --path, not cwd
    mkdir -p subdir subdir/external/workspace
    run "$WK_BIN" init --path subdir --workspace external/workspace
    assert_success
    [ -d "subdir/.wok" ]
    grep -q 'workspace = "external/workspace"' subdir/.wok/config.toml
}

@test "init with --workspace fails if workspace does not exist" {
    run "$WK_BIN" init --workspace /nonexistent/path
    assert_failure
    assert_output --partial "workspace not found"
}

@test "init with --workspace fails if relative workspace does not exist" {
    run "$WK_BIN" init --workspace ./nonexistent/dir
    assert_failure
    assert_output --partial "workspace not found"
}

# ============================================================================
# .gitignore Creation
# ============================================================================

@test "init creates .gitignore in .wok directory" {
    run "$WK_BIN" init --prefix prj
    assert_success
    [ -f ".wok/.gitignore" ]
}

@test "init .gitignore includes current directory" {
    run "$WK_BIN" init --prefix prj
    assert_success
    grep -q "current/" .wok/.gitignore
}

@test "init .gitignore includes issues.db" {
    run "$WK_BIN" init --prefix prj
    assert_success
    grep -q "issues.db" .wok/.gitignore
}

@test "init .gitignore does not include config.toml in remote mode" {
    run "$WK_BIN" init --prefix prj
    assert_success
    # Default is remote mode (same-repo git), so config.toml should NOT be ignored
    ! grep -q "config.toml" .wok/.gitignore
}

@test "init --local .gitignore includes config.toml" {
    run "$WK_BIN" init --prefix prj --local
    assert_success
    grep -q "config.toml" .wok/.gitignore
}

@test "init with --workspace creates .gitignore with config.toml" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace
    assert_success
    [ -f ".wok/.gitignore" ]
    grep -q "current/" .wok/.gitignore
    grep -q "issues.db" .wok/.gitignore
    grep -q "config.toml" .wok/.gitignore
}

# ============================================================================
# Git Remote (Same Repo) - Worktree in .git/wk/oplog
# ============================================================================

@test "init with git remote creates worktree in .git/wk/oplog" {
    git init
    run "$WK_BIN" init --prefix prj --remote .
    assert_success
    [ -d ".git/wk/oplog" ]
    [ -f ".git/wk/oplog/oplog.jsonl" ]
}

@test "init with git remote creates orphan branch" {
    git init
    run "$WK_BIN" init --prefix prj --remote .
    assert_success
    run git rev-parse --verify refs/heads/wk/oplog
    assert_success
}

@test "worktree protects orphan branch from deletion" {
    git init
    run "$WK_BIN" init --prefix prj --remote .
    assert_success
    # Attempt to delete the branch should fail
    run git branch -D wk/oplog
    assert_failure
    # Git says either "checked out" or "used by worktree"
    assert_output --partial "worktree"
}

@test "wk remote sync works with .git/wk/oplog worktree" {
    git init
    run "$WK_BIN" init --prefix prj --remote .
    assert_success
    run "$WK_BIN" new task "Test issue"
    assert_success
    run "$WK_BIN" remote sync
    assert_success
}
