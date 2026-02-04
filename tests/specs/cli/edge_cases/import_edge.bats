#!/usr/bin/env bats
load '../../helpers/common'

@test "import empty file succeeds" {
    touch empty.jsonl
    run "$WK_BIN" import empty.jsonl
    assert_success
}

@test "import handles multiple issues" {
    cat > multi.jsonl << 'EOF'
{"id":"test-m1","issue_type":"task","title":"Task 1","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-m2","issue_type":"task","title":"Task 2","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-m3","issue_type":"task","title":"Task 3","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import multi.jsonl
    assert_success
    run "$WK_BIN" list --all
    assert_output --partial "test-m1"
    assert_output --partial "test-m2"
    assert_output --partial "test-m3"
}

@test "import preserves labels" {
    cat > labeled.jsonl << 'EOF'
{"id":"test-labels","issue_type":"task","title":"Labeled task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["project:auth","urgent"],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import labeled.jsonl
    assert_success
    run "$WK_BIN" show test-labels
    assert_output --partial "project:auth"
    assert_output --partial "urgent"
}

@test "import preserves notes" {
    cat > noted.jsonl << 'EOF'
{"id":"test-notes","issue_type":"task","title":"Noted task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[{"id":1,"issue_id":"test-notes","status":"todo","content":"A note","created_at":"2024-01-01T00:00:00Z"}],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import noted.jsonl
    assert_success
    run "$WK_BIN" show test-notes
    assert_output --partial "A note"
}

@test "import idempotent - running twice produces same result" {
    cat > idem.jsonl << 'EOF'
{"id":"test-idem","issue_type":"task","title":"Idempotent","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    "$WK_BIN" import idem.jsonl
    run "$WK_BIN" import idem.jsonl
    assert_success
    # Count should still be 1
    count=$("$WK_BIN" list --all | grep -c "test-idem" || echo 0)
    [ "$count" -eq 1 ]
}

@test "import preserves dependencies" {
    # Create an issue first that will be referenced
    cat > deps.jsonl << 'EOF'
{"id":"test-blocker","issue_type":"task","title":"Blocker task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-blocked","issue_type":"task","title":"Blocked task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[{"from_id":"test-blocker","to_id":"test-blocked","relation":"blocks","created_at":"2024-01-01T00:00:00Z"}],"events":[]}
EOF
    run "$WK_BIN" import deps.jsonl
    assert_success
    # Verify the blocked issue shows as blocked
    run "$WK_BIN" show test-blocked
    assert_output --partial "test-blocker"
}

@test "import handles all issue types" {
    cat > types.jsonl << 'EOF'
{"id":"test-feature","issue_type":"feature","title":"A feature","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-task","issue_type":"task","title":"A task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-bug","issue_type":"bug","title":"A bug","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import types.jsonl
    assert_success
    run "$WK_BIN" show test-feature
    assert_output --partial "[feature]"
    run "$WK_BIN" show test-task
    assert_output --partial "[task]"
    run "$WK_BIN" show test-bug
    assert_output --partial "[bug]"
}

@test "import handles all status values" {
    cat > statuses.jsonl << 'EOF'
{"id":"test-todo","issue_type":"task","title":"Todo issue","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-inprog","issue_type":"task","title":"In progress issue","status":"in_progress","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-done","issue_type":"task","title":"Done issue","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-closed","issue_type":"task","title":"Closed issue","status":"closed","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import statuses.jsonl
    assert_success
    run "$WK_BIN" show test-todo
    assert_output --partial "Status: todo"
    run "$WK_BIN" show test-inprog
    assert_output --partial "Status: in_progress"
    run "$WK_BIN" show test-done
    assert_output --partial "Status: done"
    run "$WK_BIN" show test-closed
    assert_output --partial "Status: closed"
}

@test "import skips empty lines" {
    cat > withempty.jsonl << 'EOF'
{"id":"test-skip1","issue_type":"task","title":"Task 1","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}

{"id":"test-skip2","issue_type":"task","title":"Task 2","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import withempty.jsonl
    assert_success
    run "$WK_BIN" show test-skip1
    assert_success
    run "$WK_BIN" show test-skip2
    assert_success
}

@test "import beads format converts status correctly" {
    cat > beads_status.jsonl << 'EOF'
{"id":"bd-open","title":"Open issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-inprog","title":"In progress issue","status":"in_progress","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-closed","title":"Closed issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import --format bd beads_status.jsonl
    assert_success
    run "$WK_BIN" show bd-open
    assert_output --partial "Status: todo"
    run "$WK_BIN" show bd-inprog
    assert_output --partial "Status: in_progress"
    run "$WK_BIN" show bd-closed
    assert_output --partial "Status: done"
}

@test "import beads format converts types correctly" {
    cat > beads_types.jsonl << 'EOF'
{"id":"bd-task","title":"Task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-bug","title":"Bug","status":"open","priority":2,"issue_type":"bug","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-feature","title":"Feature","status":"open","priority":2,"issue_type":"feature","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-epic","title":"Epic","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import --format bd beads_types.jsonl
    assert_success
    run "$WK_BIN" show bd-task
    assert_output --partial "[task]"
    run "$WK_BIN" show bd-bug
    assert_output --partial "[bug]"
    run "$WK_BIN" show bd-feature
    assert_output --partial "[feature]"
    # Epic is preserved as epic
    run "$WK_BIN" show bd-epic
    assert_output --partial "[epic]"
}

@test "import beads format preserves labels" {
    cat > beads_labels.jsonl << 'EOF'
{"id":"bd-labels","title":"Labeled issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent","area:backend"]}
EOF
    run "$WK_BIN" import --format bd beads_labels.jsonl
    assert_success
    run "$WK_BIN" show bd-labels
    assert_output --partial "urgent"
    assert_output --partial "area:backend"
}
