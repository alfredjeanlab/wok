// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Text normalization for issue fields.
//!
//! Handles whitespace trimming, title splitting, and quote-aware
//! newline handling per REQUIREMENTS.md specification.

/// Result of normalizing a title that may need splitting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedTitle {
    /// The normalized title text
    pub title: String,
    /// Optional description extracted from title (prepended to existing)
    pub extracted_description: Option<String>,
}

/// Trim whitespace from a simple text field (note, description, reason).
pub fn trim_field(text: &str) -> String {
    text.trim().to_string()
}

/// Normalize a title with potential splitting and quote handling.
pub fn normalize_title(text: &str) -> NormalizedTitle {
    let trimmed = text.trim();

    // Check for split point
    if let Some((title_part, desc_part)) = find_split_point(trimmed) {
        NormalizedTitle {
            title: normalize_title_text(title_part.trim()),
            extracted_description: Some(desc_part.trim().to_string()),
        }
    } else {
        NormalizedTitle {
            title: normalize_title_text(trimmed),
            extracted_description: None,
        }
    }
}

/// Find split point if double-newline exists after threshold.
fn find_split_point(text: &str) -> Option<(&str, &str)> {
    // Find all double-newline positions
    for (idx, _) in text.match_indices("\n\n") {
        // Check if this position is past the threshold
        let prefix = &text[..idx];
        if is_past_threshold(prefix) {
            return Some((&text[..idx], &text[idx + 2..]));
        }
    }
    None
}

/// Check if text is past the "3 words or 20 chars" threshold.
fn is_past_threshold(text: &str) -> bool {
    let char_count = text.chars().count();
    let word_count = text.split_whitespace().count();

    char_count >= 20 || word_count >= 3
}

/// Normalize title text with quote-aware newline handling.
fn normalize_title_text(text: &str) -> String {
    let tokens = tokenize(text);
    let mut result = String::new();

    for token in tokens {
        match token {
            Token::Quoted { quote, content } => {
                result.push(quote);
                result.push_str(&escape_newlines(&content));
                result.push(closing_quote(quote));
            }
            Token::Unquoted(s) => {
                result.push_str(&collapse_whitespace(&s));
            }
        }
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Quoted { quote: char, content: String },
    Unquoted(String),
}

/// Tokenize into quoted and unquoted segments.
fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current_unquoted = String::new();

    while let Some(c) = chars.next() {
        if is_opening_quote(c) {
            // Flush any accumulated unquoted content
            if !current_unquoted.is_empty() {
                tokens.push(Token::Unquoted(std::mem::take(&mut current_unquoted)));
            }

            // Find the matching closing quote
            let expected_close = closing_quote(c);
            let mut quoted_content = String::new();

            loop {
                match chars.next() {
                    Some(ch) if ch == expected_close => {
                        // Found closing quote
                        tokens.push(Token::Quoted {
                            quote: c,
                            content: quoted_content,
                        });
                        break;
                    }
                    Some(ch) => {
                        quoted_content.push(ch);
                    }
                    None => {
                        // Unclosed quote - treat rest as quoted
                        tokens.push(Token::Quoted {
                            quote: c,
                            content: quoted_content,
                        });
                        break;
                    }
                }
            }
        } else {
            current_unquoted.push(c);
        }
    }

    // Flush any remaining unquoted content
    if !current_unquoted.is_empty() {
        tokens.push(Token::Unquoted(current_unquoted));
    }

    tokens
}

/// Escape newlines as \n within quoted strings.
fn escape_newlines(text: &str) -> String {
    text.replace('\r', "\\r").replace('\n', "\\n")
}

/// Collapse whitespace and newlines in unquoted text.
fn collapse_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut last_was_space = false;

    for c in text.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result
}

/// Check if a character is an opening quote.
fn is_opening_quote(c: char) -> bool {
    matches!(
        c,
        '"' | '\'' | '`' | '\u{201C}' | '\u{201D}' | '\u{2018}' | '\u{2019}'
    )
}

/// Get closing quote for opening quote.
fn closing_quote(open: char) -> char {
    match open {
        '\u{201C}' => '\u{201D}', // " -> "
        '\u{2018}' => '\u{2019}', // ' -> '
        c => c,                   // Same character for ASCII quotes
    }
}

#[cfg(test)]
#[path = "normalize_tests.rs"]
mod tests;
