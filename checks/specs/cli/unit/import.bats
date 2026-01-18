#!/usr/bin/env bats
load '../../helpers/common'

@test "import from file, stdin, and --input flag" {
    # Import from file
    cat > import.jsonl << 'EOF'
{"id":"test-imp1","issue_type":"task","title":"Imported task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import import.jsonl
    assert_success
    run "$WK_BIN" show test-imp1
    assert_success
    assert_output --partial "Imported task"

    # Import from stdin
    echo '{"id":"test-std1","issue_type":"task","title":"Stdin task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}' | \
        "$WK_BIN" import -
    run "$WK_BIN" show test-std1
    assert_success
    assert_output --partial "Stdin task"

    # --input flag
    cat > import2.jsonl << 'EOF'
{"id":"test-iflag","issue_type":"task","title":"Flag task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --input import2.jsonl
    assert_success
}

@test "import updates existing issues and detects collisions" {
    # Updates existing issues
    id=$(create_issue task "Original title")
    cat > import.jsonl << EOF
{"id":"$id","issue_type":"task","title":"Updated title","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import import.jsonl
    assert_success
    run "$WK_BIN" show "$id"
    assert_output --partial "Updated title"

    # Detects collisions
    id2=$(create_issue task "Original")
    "$WK_BIN" start "$id2"
    cat > import2.jsonl << EOF
{"id":"$id2","issue_type":"task","title":"Original","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import import2.jsonl
    assert_success
    assert_output --partial "collision"
}

@test "import --dry-run shows preview without creating" {
    cat > import.jsonl << 'EOF'
{"id":"test-dry1","issue_type":"task","title":"Dry run task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --dry-run import.jsonl
    assert_success
    assert_output --partial "create"
    run "$WK_BIN" show test-dry1
    assert_failure
}

@test "import warns about missing dependencies" {
    cat > import.jsonl << 'EOF'
{"id":"test-dep1","issue_type":"task","title":"Task with deps","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[{"from_id":"test-dep1","to_id":"nonexistent-123","relation":"blocks","created_at":"2024-01-01T00:00:00Z"}],"events":[]}
EOF
    run "$WK_BIN" import import.jsonl
    assert_success
    assert_output --partial "warning"
    assert_output --partial "nonexistent"
}

@test "import --status, --type, --label, --prefix filters" {
    # --status filter
    cat > import.jsonl << 'EOF'
{"id":"test-filt1","issue_type":"task","title":"Todo task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-filt2","issue_type":"task","title":"Done task","status":"done","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --status todo import.jsonl
    assert_success
    run "$WK_BIN" show test-filt1
    assert_success
    run "$WK_BIN" show test-filt2
    assert_failure

    # --type filter
    cat > import2.jsonl << 'EOF'
{"id":"test-type1","issue_type":"task","title":"Task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"test-type2","issue_type":"bug","title":"Bug","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --type task import2.jsonl
    assert_success
    run "$WK_BIN" show test-type1
    assert_success
    run "$WK_BIN" show test-type2
    assert_failure

    # --label filter
    cat > import3.jsonl << 'EOF'
{"id":"test-label1","issue_type":"task","title":"Labeled","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":["urgent"],"notes":[],"deps":[],"events":[]}
{"id":"test-label2","issue_type":"task","title":"Unlabeled","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --label urgent import3.jsonl
    assert_success
    run "$WK_BIN" show test-label1
    assert_success
    run "$WK_BIN" show test-label2
    assert_failure

    # --prefix filter
    cat > import4.jsonl << 'EOF'
{"id":"myproj-a1","issue_type":"task","title":"My project task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
{"id":"other-b2","issue_type":"task","title":"Other project task","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import --prefix myproj import4.jsonl
    assert_success
    run "$WK_BIN" show myproj-a1
    assert_success
    run "$WK_BIN" show other-b2
    assert_failure
}

@test "import auto-detects bd format and --format bd parses beads format" {
    # Auto-detects bd format from .beads/issues.jsonl path
    mkdir -p .beads
    cat > .beads/issues.jsonl << 'EOF'
{"id":"bd-auto1","title":"Beads issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import .beads/issues.jsonl
    assert_success
    run "$WK_BIN" show bd-auto1
    assert_success

    # --format bd parses beads format explicitly
    cat > beads.jsonl << 'EOF'
{"id":"bd-fmt1","title":"Beads task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import --format bd beads.jsonl
    assert_success
    run "$WK_BIN" show bd-fmt1
    assert_success
}

@test "import fails on invalid JSON and shows help with no input" {
    echo "not valid json" > invalid.jsonl
    run "$WK_BIN" import invalid.jsonl
    assert_failure
    assert_output --partial "error"

    run "$WK_BIN" import
    assert_failure
}

@test "import beads format converts dependency types" {
    # blocks dependency
    cat > bd_deps.jsonl << 'EOF'
{"id":"bd-blocker","title":"Blocker","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-blocked","title":"Blocked","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-blocker","type":"blocks"}]}
EOF
    run "$WK_BIN" import --format bd bd_deps.jsonl
    assert_success
    run "$WK_BIN" show bd-blocked
    assert_output --partial "Blocked by:"
    assert_output --partial "bd-blocker"

    # parent dependency
    cat > bd_parent.jsonl << 'EOF'
{"id":"bd-child-p","title":"Child","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-parent-p","title":"Parent","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-child-p","type":"parent"}]}
EOF
    run "$WK_BIN" import --format bd bd_parent.jsonl
    assert_success
    run "$WK_BIN" show bd-parent-p
    assert_output --partial "Tracks:"
    assert_output --partial "bd-child-p"

    # parent-child dependency
    cat > bd_child.jsonl << 'EOF'
{"id":"bd-parent-c","title":"Parent Epic","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-child-c","title":"Child Task","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-parent-c","type":"parent-child"}]}
EOF
    run "$WK_BIN" import --format bd bd_child.jsonl
    assert_success
    run "$WK_BIN" show bd-child-c
    assert_output --partial "Tracked by:"
    assert_output --partial "bd-parent-c"

    # tracks dependency
    cat > bd_tracks.jsonl << 'EOF'
{"id":"bd-tracked","title":"Tracked","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
{"id":"bd-tracker","title":"Tracker","status":"open","priority":2,"issue_type":"epic","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","dependencies":[{"depends_on_id":"bd-tracked","type":"tracks"}]}
EOF
    run "$WK_BIN" import --format bd bd_tracks.jsonl
    assert_success
    run "$WK_BIN" show bd-tracker
    assert_output --partial "Tracks:"
    assert_output --partial "bd-tracked"
}

@test "import beads close reason maps to status and creates note" {
    # closed with failure reason maps to closed status
    cat > bd_fail.jsonl << 'EOF'
{"id":"bd-fail","title":"Failed issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"abandoned due to lack of resources"}
EOF
    run "$WK_BIN" import --format bd bd_fail.jsonl
    assert_success
    run "$WK_BIN" show bd-fail
    assert_output --partial "Status: closed"

    # closed without failure reason maps to done status
    cat > bd_success.jsonl << 'EOF'
{"id":"bd-success","title":"Successful issue","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"Completed successfully"}
EOF
    run "$WK_BIN" import --format bd bd_success.jsonl
    assert_success
    run "$WK_BIN" show bd-success
    assert_output --partial "Status: done"

    # close_reason creates close event and note
    cat > bd_reason.jsonl << 'EOF'
{"id":"bd-reason","title":"Issue with reason","status":"closed","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","close_reason":"duplicate of bd-other"}
EOF
    run "$WK_BIN" import --format bd bd_reason.jsonl
    assert_success
    run "$WK_BIN" show bd-reason
    assert_output --partial "Close Reason:"
    assert_output --partial "duplicate of bd-other"
}

@test "import beads format converts priority and preserves comments" {
    # Priority converts to label
    cat > bd_prio.jsonl << 'EOF'
{"id":"bd-prio","title":"Priority issue","status":"open","priority":1,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import --format bd bd_prio.jsonl
    assert_success
    run "$WK_BIN" show bd-prio
    assert_output --partial "priority:1"

    # Does not add priority:0 label
    cat > bd_prio0.jsonl << 'EOF'
{"id":"bd-prio0","title":"No priority issue","status":"open","priority":0,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}
EOF
    run "$WK_BIN" import --format bd bd_prio0.jsonl
    assert_success
    run "$WK_BIN" show bd-prio0
    refute_output --partial "priority:0"

    # Preserves comments using text field
    cat > bd_comment.jsonl << 'EOF'
{"id":"bd-comment","title":"Commented issue","status":"open","priority":2,"issue_type":"task","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","comments":[{"text":"This is a comment from beads","created_at":"2024-01-01T00:00:00Z"}]}
EOF
    run "$WK_BIN" import --format bd bd_comment.jsonl
    assert_success
    run "$WK_BIN" show bd-comment
    assert_output --partial "Description:"
    assert_output --partial "This is a comment from beads"
}

@test "import rejects -i and -p shorthands" {
    cat > import.jsonl << 'EOF'
{"id":"test-i","issue_type":"task","title":"Test","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import -i import.jsonl
    assert_failure
    assert_output --partial "unexpected argument '-i'"

    cat > import2.jsonl << 'EOF'
{"id":"myproj-test","issue_type":"task","title":"Test","status":"todo","created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z","labels":[],"notes":[],"deps":[],"events":[]}
EOF
    run "$WK_BIN" import -p myproj import2.jsonl
    assert_failure
    assert_output --partial "unexpected argument '-p'"
}
