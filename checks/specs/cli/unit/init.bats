#!/usr/bin/env bats
load '../../helpers/common'


# Each test needs fresh directory, so use default setup/teardown

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    # Stop daemon if running (from remote sync tests)
    if command -v timeout >/dev/null 2>&1; then
        timeout 1 "$WK_BIN" remote stop 2>/dev/null || true
    fi
    # Force kill by PID if daemon.pid exists
    local daemon_pid_file="${TEST_DIR}/.wok/daemon.pid"
    if [ -f "$daemon_pid_file" ]; then
        local pid
        pid=$(cat "$daemon_pid_file" 2>/dev/null || true)
        if [ -n "$pid" ]; then
            kill -9 "$pid" 2>/dev/null || true
        fi
    fi
    sleep 0.01
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

@test "init creates .wok directory and fails if already initialized" {
    run "$WK_BIN" init --prefix myapp
    assert_success
    [ -n "$output" ]
    [ -d ".wok" ]
    [ -f ".wok/config.toml" ]
    [ -f ".wok/issues.db" ]
    grep -q 'prefix = "myapp"' .wok/config.toml

    # Fails if already initialized
    run "$WK_BIN" init --prefix prj
    assert_failure
    [ -n "$output" ]

    # Also fails if .wok exists
    rm -rf .wok
    mkdir -p .wok
    run "$WK_BIN" init --prefix prj
    assert_failure
}

@test "init with --path creates at specified location" {
    mkdir -p subdir
    run "$WK_BIN" init --path subdir --prefix sub
    assert_success
    [ -d "subdir/.wok" ]
    [ -f "subdir/.wok/config.toml" ]
    [ -f "subdir/.wok/issues.db" ]
    grep -q 'prefix = "sub"' subdir/.wok/config.toml

    # Creates parent directories if needed
    run "$WK_BIN" init --path nested/deep/dir --prefix prj
    assert_success
    [ -d "nested/deep/dir/.wok" ]

    # Fails if already initialized at path
    run "$WK_BIN" init --path subdir --prefix sub
    assert_failure
}

@test "init prefix handling and validation" {
    # Without prefix uses directory name
    mkdir -p myproject && cd myproject
    run "$WK_BIN" init
    assert_success
    grep -q 'prefix = "myproject"' .wok/config.toml
    cd ..

    # Lowercases and keeps alphanumeric
    mkdir -p "MyProject123" && cd "MyProject123"
    run "$WK_BIN" init
    assert_success
    grep -q 'prefix = "myproject123"' .wok/config.toml
    cd ..

    # Explicit prefix overrides directory default
    mkdir -p "somedir" && cd "somedir"
    run "$WK_BIN" init --prefix custom
    assert_success
    grep -q 'prefix = "custom"' .wok/config.toml
    cd ..

    # Fails with invalid directory name for prefix
    mkdir -p "a---" && cd "a---"
    run "$WK_BIN" init
    assert_failure
    cd ..

    # Valid prefixes
    mkdir -p validpfx && cd validpfx
    run "$WK_BIN" init --prefix abc
    assert_success
    rm -rf .wok

    run "$WK_BIN" init --prefix ab
    assert_success
    rm -rf .wok

    run "$WK_BIN" init --prefix abc123
    assert_success
    grep -q 'prefix = "abc123"' .wok/config.toml
    rm -rf .wok

    run "$WK_BIN" init --prefix mylongprefix
    assert_success
    rm -rf .wok

    # Invalid prefixes
    local invalid_prefixes=("ABC" "123" "my-prefix" "my_prefix" "a")
    for prefix in "${invalid_prefixes[@]}"; do
        run "$WK_BIN" init --prefix "$prefix"
        assert_failure
    done
}

