#!/usr/bin/env bats
load '../../helpers/common'

@test "title with double quotes" {
    run "$WK_BIN" new 'Fix the "bug" issue'
    assert_success
    assert_output --partial '"bug"'
}

@test "title with single quotes" {
    run "$WK_BIN" new "User's profile page"
    assert_success
    assert_output --partial "User's"
}

@test "title with backticks" {
    run "$WK_BIN" new 'Fix \`code\` formatting'
    assert_success
}

@test "title with unicode emoji" {
    run "$WK_BIN" new "Fix emoji handling 🚀"
    assert_success
    assert_output --partial "🚀"
}

@test "title with unicode characters" {
    run "$WK_BIN" new "Café résumé naïve"
    assert_success
    assert_output --partial "Café"
}

@test "title with CJK characters" {
    run "$WK_BIN" new "日本語タイトル"
    assert_success
    assert_output --partial "日本語"
}

@test "title with special shell characters" {
    run "$WK_BIN" new 'Task with $dollar and &ampersand'
    assert_success
}

@test "title with parentheses and brackets" {
    run "$WK_BIN" new "Fix (critical) [bug] issue"
    assert_success
    assert_output --partial "(critical)"
    assert_output --partial "[bug]"
}

@test "note with newlines preserved" {
    id=$(create_issue task "Test")
    run "$WK_BIN" note "$id" $'Line 1\nLine 2\nLine 3'
    assert_success
}

@test "note with special characters" {
    id=$(create_issue task "Test")
    run "$WK_BIN" note "$id" 'Note with "quotes" and '\''apostrophes'\'''
    assert_success
}

@test "label with colon" {
    id=$(create_issue task "Test")
    run "$WK_BIN" label "$id" "namespace:value"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "namespace:value"
}

@test "title with leading/trailing whitespace" {
    run "$WK_BIN" new "  Trimmed title  "
    assert_success
}

@test "very long title" {
    local long_title
    long_title=$(printf 'x%.0s' {1..500})
    run "$WK_BIN" new "$long_title"
    assert_success
}

@test "title with only numbers" {
    run "$WK_BIN" new "12345"
    assert_success
}

@test "hyphen in title" {
    run "$WK_BIN" new "Fix foo-bar-baz issue"
    assert_success
}
