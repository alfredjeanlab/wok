// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Performance timing instrumentation for debugging.
//!
//! Enable with `WK_TIMINGS=1` environment variable.
//! Output goes to stderr in format: `[timings] phase::name XXms`

use std::time::Instant;

/// Check if timings are enabled via WK_TIMINGS environment variable.
#[inline]
pub fn timings_enabled() -> bool {
    std::env::var("WK_TIMINGS").is_ok()
}

/// Print a timing result to stderr if timings are enabled.
#[inline]
pub fn print_timing(phase: &str, start: Instant) {
    if timings_enabled() {
        let elapsed = start.elapsed();
        eprintln!("[timings] {} {}ms", phase, elapsed.as_millis());
    }
}

/// Macro for timing a block of code.
///
/// Usage:
/// ```rust,ignore
/// let result = time_phase!("db::open", {
///     Database::open(&path)
/// });
/// ```
#[macro_export]
macro_rules! time_phase {
    ($phase:expr, $block:expr) => {{
        let __start = std::time::Instant::now();
        let __result = $block;
        $crate::timings::print_timing($phase, __start);
        __result
    }};
}
