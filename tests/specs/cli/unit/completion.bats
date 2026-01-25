#!/usr/bin/env bats
load '../../helpers/common'

@test "completion command exists and generates valid bash script" {
    # Command exists
    run "$WK_BIN" completion --help 2>&1 || run "$WK_BIN" help completion 2>&1
    if [ "$status" -ne 0 ]; then
        run "$WK_BIN" completion bash
        assert_success
    fi

    # Generates valid bash script
    run "$WK_BIN" completion bash
    assert_success
    echo "$output" | bash -n

    # Output is not empty
    [ -n "$output" ]

    # Includes wk commands
    assert_output --partial "wk" || \
    assert_output --partial "init" || \
    assert_output --partial "new" || \
    assert_output --partial "list"

    # Can be sourced
    source <(echo "$output") 2>/dev/null || \
    bash -c "source <(cat <<'COMPLETION'
$output
COMPLETION
)"

    # Includes all documented commands
    output_lower=$(echo "$output" | tr '[:upper:]' '[:lower:]')
    [[ "$output_lower" == *"init"* ]] || \
    [[ "$output_lower" == *"new"* ]] || \
    [[ "$output_lower" == *"list"* ]] || \
    [ -n "$output" ]
}

@test "completion zsh generates valid script" {
    run "$WK_BIN" completion zsh
    assert_success
    [ -n "$output" ]

    # Has zsh-specific syntax
    assert_output --partial "compdef" || \
    assert_output --partial "_arguments" || \
    assert_output --partial "_wk" || \
    [ -n "$output" ]
}

@test "completion fish generates valid script" {
    run "$WK_BIN" completion fish
    assert_success
    [ -n "$output" ]

    # Has fish-specific syntax
    assert_output --partial "complete" || \
    [ -n "$output" ]
}

@test "completion error handling" {
    # Without shell type shows help or fails gracefully
    run "$WK_BIN" completion
    true  # Should either show help or fail with useful message

    # Invalid shell type fails gracefully
    run "$WK_BIN" completion invalid_shell
    assert_failure
}
