#!/usr/bin/env bats
load '../../helpers/common'

@test "identical titles get unique IDs" {
    init_project
    id1=$(create_issue task "Duplicate title")
    id2=$(create_issue task "Duplicate title")

    [ -n "$id1" ]
    [ -n "$id2" ]
    [ "$id1" != "$id2" ]
}

@test "collision suffix increments" {
    init_project
    id1=$(create_issue task "Same title")
    id2=$(create_issue task "Same title")
    id3=$(create_issue task "Same title")

    # All should be unique
    [ "$id1" != "$id2" ]
    [ "$id2" != "$id3" ]
    [ "$id1" != "$id3" ]
}

@test "all collided IDs are valid" {
    init_project
    id1=$(create_issue task "Collision test")
    id2=$(create_issue task "Collision test")

    run "$WK_BIN" show "$id1"
    assert_success

    run "$WK_BIN" show "$id2"
    assert_success
}

@test "collided IDs can be used independently" {
    init_project
    id1=$(create_issue task "Same name")
    id2=$(create_issue task "Same name")

    "$WK_BIN" start "$id1"

    # id1 should be in_progress, id2 should still be todo
    run "$WK_BIN" show "$id1"
    assert_output --partial "Status: in_progress"

    run "$WK_BIN" show "$id2"
    assert_output --partial "Status: todo"
}

@test "many collisions handled" {
    init_project
    local ids=()
    for i in {1..10}; do
        id=$(create_issue task "Repeated title")
        ids+=("$id")
    done

    # All should be unique
    local unique_count
    unique_count=$(printf '%s\n' "${ids[@]}" | sort -u | wc -l)
    [ "$unique_count" -eq 10 ]
}

@test "different types with same title get unique IDs" {
    init_project
    id1=$(create_issue task "Multi-type")
    id2=$(create_issue bug "Multi-type")
    id3=$(create_issue feature "Multi-type")

    [ "$id1" != "$id2" ]
    [ "$id2" != "$id3" ]
    [ "$id1" != "$id3" ]
}
