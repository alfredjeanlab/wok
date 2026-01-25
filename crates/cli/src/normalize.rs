// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Alfred Jean LLC

//! Text normalization for issue fields.
//!
//! Handles whitespace trimming, title splitting, and quote-aware
//! newline handling per REQUIREMENTS.md specification.

/// Maximum length for a title before auto-truncation.
/// Titles longer than this are truncated and full content moves to description.
const TITLE_TRUNCATE_LENGTH: usize = 120;

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

    // Check for split point (double-newline after threshold)
    let (title_text, desc_text) = if let Some((title_part, desc_part)) = find_split_point(trimmed) {
        (
            normalize_title_text(title_part.trim()),
            Some(desc_part.trim().to_string()),
        )
    } else {
        (normalize_title_text(trimmed), None)
    };

    // If title is too long, truncate and move full original content to description
    if title_text.chars().count() > TITLE_TRUNCATE_LENGTH {
        let truncated = truncate_at_word_boundary(&title_text, TITLE_TRUNCATE_LENGTH);
        // Preserve full original input (trimmed) in description
        let full_description = match desc_text {
            Some(desc) => format!("{}\n\n{}", trimmed, desc),
            None => trimmed.to_string(),
        };
        NormalizedTitle {
            title: truncated,
            extracted_description: Some(full_description),
        }
    } else {
        NormalizedTitle {
            title: title_text,
            extracted_description: desc_text,
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

/// Truncate text at a word boundary near the given character limit.
/// Tries to break at a space before the limit, falling back to hard truncation.
fn truncate_at_word_boundary(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }

    // Look for last space within the limit
    let search_end = max_chars.min(chars.len());
    let mut last_space = None;
    for i in (0..search_end).rev() {
        if chars[i] == ' ' {
            last_space = Some(i);
            break;
        }
    }

    // Use word boundary if found reasonably close, otherwise hard truncate
    let truncate_at = match last_space {
        Some(pos) if pos > max_chars / 2 => pos,
        _ => max_chars,
    };

    let truncated: String = chars[..truncate_at].iter().collect();
    format!("{}...", truncated.trim_end())
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
