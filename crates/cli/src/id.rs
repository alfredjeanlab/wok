// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

/// Generate an issue ID from prefix, title, and timestamp.
/// Format: {prefix}-{hash} where hash is first 8 hex chars of SHA256(title + timestamp)
pub fn generate_id(prefix: &str, title: &str, created_at: &DateTime<Utc>) -> String {
    let input = format!("{}{}", title, created_at.to_rfc3339());
    let hash = Sha256::digest(input.as_bytes());
    let short_hash = hex::encode(&hash[..4]); // First 8 hex chars (4 bytes)
    format!("{}-{}", prefix, short_hash)
}

/// Generate a unique ID, handling collisions by appending incrementing suffix.
/// Returns the ID and whether it needed a suffix.
pub fn generate_unique_id<F>(
    prefix: &str,
    title: &str,
    created_at: &DateTime<Utc>,
    exists: F,
) -> String
where
    F: Fn(&str) -> bool,
{
    let base_id = generate_id(prefix, title, created_at);

    if !exists(&base_id) {
        return base_id;
    }

    // Handle collision with incrementing suffix
    let mut suffix = 2;
    loop {
        let id = format!("{}-{}", base_id, suffix);
        if !exists(&id) {
            return id;
        }
        suffix += 1;
    }
}

/// Validate that a prefix is valid (2+ lowercase alphanumeric with at least one letter)
pub fn validate_prefix(prefix: &str) -> bool {
    prefix.len() >= 2
        && prefix
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && prefix.chars().any(|c| c.is_ascii_lowercase())
}

#[cfg(test)]
#[path = "id_tests.rs"]
mod tests;
