#!/usr/bin/env bats
load '../../helpers/common'

@test "basic transitions: start, reopen (from in_progress), done" {
    # start transitions todo to in_progress
    id=$(create_issue task "LifeBasic Test task")
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: in_progress"

    # reopen transitions in_progress to todo (no reason needed)
    run "$WK_BIN" reopen "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: todo"

    # start again, then done
    "$WK_BIN" start "$id"
    run "$WK_BIN" done "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: done"
}

@test "close and reopen require --reason" {
    # close requires --reason
    id=$(create_issue task "LifeReason Test close")
    run "$WK_BIN" close "$id"
    assert_failure

    # close with --reason succeeds from todo
    run "$WK_BIN" close "$id" --reason "duplicate"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: closed"

    # close with --reason succeeds from in_progress
    id2=$(create_issue task "LifeReason Test close inprog")
    "$WK_BIN" start "$id2"
    run "$WK_BIN" close "$id2" --reason "abandoned"
    assert_success
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: closed"

    # reopen requires --reason from done
    id3=$(create_issue task "LifeReason Test reopen")
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    run "$WK_BIN" reopen "$id3"
    assert_failure

    # reopen with --reason succeeds from done
    run "$WK_BIN" reopen "$id3" --reason "regression found"
    assert_success
    run "$WK_BIN" show "$id3"
    assert_output --partial "Status: todo"

    # reopen with --reason succeeds from closed
    run "$WK_BIN" reopen "$id" --reason "not actually a duplicate"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: todo"
}

@test "lenient transitions succeed from any state" {
    # start on a done issue → in_progress
    id=$(create_issue task "LifeLenient done-start")
    "$WK_BIN" start "$id"
    "$WK_BIN" done "$id"
    run "$WK_BIN" start "$id"
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Status: in_progress"

    # start on a closed issue → in_progress
    id2=$(create_issue task "LifeLenient closed-start")
    "$WK_BIN" close "$id2" --reason "duplicate"
    run "$WK_BIN" start "$id2"
    assert_success
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: in_progress"

    # start on an in_progress issue → idempotent
    run "$WK_BIN" start "$id"
    assert_success

    # done on a closed issue → done (with reason)
    id3=$(create_issue task "LifeLenient closed-done")
    "$WK_BIN" close "$id3" --reason "mistake"
    run "$WK_BIN" done "$id3" --reason "actually completed"
    assert_success
    run "$WK_BIN" show "$id3"
    assert_output --partial "Status: done"

    # done on a done issue → idempotent
    run "$WK_BIN" done "$id3"
    assert_success

    # close on a done issue → closed
    id4=$(create_issue task "LifeLenient done-close")
    "$WK_BIN" start "$id4"
    "$WK_BIN" done "$id4"
    run "$WK_BIN" close "$id4" --reason "actually not needed"
    assert_success
    run "$WK_BIN" show "$id4"
    assert_output --partial "Status: closed"

    # close on a closed issue → idempotent
    run "$WK_BIN" close "$id4" --reason "still not needed"
    assert_success

    # reopen on a todo issue → idempotent
    id5=$(create_issue task "LifeLenient todo-reopen")
    run "$WK_BIN" reopen "$id5"
    assert_success

    # done from todo without reason still fails (for agents)
    id6=$(create_issue task "LifeLenient todo-done-noreason")
    run "$WK_BIN" done "$id6"
    assert_failure
}

@test "reason notes appear in correct sections" {
    # close --reason creates note in Close Reason section
    id=$(create_issue task "LifeNotes Test close")
    "$WK_BIN" close "$id" --reason "duplicate of other-123"
    run "$WK_BIN" show "$id"
    assert_success
    assert_output --partial "Close Reason:"
    assert_output --partial "duplicate of other-123"

    # done --reason from todo creates note in Summary section
    id2=$(create_issue task "LifeNotes Test done")
    "$WK_BIN" done "$id2" --reason "already completed upstream"
    run "$WK_BIN" show "$id2"
    assert_success
    assert_output --partial "Summary:"
    assert_output --partial "already completed upstream"

    # reopen --reason creates note in Description section
    id3=$(create_issue task "LifeNotes Test reopen")
    "$WK_BIN" start "$id3"
    "$WK_BIN" done "$id3"
    "$WK_BIN" reopen "$id3" --reason "regression found in v2"
    run "$WK_BIN" show "$id3"
    assert_success
    assert_output --partial "Description:"
    assert_output --partial "regression found in v2"
}

