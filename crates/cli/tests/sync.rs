// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

mod common;
use common::*;

#[test]
fn remote_status_local_mode() {
    let temp = init_temp_local();

    // In local mode, remote status should say "not applicable"
    wk().arg("remote")
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("not applicable"))
        .stdout(predicate::str::contains("no remote configured"));
}

#[test]
fn remote_status_remote_mode() {
    let temp = init_temp_remote(); // Explicitly initialize with remote mode

    // In remote mode, remote status should show remote URL
    wk().arg("remote")
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote: git:."));
}

#[test]
fn remote_sync_local_mode() {
    let temp = init_temp_local();

    // In local mode, remote sync should be silent (nothing to sync)
    wk().arg("remote")
        .arg("sync")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
