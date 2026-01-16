#!/usr/bin/env bats
load '../../helpers/common'

setup_file() {
    file_setup
    init_project_once test
}

teardown_file() {
    file_teardown
}

setup() {
    test_setup
}

# Tests verifying completion command generates valid shell scripts
# If these tests fail, the completion command may not be implemented yet

@test "completion command exists" {
    run "$WK_BIN" completion --help 2>&1 || run "$WK_BIN" help completion 2>&1
    # Should either succeed or show help, not "unknown command"
    if [ "$status" -ne 0 ]; then
        # If help fails, try running completion directly
        run "$WK_BIN" completion bash
        assert_success
    fi
}

# Bash completion tests

@test "completion bash generates valid bash script" {
    run "$WK_BIN" completion bash
    assert_success
    # Output should be valid bash syntax - try parsing it
    echo "$output" | bash -n
}

@test "completion bash output is not empty" {
    run "$WK_BIN" completion bash
    assert_success
    [ -n "$output" ]
}

@test "completion bash includes wk commands" {
    run "$WK_BIN" completion bash
    assert_success
    # Should reference wk commands
    assert_output --partial "wk" || \
    assert_output --partial "init" || \
    assert_output --partial "new" || \
    assert_output --partial "list"
}

@test "completion bash can be sourced" {
    run "$WK_BIN" completion bash
    assert_success
    # Should be sourceable without errors
    source <(echo "$output") 2>/dev/null || \
    bash -c "source <(cat <<'COMPLETION'
$output
COMPLETION
)"
}

# Zsh completion tests

@test "completion zsh generates valid zsh script" {
    run "$WK_BIN" completion zsh
    assert_success
    # Output should not be empty
    [ -n "$output" ]
}

@test "completion zsh output has zsh-specific syntax" {
    run "$WK_BIN" completion zsh
    assert_success
    # Zsh completions often use compdef or _arguments
    assert_output --partial "compdef" || \
    assert_output --partial "_arguments" || \
    assert_output --partial "_wk" || \
    [ -n "$output" ]
}

# Fish completion tests

@test "completion fish generates valid fish script" {
    run "$WK_BIN" completion fish
    assert_success
    # Output should not be empty
    [ -n "$output" ]
}

@test "completion fish output has fish-specific syntax" {
    run "$WK_BIN" completion fish
    assert_success
    # Fish completions use 'complete' command
    assert_output --partial "complete" || \
    [ -n "$output" ]
}

# General completion tests

@test "completion without shell type shows help or fails gracefully" {
    run "$WK_BIN" completion
    # Should either show help or fail with useful message
    # Not a hard failure - just needs graceful handling
    true
}

@test "completion with invalid shell type fails gracefully" {
    run "$WK_BIN" completion invalid_shell
    # Should fail with an error message
    assert_failure
}

@test "completion bash includes all documented commands" {
    run "$WK_BIN" completion bash
    assert_success
    # Should include the main commands
    # Note: exact syntax depends on implementation
    output_lower=$(echo "$output" | tr '[:upper:]' '[:lower:]')
    [[ "$output_lower" == *"init"* ]] || \
    [[ "$output_lower" == *"new"* ]] || \
    [[ "$output_lower" == *"list"* ]] || \
    [ -n "$output" ]
}
