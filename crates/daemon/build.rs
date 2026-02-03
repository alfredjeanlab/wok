// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = match env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("cargo:warning=OUT_DIR not set: {e}");
            std::process::exit(1);
        }
    };
    let dest_path = Path::new(&out_dir).join("env_names.rs");

    let mut file = match fs::File::create(&dest_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("cargo:warning=failed to create env_names.rs: {e}");
            std::process::exit(1);
        }
    };

    let contents = r#"/// Environment variable: override the wok state directory.
pub const WOK_STATE_DIR: &str = "WOK_STATE_DIR";

/// Environment variable: XDG base directory for state data.
pub const XDG_STATE_HOME: &str = "XDG_STATE_HOME";

/// Environment variable: controls log level filtering (used by tracing-subscriber).
pub const RUST_LOG: &str = "RUST_LOG";
"#;

    if let Err(e) = file.write_all(contents.as_bytes()) {
        eprintln!("cargo:warning=failed to write env_names.rs: {e}");
        std::process::exit(1);
    }
}
