// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Filter expressions for time-based issue filtering.
//!
//! This module provides a flexible filter expression syntax for filtering
//! issues by age or update time. Expressions take the form:
//!
//! ```text
//! field op value
//! ```
//!
//! # Fields
//!
//! - `age` / `created` - Time since creation (synonyms)
//! - `updated` / `activity` - Time since last update (synonyms)
//!
//! # Operators
//!
//! - `<`, `<=`, `>`, `>=`, `=`, `!=`
//!
//! # Values
//!
//! - Duration: `3d`, `1w`, `24h`, `30m`, `1M`, `1y`
//! - Date: `2024-01-01` (YYYY-MM-DD format)
//!
//! # Examples
//!
//! ```text
//! age < 3d          # Created less than 3 days ago
//! age >= 1w         # Created at least 1 week ago
//! updated < 24h     # Updated in the last 24 hours
//! activity > 7d     # Not updated in 7+ days (stale)
//! created > 2024-01-01  # Created after a specific date
//! ```

mod eval;
mod expr;
mod parser;

pub use expr::{CompareOp, FilterExpr, FilterField, FilterValue};
pub use parser::{parse_duration, parse_filter};
