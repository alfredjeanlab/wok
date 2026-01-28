#!/usr/bin/env bats
load '../helpers/remote_common'

setup() {
    setup_remote
}

teardown() {
    teardown_remote
}

@test "find_free_port returns valid port number" {
    local port
    port=$(find_free_port)
    [[ "$port" =~ ^[0-9]+$ ]]
    [ "$port" -ge 17800 ]
    [ "$port" -le 18999 ]
}

@test "start_server and stop_server lifecycle" {
    require_wk_remote_bin || skip "wk-remote not available"

    start_server
    [ -n "$SERVER_PID" ]
    [ -n "$SERVER_PORT" ]
    [ -n "$SERVER_URL" ]

    # Verify server is running
    nc -z 127.0.0.1 "$SERVER_PORT"

    stop_server

    # Verify server stopped
    sleep 0.1
    ! nc -z 127.0.0.1 "$SERVER_PORT" 2>/dev/null || true
}
