#!/usr/bin/env bats
load '../../helpers/common'


# Each test needs fresh directory, so use default setup/teardown

setup() {
    TEST_DIR="$(mktemp -d)"
    cd "$TEST_DIR" || exit 1
    export HOME="$TEST_DIR"
}

teardown() {
    cd / || exit 1
    rm -rf "$TEST_DIR"
}

@test "init creates .wok directory and fails if already initialized" {
    run "$WK_BIN" init --prefix myapp --private
    assert_success
    [ -n "$output" ]
    [ -d ".wok" ]
    [ -f ".wok/config.toml" ]
    [ -f ".wok/issues.db" ]
    grep -q 'prefix = "myapp"' .wok/config.toml

    # Fails if already initialized
    run "$WK_BIN" init --prefix prj --private
    assert_failure
    [ -n "$output" ]

    # Succeeds if .wok exists but has no config.toml
    rm -rf .wok
    mkdir -p .wok
    run "$WK_BIN" init --prefix prj --private
    assert_success
    [ -f ".wok/config.toml" ]
}

@test "init with --path creates at specified location" {
    mkdir -p subdir
    run "$WK_BIN" init --path subdir --prefix sub --private
    assert_success
    [ -d "subdir/.wok" ]
    [ -f "subdir/.wok/config.toml" ]
    [ -f "subdir/.wok/issues.db" ]
    grep -q 'prefix = "sub"' subdir/.wok/config.toml

    # Creates parent directories if needed
    run "$WK_BIN" init --path nested/deep/dir --prefix prj --private
    assert_success
    [ -d "nested/deep/dir/.wok" ]

    # Fails if already initialized at path
    run "$WK_BIN" init --path subdir --prefix sub --private
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
    run "$WK_BIN" init --prefix prj --private
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
    run "$WK_BIN" init --prefix myprj --private
    assert_success
    run "$WK_BIN" new task "Test issue"
    assert_success
    assert_output --regexp 'myprj-[a-z0-9]+'
}


@test "init creates .gitignore with correct entries" {
    # User-level mode ignores config.toml (no local db)
    run "$WK_BIN" init --prefix prj
    assert_success
    [ -f ".wok/.gitignore" ]
    grep -q "config.toml" .wok/.gitignore
    ! grep -q "issues.db" .wok/.gitignore
    rm -rf .wok

    # Private mode ignores config.toml and issues.db
    run "$WK_BIN" init --prefix prj --private
    assert_success
    [ -f ".wok/.gitignore" ]
    grep -q "config.toml" .wok/.gitignore
    grep -q "issues.db" .wok/.gitignore
}


@test "init defaults to local mode without remote" {
    run "$WK_BIN" init --prefix prj
    assert_success

    # Should not have remote config
    ! grep -q "\[remote\]" .wok/config.toml
    ! grep -q "url =" .wok/config.toml
}
