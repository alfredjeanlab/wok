// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Parser for filter expressions.
//!
//! Parses expressions like `age < 3d` or `updated > 1w` into structured
//! [`FilterExpr`] values.

use chrono::{Duration, NaiveDate};

use crate::error::{Error, Result};

use super::expr::{CompareOp, FilterExpr, FilterField, FilterValue};

/// Parse a filter expression from a string.
///
/// # Examples
///
/// ```ignore
/// let expr = parse_filter("age < 3d")?;
/// let expr = parse_filter("updated >= 1w")?;
/// let expr = parse_filter("created > 2024-01-01")?;
/// ```
///
/// # Errors
///
/// Returns an error with a helpful message if the expression is invalid.
pub fn parse_filter(input: &str) -> Result<FilterExpr> {
    let input = input.trim();

    if input.is_empty() {
        return Err(Error::InvalidInput("empty filter expression".to_string()));
    }

    // Extract field name (until whitespace or operator character)
    let (field_str, rest) = split_field(input)?;
    let field = parse_field(field_str)?;

    // Extract operator
    let rest = rest.trim_start();
    let (op, rest) = parse_operator(rest)?;

    // Extract value
    let value_str = rest.trim();
    if value_str.is_empty() {
        return Err(Error::InvalidInput(format!(
            "missing value in filter expression: \"{input}\""
        )));
    }
    let value = parse_value(value_str)?;

    Ok(FilterExpr { field, op, value })
}

/// Split input into field name and rest.
fn split_field(input: &str) -> Result<(&str, &str)> {
    // Find where the field ends (at whitespace or operator character)
    let end = input
        .find(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '=' || c == '!')
        .unwrap_or(input.len());

    if end == 0 {
        return Err(Error::InvalidInput(format!(
            "missing field name in filter expression: \"{input}\""
        )));
    }

    Ok((&input[..end], &input[end..]))
}

/// Parse a field name into a FilterField.
fn parse_field(s: &str) -> Result<FilterField> {
    match s.to_lowercase().as_str() {
        "age" | "created" => Ok(FilterField::Age),
        "updated" | "activity" => Ok(FilterField::Updated),
        "closed" | "completed" | "done" => Ok(FilterField::Closed),
        _ => Err(Error::InvalidInput(format!(
            "unknown field '{s}'. Valid fields: {}",
            FilterField::valid_names()
        ))),
    }
}

/// Parse an operator from the start of the string.
fn parse_operator(s: &str) -> Result<(CompareOp, &str)> {
    // Try two-character operators first
    if s.len() >= 2 {
        match &s[..2] {
            "<=" => return Ok((CompareOp::Le, &s[2..])),
            ">=" => return Ok((CompareOp::Ge, &s[2..])),
            "!=" => return Ok((CompareOp::Ne, &s[2..])),
            // Catch invalid double operators
            "<<" | ">>" | "==" => {
                return Err(Error::InvalidInput(format!(
                    "unknown operator '{}'. Valid operators: {}",
                    &s[..2],
                    CompareOp::valid_symbols()
                )));
            }
            _ => {}
        }
    }

    // Try single-character operators
    if !s.is_empty() {
        match s.chars().next() {
            Some('<') => return Ok((CompareOp::Lt, &s[1..])),
            Some('>') => return Ok((CompareOp::Gt, &s[1..])),
            Some('=') => return Ok((CompareOp::Eq, &s[1..])),
            _ => {}
        }
    }

    // Extract what looks like an operator for error message
    let op_end = s
        .find(|c: char| c.is_whitespace() || c.is_alphanumeric())
        .unwrap_or(s.len().min(3));
    let bad_op = if op_end > 0 { &s[..op_end] } else { "(none)" };

    Err(Error::InvalidInput(format!(
        "unknown operator '{bad_op}'. Valid operators: {}",
        CompareOp::valid_symbols()
    )))
}

/// Parse a value (duration or date).
fn parse_value(s: &str) -> Result<FilterValue> {
    // Try parsing as a date first (YYYY-MM-DD format)
    if let Some(date) = try_parse_date(s) {
        return Ok(FilterValue::Date(date));
    }

    // Try parsing as a duration
    parse_duration(s).map(FilterValue::Duration)
}

/// Try to parse a date in YYYY-MM-DD format.
fn try_parse_date(s: &str) -> Option<NaiveDate> {
    // Check basic format: exactly 10 chars with dashes at positions 4 and 7
    if s.len() != 10 {
        return None;
    }
    let bytes = s.as_bytes();
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return None;
    }

    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Parse a duration string like "3d", "1w", "24h".
pub fn parse_duration(s: &str) -> Result<Duration> {
    if s.is_empty() {
        return Err(Error::InvalidInput("empty duration".to_string()));
    }

    // Split into number and unit
    let (num_str, unit) = split_number_unit(s)?;

    // Parse the number
    let num: i64 = num_str
        .parse()
        .map_err(|_| Error::InvalidInput(format!("invalid number in duration: '{num_str}'")))?;

    // Check for negative durations
    if num < 0 {
        return Err(Error::InvalidInput(
            "negative durations are not allowed".to_string(),
        ));
    }

    // Convert to Duration based on unit
    match unit {
        "ms" => Ok(Duration::milliseconds(num)),
        "s" => Ok(Duration::seconds(num)),
        "m" => Ok(Duration::minutes(num)),
        "h" => Ok(Duration::hours(num)),
        "d" => Ok(Duration::days(num)),
        "w" => Ok(Duration::weeks(num)),
        "M" => Ok(Duration::days(num.saturating_mul(30))), // Approximate month
        "y" => Ok(Duration::days(num.saturating_mul(365))), // Approximate year
        _ => Err(Error::InvalidInput(format!(
            "unknown duration unit '{unit}'. Valid units: ms, s, m, h, d, w, M, y"
        ))),
    }
}

/// Split a duration string into number and unit parts.
fn split_number_unit(s: &str) -> Result<(&str, &str)> {
    // Find where digits end
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '-')
        .unwrap_or(s.len());

    if num_end == 0 {
        return Err(Error::InvalidInput(format!(
            "duration must start with a number: '{s}'"
        )));
    }

    let num_str = &s[..num_end];
    let unit = &s[num_end..];

    if unit.is_empty() {
        return Err(Error::InvalidInput(format!(
            "duration missing unit: '{s}'. Valid units: ms, s, m, h, d, w, M, y"
        )));
    }

    Ok((num_str, unit))
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
