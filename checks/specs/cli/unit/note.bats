#!/usr/bin/env bats
load '../../helpers/common'

@test "note adds notes to issue and appears in show output" {
    # Note adds note to issue
    id=$(create_issue task "NoteBasic Test task")
    run "$WK_BIN" note "$id" "My note"
    assert_success

    # Note appears in show output
    id=$(create_issue task "NoteBasic Show task")
    "$WK_BIN" note "$id" "Important note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Important note"

    # Multiple notes can be added and preserve order
    id=$(create_issue task "NoteBasic Multi task")
    "$WK_BIN" note "$id" "First"
    "$WK_BIN" note "$id" "Second"
    "$WK_BIN" note "$id" "Third"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "First"
    assert_output --partial "Second"
    assert_output --partial "Third"

    # Note logs event
    id=$(create_issue task "NoteBasic Log task")
    "$WK_BIN" note "$id" "My note"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "noted"

    # Note shows timestamp on metadata line
    id=$(create_issue task "NoteBasic Timestamp task")
    "$WK_BIN" note "$id" "Test note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --regexp "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}"
}

@test "note records status at time of note" {
    # Records todo status
    id=$(create_issue task "NoteStatus Todo task")
    "$WK_BIN" note "$id" "Todo note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "todo"
    assert_output --partial "Todo note"

    # Records in_progress status
    id=$(create_issue task "NoteStatus Progress task")
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "Progress note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "in_progress"
    assert_output --partial "Progress note"

    # Records done status
    id=$(create_issue task "NoteStatus Done task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" note "$id" "Done note"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Done note"
}

@test "note error handling" {
    # Note with nonexistent issue fails
    run "$WK_BIN" note "test-nonexistent" "My note"
    assert_failure

    # Note requires content
    id=$(create_issue task "NoteErr Test task")
    run "$WK_BIN" note "$id"
    assert_failure

    # Note on closed issue fails
    id=$(create_issue task "NoteErr Closed task")
    "$WK_BIN" close "$id" --reason "wontfix"
    run "$WK_BIN" note "$id" "Should fail"
    assert_failure
    assert_output --partial "cannot add notes to closed issues"

    # Note rejects -r shorthand
    id=$(create_issue task "NoteErr Shorthand task")
    "$WK_BIN" note "$id" "Original note"
    run "$WK_BIN" note "$id" -r "Replacement"
    assert_failure
    assert_output --partial "unexpected argument '-r'"
}

@test "note semantic labels by status" {
    # Note in todo shows Description label
    id=$(create_issue task "NoteSem Todo task")
    "$WK_BIN" note "$id" "Requirements and context"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "Requirements and context"

    # Note in progress shows Progress label
    id=$(create_issue task "NoteSem Progress task")
    "$WK_BIN" start "$id"
    "$WK_BIN" note "$id" "Working on implementation"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Progress:"
    assert_output --partial "Working on implementation"

    # Note in done shows Summary label
    id=$(create_issue task "NoteSem Done task")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    "$WK_BIN" note "$id" "Completed successfully"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Summary:"
    assert_output --partial "Completed successfully"
}
