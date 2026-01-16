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

# Export/import roundtrip

@test "export includes links" {
    id=$(create_issue task "Task with link")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/789"

    run "$WK_BIN" export "$BATS_FILE_TMPDIR/export.jsonl"
    assert_success

    run grep "links" "$BATS_FILE_TMPDIR/export.jsonl"
    assert_success
    assert_output --partial "github.com"
}

@test "export includes link metadata" {
    id=$(create_issue task "Task with metadata")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/123" --reason tracks

    run "$WK_BIN" export "$BATS_FILE_TMPDIR/export2.jsonl"
    assert_success

    run grep "$id" "$BATS_FILE_TMPDIR/export2.jsonl"
    assert_output --partial '"link_type":"github"'
    assert_output --partial '"rel":"tracks"'
}

@test "import wk format with links" {
    # Create and export issue with links
    id=$(create_issue task "Exportable task")
    "$WK_BIN" link "$id" "jira://TEST-123"
    "$WK_BIN" export "$BATS_FILE_TMPDIR/with_links.jsonl"

    # Import to a new project in a subdirectory
    mkdir -p "$BATS_FILE_TMPDIR/import_test"
    cd "$BATS_FILE_TMPDIR/import_test"
    "$WK_BIN" init --prefix tst

    run "$WK_BIN" import "$BATS_FILE_TMPDIR/with_links.jsonl"
    assert_success

    # Verify link was preserved
    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"
    assert_output --partial "TEST-123"

    # Return to main temp dir for subsequent tests
    cd "$BATS_FILE_TMPDIR"
}

@test "import bd format (beads) does not include links" {
    # Create bd-format JSONL (beads format doesn't have external links)
    cat > "$BATS_FILE_TMPDIR/bd_issues.jsonl" << 'EOF'
{"id":"bd-1234","title":"Imported from bd","status":"open","issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF

    run "$WK_BIN" import --format bd "$BATS_FILE_TMPDIR/bd_issues.jsonl"
    assert_success

    run "$WK_BIN" show "bd-1234"
    assert_success
    # No links section since beads format doesn't support them
    refute_output --partial "Links:"
}

# Event naming verification

@test "dep events use related/unrelated (not linked/unlinked)" {
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")

    "$WK_BIN" dep "$id1" blocks "$id2"

    run "$WK_BIN" log "$id1"
    assert_output --partial "related"
    refute_output --partial "linked"
}

@test "undep events use unrelated" {
    id1=$(create_issue task "Blocker")
    id2=$(create_issue task "Blocked")

    "$WK_BIN" dep "$id1" blocks "$id2"
    "$WK_BIN" undep "$id1" blocks "$id2"

    run "$WK_BIN" log "$id1"
    assert_output --partial "unrelated"
    refute_output --partial "unlinked"
}

@test "link events use linked" {
    id=$(create_issue task "Task")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/1"

    run "$WK_BIN" log "$id"
    assert_output --partial "linked"
}

# Multiple links on single issue

@test "issue can have multiple links" {
    id=$(create_issue task "Multi-linked issue")
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/1"
    "$WK_BIN" link "$id" "https://github.com/org/repo/issues/2"
    "$WK_BIN" link "$id" "jira://PE-1234"

    run "$WK_BIN" show "$id"
    assert_output --partial "Links:"
    # Should show all three links
    assert_output --partial "issues/1"
    assert_output --partial "issues/2"
    assert_output --partial "PE-1234"
}

# Links with combined features

@test "new with --link and --label creates both" {
    run "$WK_BIN" new task "Labeled linked task" \
        --label important \
        --link "https://github.com/org/repo/issues/100"
    assert_success

    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)

    run "$WK_BIN" show "$id"
    assert_output --partial "Labels: important"
    assert_output --partial "Links:"
    assert_output --partial "github.com"
}

@test "new with --link and --note creates both" {
    run "$WK_BIN" new task "Noted linked task" \
        --note "Initial note" \
        --link "https://github.com/org/repo/issues/200"
    assert_success

    id=$(echo "$output" | grep -oE '[a-z]+-[a-z0-9]+' | head -1)

    run "$WK_BIN" show "$id"
    assert_output --partial "Description:"
    assert_output --partial "Initial note"
    assert_output --partial "Links:"
}

# Confluence vs Jira detection

@test "confluence detection is prioritized over jira for atlassian URLs with /wiki/" {
    id=$(create_issue task "Confluence test")
    "$WK_BIN" link "$id" "https://company.atlassian.net/wiki/spaces/TEAM/pages/456789"

    run "$WK_BIN" show "$id"
    assert_output --partial "[confluence]"
    # Should NOT show jira even though it's atlassian.net
    refute_output --partial "[jira]"
}

@test "jira detection works for atlassian browse URLs" {
    id=$(create_issue task "Jira test")
    "$WK_BIN" link "$id" "https://company.atlassian.net/browse/PROJ-123"

    run "$WK_BIN" show "$id"
    assert_output --partial "[jira]"
    assert_output --partial "PROJ-123"
}