@test "batch start transitions multiple from todo to in_progress" {
    id1=$(create_issue task "LifeBatch Task 1")
    id2=$(create_issue task "LifeBatch Task 2")
    run "$WK_BIN" start "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: in_progress"
}

@test "batch reopen transitions multiple from in_progress to todo" {
    id1=$(create_issue task "LifeBatchReopen Task 1")
    id2=$(create_issue task "LifeBatchReopen Task 2")
    "$WK_BIN" start "$id1"
    "$WK_BIN" start "$id2"
    run "$WK_BIN" reopen "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: todo"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: todo"
}

@test "batch done transitions multiple from in_progress to done" {
    id1=$(create_issue task "LifeBatchDone Task 1")
    id2=$(create_issue task "LifeBatchDone Task 2")
    "$WK_BIN" start "$id1"
    "$WK_BIN" start "$id2"
    run "$WK_BIN" done "$id1" "$id2"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: done"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: done"
}

@test "batch done with --reason transitions multiple from todo to done" {
    id1=$(create_issue task "LifeBatchDoneReason Task 1")
    id2=$(create_issue task "LifeBatchDoneReason Task 2")
    run "$WK_BIN" done "$id1" "$id2" --reason "already completed"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: done"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: done"
}

@test "batch close and reopen with --reason" {
    # Close multiple
    id1=$(create_issue task "LifeBatchClose Task 1")
    id2=$(create_issue task "LifeBatchClose Task 2")
    run "$WK_BIN" close "$id1" "$id2" --reason "duplicate"
    assert_success
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: closed"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: closed"

    # Reopen multiple
    id3=$(create_issue task "LifeBatchReopen2 Task 1")
    id4=$(create_issue task "LifeBatchReopen2 Task 2")
    "$WK_BIN" start "$id3"
    "$WK_BIN" start "$id4"
    "$WK_BIN" done "$id3"
    "$WK_BIN" done "$id4"
    run "$WK_BIN" reopen "$id3" "$id4" --reason "regression"
    assert_success
    run "$WK_BIN" show "$id3"
    assert_output --partial "Status: todo"
    run "$WK_BIN" show "$id4"
    assert_output --partial "Status: todo"
}

@test "batch start with already-started is idempotent" {
    id1=$(create_issue task "LifeBatchFail Task 1")
    id2=$(create_issue task "LifeBatchFail Task 2")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" start "$id1" "$id2"
    assert_success
    assert_output --partial "Started 2 of 2"
    # both should be in_progress
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"
    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: in_progress"
}

@test "batch start with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialBatch Task 1")
    run "$WK_BIN" start "$id1" "unknown-123" "unknown-456"
    assert_failure
    assert_output --partial "Started 1 of 3"
    assert_output --partial "Unknown IDs: unknown-123, unknown-456"

    # Verify the valid issue was still transitioned
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"
}

@test "batch start with mixed unknown and already-started" {
    id1=$(create_issue task "PartialMixed Task 1")
    id2=$(create_issue task "PartialMixed Task 2")
    "$WK_BIN" start "$id1"  # Now in_progress, start again is idempotent

    run "$WK_BIN" start "$id1" "$id2" "unknown-789"
    assert_failure
    assert_output --partial "Started 2 of 3"
    assert_output --partial "Unknown IDs: unknown-789"
}

@test "batch done with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialDone Task 1")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" done "$id1" "unknown-123"
    assert_failure
    assert_output --partial "Completed 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch close with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialClose Task 1")
    run "$WK_BIN" close "$id1" "unknown-123" --reason "duplicate"
    assert_failure
    assert_output --partial "Closed 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch reopen with unknown IDs performs partial update" {
    id1=$(create_issue task "PartialReopen Task 1")
    "$WK_BIN" start "$id1"
    run "$WK_BIN" reopen "$id1" "unknown-123"
    assert_failure
    assert_output --partial "Reopened 1 of 2"
    assert_output --partial "Unknown IDs: unknown-123"
}

@test "batch operation all unknown IDs shows all as unknown" {
    run "$WK_BIN" start "unknown-1" "unknown-2" "unknown-3"
    assert_failure
    assert_output --partial "Started 0 of 3"
    assert_output --partial "Unknown IDs: unknown-1, unknown-2, unknown-3"
}

@test "batch operation with single ID preserves current behavior" {
    # Single unknown ID should show simple error, no summary
    run "$WK_BIN" start "unknown-single"
    assert_failure
    assert_output --partial "issue not found: unknown-single"
    refute_output --partial "Started 0 of 1"
}
