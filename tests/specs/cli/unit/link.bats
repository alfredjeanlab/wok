#!/usr/bin/env bats
load '../../helpers/common'

@test "link detects provider type from URL" {
    # GitHub
    id=$(create_issue task "LinkDetect Test github")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    assert_success
    assert_output --partial "Added link"
    run "$WK_BIN" show "$id"
    assert_output --partial "[github]"
    assert_output --partial "github.com"

    # Jira from atlassian URL
    id=$(create_issue task "LinkDetect Test jira")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/browse/PE-5555"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[jira]"
    assert_output --partial "PE-5555"

    # GitLab
    id=$(create_issue task "LinkDetect Test gitlab")
    run "$WK_BIN" link "$id" "https://gitlab.com/org/project/issues/456"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[gitlab]"

    # Confluence (has /wiki in path)
    id=$(create_issue task "LinkDetect Test confluence")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/wiki/spaces/DOC/pages/123"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "[confluence]"
    refute_output --partial "[jira]"

    # Jira shorthand
    id=$(create_issue task "LinkDetect Test jira shorthand")
    run "$WK_BIN" link "$id" "jira://PE-5555"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "PE-5555"

    # Unknown URL
    id=$(create_issue task "LinkDetect Test unknown")
    run "$WK_BIN" link "$id" "https://example.com/issue/123"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "example.com"
}

@test "link --reason flags (tracks, blocks) and -r shorthand" {
    # --reason tracks
    id=$(create_issue task "LinkReason Test tracks")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason tracks
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "(tracks)"

    # --reason blocks
    id=$(create_issue task "LinkReason Test blocks")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason blocks
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "(blocks)"

    # -r shorthand
    id=$(create_issue task "LinkReason Test shorthand")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" -r tracks
    assert_success
}

@test "link import requires known provider and detectable ID" {
    # Requires known provider
    id=$(create_issue task "LinkImport Test unknown")
    run "$WK_BIN" link "$id" "https://example.com/issue/123" --reason import
    assert_failure
    assert_output --partial "requires a known provider"

    # Requires detectable ID (Confluence URLs don't have extractable IDs)
    id=$(create_issue task "LinkImport Test no id")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/wiki/spaces/DOC" --reason import
    assert_failure
    assert_output --partial "requires a detectable issue ID"

    # Succeeds with github URL
    id=$(create_issue task "LinkImport Test github")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/456" --reason import
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "(import)"

    # Succeeds with jira URL
    id=$(create_issue task "LinkImport Test jira")
    run "$WK_BIN" link "$id" "https://company.atlassian.net/browse/PE-1234" --reason import
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "(import)"
}

@test "show displays Links section when non-empty and hides when empty" {
    id=$(create_issue task "LinkShow Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"

    id2=$(create_issue task "LinkShow Test empty")
    run "$WK_BIN" show "$id2"
    refute_output --partial "Links:"

    # Multiple links
    id3=$(create_issue task "LinkShow Test multiple")
    "$WK_BIN" link "$id3" "https://github.com/org/repo/issues/123"
    "$WK_BIN" link "$id3" "jira://PE-5555"
    run "$WK_BIN" show "$id3"
    assert_output --partial "github.com"
    assert_output --partial "PE-5555"
}

@test "new --link option adds link (including -l shorthand and multiple)" {
    # --link adds link
    run "$WK_BIN" new task "LinkNew Linked task" --link "https://github.com/org/repo/issues/456"
    assert_success
    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)
    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"
    assert_output --partial "github.com"

    # -l shorthand
    run "$WK_BIN" new task "LinkNew Linked shorthand" -l "https://github.com/org/repo/issues/789"
    assert_success

    # Multiple --link
    run "$WK_BIN" new task "LinkNew Multi-linked" \
        --link "https://github.com/org/repo/issues/1" \
        --link "jira://PE-1234"
    assert_success
    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)
    run "$WK_BIN" show "$id"
    assert_output --partial "github.com"
    assert_output --partial "PE-1234"
}

@test "link error handling: nonexistent issue and invalid reason" {
    run "$WK_BIN" link "test-nonexistent" "https://github.com/org/repo/issues/123"
    assert_failure

    id=$(create_issue task "LinkError Test task")
    run "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason invalid
    assert_failure
}

@test "show json format includes links and log shows linked event" {
    id=$(create_issue task "LinkJSON Test task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" show "$id" --output json
    assert_success
    assert_output --partial '"links":'
    assert_output --partial '"link_type":"github"'

    run "$WK_BIN" log "$id"
    assert_output --partial "linked"
}

@test "unlink removes a link from an issue" {
    id=$(create_issue task "Unlink Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"

    run "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/123"
    assert_success
    assert_output --partial "Removed link"

    run "$WK_BIN" show "$id"
    refute_output --partial "Links:"
}

@test "unlink with nonexistent URL succeeds with message" {
    id=$(create_issue task "Unlink Nonexistent Test")

    run "$WK_BIN" unlink "$id" "https://example.com/not-linked"
    assert_success
    assert_output --partial "not found"
}

@test "unlink with nonexistent issue fails" {
    run "$WK_BIN" unlink "test-nonexistent" "https://github.com/org/repo/issues/123"
    assert_failure
}

@test "unlink removes only the specified link" {
    id=$(create_issue task "Unlink Multiple Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/1"
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/2"

    run "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/1"
    assert_success

    # Check that only issues/2 appears in Links section (issues/1 may still appear in log)
    run "$WK_BIN" show "$id" --output json
    assert_success
    # The links array should only have issues/2
    [[ "$(echo "$output" | grep -o '"url":"https://github.com/org/repo/issues/[0-9]"' | wc -l)" -eq 1 ]]
    assert_output --partial '"url":"https://github.com/org/repo/issues/2"'
    refute_output --partial '"url":"https://github.com/org/repo/issues/1"'
}

@test "log shows unlinked event" {
    id=$(create_issue task "Unlink Log Test")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123"
    "$WK_BIN" unlink "$id" "https://github.com/org/repo/issues/123"

    run "$WK_BIN" log "$id"
    assert_output --partial "unlinked"
}
