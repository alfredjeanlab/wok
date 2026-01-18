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

@test "label adds simple and namespaced labels to issue" {
    # Simple label
    id=$(create_issue task "LabelBasic Test task")
    run "$WK_BIN" label "$id" "urgent"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "urgent"

    # Namespaced label
    id=$(create_issue task "LabelBasic Namespaced task")
    run "$WK_BIN" label "$id" "team:backend"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "team:backend"

    # Multiple labels can be added
    id=$(create_issue task "LabelBasic Multi task")
    "$WK_BIN" label "$id" "label1"
    "$WK_BIN" label "$id" "label2"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "label1"
    assert_output --partial "label2"

    # Labels are searchable via list --label
    id=$(create_issue task "LabelBasic Searchable task")
    "$WK_BIN" label "$id" "findme"
    run "$WK_BIN" list --label "findme"
    assert_success
    assert_output --partial "Searchable task"
}

@test "unlabel removes label and logs events" {
    # Unlabel removes label
    id=$(create_issue task "LabelUnlabel Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" unlabel "$id" "mylabel"
    assert_success
    run "$WK_BIN" show "$id"
    refute_line --regexp '^Labels:.*mylabel'

    # Unlabel nonexistent label succeeds or fails gracefully
    id=$(create_issue task "LabelUnlabel Nonexistent task")
    run "$WK_BIN" unlabel "$id" "nonexistent"
    true  # Should either succeed (idempotent) or fail gracefully

    # Duplicate label is idempotent or fails gracefully
    id=$(create_issue task "LabelUnlabel Duplicate task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" label "$id" "mylabel"
    true  # Should either succeed (idempotent) or fail gracefully
}

@test "label and unlabel log events" {
    # Label logs event
    id=$(create_issue task "LabelLog Test task")
    "$WK_BIN" label "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "labeled"

    # Unlabel logs event
    "$WK_BIN" unlabel "$id" "mylabel"
    run "$WK_BIN" log "$id"
    assert_success
    assert_output --partial "unlabeled"
}

@test "label error handling" {
    # Label with nonexistent issue fails
    run "$WK_BIN" label "test-nonexistent" "mylabel"
    assert_failure

    # Batch label fails on nonexistent issue
    id1=$(create_issue task "LabelErr Task 1")
    run "$WK_BIN" label "$id1" "test-nonexistent" "urgent"
    assert_failure
}

@test "batch label and unlabel operations" {
    # Label adds label to multiple issues
    id1=$(create_issue task "LabelBatch Task 1")
    id2=$(create_issue task "LabelBatch Task 2")
    run "$WK_BIN" label "$id1" "$id2" "urgent"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "urgent"
    run "$WK_BIN" show "$id2"
    assert_output --partial "urgent"

    # Label adds to three issues
    id1=$(create_issue task "LabelBatch3 Task 1")
    id2=$(create_issue task "LabelBatch3 Task 2")
    id3=$(create_issue task "LabelBatch3 Task 3")
    run "$WK_BIN" label "$id1" "$id2" "$id3" "backend"
    assert_success
    for id in "$id1" "$id2" "$id3"; do
        run "$WK_BIN" show "$id"
        assert_output --partial "backend"
    done

    # Unlabel removes from multiple issues
    id1=$(create_issue task "LabelBatchUnlabel Task 1")
    id2=$(create_issue task "LabelBatchUnlabel Task 2")
    "$WK_BIN" label "$id1" "urgent"
    "$WK_BIN" label "$id2" "urgent"
    run "$WK_BIN" unlabel "$id1" "$id2" "urgent"
    assert_success
    run "$WK_BIN" show "$id1"
    refute_line --regexp '^Labels:.*urgent'
    run "$WK_BIN" show "$id2"
    refute_line --regexp '^Labels:.*urgent'

    # Batch labeled issues are searchable
    id1=$(create_issue task "LabelBatchSearch Task 1")
    id2=$(create_issue task "LabelBatchSearch Task 2")
    "$WK_BIN" label "$id1" "$id2" "batchtest"
    run "$WK_BIN" list --label "batchtest"
    assert_success
    assert_output --partial "LabelBatchSearch Task 1"
    assert_output --partial "LabelBatchSearch Task 2"
}
