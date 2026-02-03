// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR")?;
    let path = std::path::Path::new(&out_dir).join("env_vars.rs");
    let mut f = std::fs::File::create(path)?;

    let vars = [
        ("WK_TIMINGS", "WK_TIMINGS"),
        ("NO_COLOR", "NO_COLOR"),
        ("COLOR", "COLOR"),
        ("WOK_STATE_DIR", "WOK_STATE_DIR"),
        ("XDG_STATE_HOME", "XDG_STATE_HOME"),
        ("WOK_DAEMON_BINARY", "WOK_DAEMON_BINARY"),
    ];

    for (const_name, env_name) in &vars {
        writeln!(f, "pub const {const_name}: &str = \"{env_name}\";")?;
    }

    Ok(())
}
