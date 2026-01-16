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

# Basic link creation

@test "link: adds github issue link" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    assert_success
    assert_output --partial "Added link"
}

@test "link: detects github from URL" {
    id=$(create_issue task "Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    run "$WK_BIN" show "$id"
    assert_output --partial "[github]"
    assert_output --partial "github.com"
}

@test "link: detects jira from atlassian URL" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/browse/PE-5555"
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "[jira]"
    assert_output --partial "PE-5555"
}

@test "link: detects gitlab from URL" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://gitlab.com/org/project/issues/456"
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "[gitlab]"
}

@test "link: detects confluence (has /wiki in path)" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/wiki/spaces/DOC/pages/123"
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "[confluence]"
    refute_output --partial "[jira]"
}

@test "link: accepts jira:// shorthand" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "jira://PE-5555"
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "PE-5555"
}

@test "link: accepts unknown URL" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://example.com/issue/123"
    assert_success
    # Unknown URLs are still added, just without type detection
    run "$WK_BIN" show "$id"
    assert_output --partial "example.com"
}

# Relation specification

@test "link: accepts --reason tracks flag" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason tracks
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "(tracks)"
}

@test "link: accepts --reason blocks flag" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason blocks
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "(blocks)"
}

@test "link: accepts -r shorthand for reason" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" -r tracks
    assert_success
}

# Import relation requirements

@test "link: import requires known provider" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://example.com/issue/123" --reason import
    assert_failure
    assert_output --partial "requires a known provider"
}

@test "link: import requires detectable ID" {
    id=$(create_issue task "Test task")
    # Confluence URLs don't have extractable IDs
    run "$WK_BIN" link "$id" "https://company.atlassian.net/wiki/spaces/DOC" --reason import
    assert_failure
    assert_output --partial "requires a detectable issue ID"
}

@test "link: import succeeds with github URL" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/456" --reason import
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "(import)"
}

@test "link: import succeeds with jira URL" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/browse/PE-1234" --reason import
    assert_success

    run "$WK_BIN" show "$id"
    assert_output --partial "(import)"
}

# Show integration

@test "show: displays Links section when non-empty" {
    id=$(create_issue task "Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"
}

@test "show: hides Links section when empty" {
    id=$(create_issue task "Test task")

    run "$WK_BIN" show "$id"
    refute_output --partial "Links:"
}

@test "show: displays multiple links" {
    id=$(create_issue task "Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    "$WK_BIN" link "$id" "jira://PE-5555"

    run "$WK_BIN" show "$id"
    assert_output --partial "github.com"
    assert_output --partial "PE-5555"
}

# new command --link option

@test "new: --link option adds link" {
    run "$WK_BIN" new task "Linked task" --link "https://github.com/org/repo/issues/456"
    assert_success

    # Extract ID from output
    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)

    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"
    assert_output --partial "github.com"
}

@test "new: -l shorthand works for link" {
    run "$WK_BIN" new task "Linked task" -l "https://github.com/org/repo/issues/789"
    assert_success
}

@test "new: multiple --link options add multiple links" {
    run "$WK_BIN" new task "Multi-linked" \
        --link "https://github.com/org/repo/issues/1" \
        --link "jira://PE-1234"
    assert_success

    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)

    run "$WK_BIN" show "$id"
    assert_output --partial "github.com"
    assert_output --partial "PE-1234"
}

# Error handling

@test "link: fails on nonexistent issue" {
    run "$WK_BIN" link "test-nonexistent" "https://github.com/org/repo/issues/123"
    assert_failure
}

@test "link: fails with invalid reason" {
    id=$(create_issue task "Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason invalid
    assert_failure
}

# JSON output

@test "show: json format includes links" {
    id=$(create_issue task "Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" show "$id" --format json
    assert_success
    assert_output --partial '"links":'
    assert_output --partial '"link_type": "github"'
}

# Event logging

@test "link: logs linked event" {
    id=$(create_issue task "Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" log "$id"
    assert_output --partial "linked"
}