@test "init creates valid database, config, and allows issue creation" {
    run "$WK_BIN" init --prefix prj
    assert_success

    # Valid SQLite database
    run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table';"
    assert_success

    # Has required tables
    local tables=(issues deps labels notes events)
    for table in "${tables[@]}"; do
        run sqlite3 .wok/issues.db "SELECT name FROM sqlite_master WHERE type='table' AND name='$table';"
        assert_success
        [ "$output" = "$table" ]
    done

    # Empty database - no issues
    run "$WK_BIN" list
    assert_success
    refute_output --regexp '\[task\]|\[bug\]|\[feature\]'

    # Config.toml is valid TOML
    grep -qE '^prefix = "[a-z]+"' .wok/config.toml
    local line_count
    line_count=$(grep -cE '^[a-z]' .wok/config.toml || echo 0)
    [ "$line_count" -ge 1 ]

    # Allows immediate issue creation with correct prefix
    rm -rf .wok
    run "$WK_BIN" init --prefix myprj
    assert_success
    run "$WK_BIN" new task "Test issue"
    assert_success
    assert_output --regexp 'myprj-[a-z0-9]+'
}

@test "init with --workspace" {
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace
    assert_success
    [ -f ".wok/config.toml" ]
    [ ! -f ".wok/issues.db" ]
    grep -q 'workspace = "/tmp/workspace"' .wok/config.toml
    ! grep -q '^prefix' .wok/config.toml
    rm -rf .wok

    # With both workspace and prefix
    run "$WK_BIN" init --workspace /tmp/workspace --prefix prj
    assert_success
    grep -q 'workspace = "/tmp/workspace"' .wok/config.toml
    grep -q 'prefix = "prj"' .wok/config.toml
    [ ! -f ".wok/issues.db" ]
    rm -rf .wok

    # Validates prefix if provided
    run "$WK_BIN" init --workspace /tmp/workspace --prefix ABC
    assert_failure
    rm -rf .wok

    # Accepts relative path
    mkdir -p external/workspace
    run "$WK_BIN" init --workspace external/workspace
    assert_success
    grep -q 'workspace = "external/workspace"' .wok/config.toml
    rm -rf .wok

    # At specific --path
    mkdir -p subdir subdir/external/workspace
    run "$WK_BIN" init --path subdir --workspace external/workspace
    assert_success
    [ -d "subdir/.wok" ]
    grep -q 'workspace = "external/workspace"' subdir/.wok/config.toml

    # Fails if workspace does not exist
    rm -rf .wok subdir/.wok
    run "$WK_BIN" init --workspace /nonexistent/path
    assert_failure
    assert_output --partial "workspace not found"

    run "$WK_BIN" init --workspace ./nonexistent/dir
    assert_failure
    assert_output --partial "workspace not found"
}

@test "init creates .gitignore with correct entries" {
    run "$WK_BIN" init --prefix prj
    assert_success
    [ -f ".wok/.gitignore" ]
    grep -q "current/" .wok/.gitignore
    grep -q "issues.db" .wok/.gitignore
    # Default (remote mode) does not ignore config.toml
    ! grep -q "config.toml" .wok/.gitignore
    rm -rf .wok

    # --local mode ignores config.toml
    run "$WK_BIN" init --prefix prj --local
    assert_success
    grep -q "config.toml" .wok/.gitignore
    rm -rf .wok

    # --workspace mode ignores config.toml
    mkdir -p /tmp/workspace
    run "$WK_BIN" init --workspace /tmp/workspace
    assert_success
    [ -f ".wok/.gitignore" ]
    grep -q "current/" .wok/.gitignore
    grep -q "issues.db" .wok/.gitignore
    grep -q "config.toml" .wok/.gitignore
}

@test "init with git remote creates worktree and supports sync" {
    run timeout 3 git init
    assert_success
    run timeout 3 "$WK_BIN" init --prefix prj --remote .
    assert_success
    [ -d ".git/wk/oplog" ]
    [ -f ".git/wk/oplog/oplog.jsonl" ]

    # Creates orphan branch
    run timeout 3 git rev-parse --verify refs/heads/wk/oplog
    assert_success

    # Worktree protects branch from deletion
    run timeout 3 git branch -D wk/oplog
    assert_failure
    assert_output --partial "worktree"

    # Remote sync works with .git/wk/oplog worktree
    run timeout 3 "$WK_BIN" new task "Test issue"
    assert_success
    run timeout 3 "$WK_BIN" remote sync
    assert_success
}
