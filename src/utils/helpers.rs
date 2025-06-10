//! Helper functions and utilities
//! 
//! This module contains common helper functions used throughout the application.

use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use std::collections::HashMap;

/// Generate a new UUID v4
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// Format a timestamp for display
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format a timestamp for user display (relative time)
pub fn format_relative_time(timestamp: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(timestamp);
    
    if diff < Duration::minutes(1) {
        "just now".to_string()
    } else if diff < Duration::hours(1) {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff < Duration::days(1) {
        format!("{} hours ago", diff.num_hours())
    } else if diff < Duration::weeks(1) {
        format!("{} days ago", diff.num_days())
    } else {
        format_timestamp(timestamp)
    }
}

/// Truncate text to a maximum length with ellipsis
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// Escape markdown special characters
pub fn escape_markdown(text: &str) -> String {
    text.replace('_', r"\_")
        .replace('*', r"\*")
        .replace('[', r"\[")
        .replace(']', r"\]")
        .replace('(', r"\(")
        .replace(')', r"\)")
        .replace('~', r"\~")
        .replace('`', r"\`")
        .replace('>', r"\>")
        .replace('#', r"\#")
        .replace('+', r"\+")
        .replace('-', r"\-")
        .replace('=', r"\=")
        .replace('|', r"\|")
        .replace('{', r"\{")
        .replace('}', r"\}")
        .replace('.', r"\.")
        .replace('!', r"\!")
}

/// Parse user mention from text
pub fn parse_user_mention(text: &str) -> Option<i64> {
    if text.starts_with("@") {
        // Handle username mentions (would need database lookup)
        None
    } else if text.starts_with("tg://user?id=") {
        // Handle user ID mentions
        text.strip_prefix("tg://user?id=")
            .and_then(|id_str| id_str.parse::<i64>().ok())
    } else {
        // Try to parse as direct user ID
        text.parse::<i64>().ok()
    }
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Validate phone number format (basic validation)
pub fn is_valid_phone(phone: &str) -> bool {
    phone.chars().all(|c| c.is_ascii_digit() || c == '+' || c == '-' || c == ' ')
        && phone.len() >= 10
}

/// Extract hashtags from text
pub fn extract_hashtags(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|word| word.starts_with('#') && word.len() > 1)
        .map(|tag| tag[1..].to_lowercase())
        .collect()
}

/// Create a pagination info string
pub fn create_pagination_info(current_page: usize, total_pages: usize, total_items: usize) -> String {
    if total_pages <= 1 {
        format!("Total: {}", total_items)
    } else {
        format!("Page {} of {} (Total: {})", current_page, total_pages, total_items)
    }
}

/// Calculate pagination offset
pub fn calculate_offset(page: usize, page_size: usize) -> usize {
    page.saturating_sub(1) * page_size
}

/// Sanitize filename for safe storage
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Convert bytes to human readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Parse key-value pairs from text (e.g., "key1=value1 key2=value2")
pub fn parse_key_value_pairs(text: &str) -> HashMap<String, String> {
    let mut pairs = HashMap::new();
    
    for part in text.split_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            pairs.insert(key.to_string(), value.to_string());
        }
    }
    
    pairs
}

/// Generate a random alphanumeric string
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Check if a string contains only printable ASCII characters
pub fn is_printable_ascii(text: &str) -> bool {
    text.chars().all(|c| c.is_ascii() && !c.is_control())
}

/// Normalize whitespace in text
pub fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_text() {
        assert_eq!(truncate_text("hello", 10), "hello");
        assert_eq!(truncate_text("hello world", 8), "hello...");
    }

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("*bold*"), r"\*bold\*");
        assert_eq!(escape_markdown("_italic_"), r"\_italic\_");
    }

    #[test]
    fn test_parse_user_mention() {
        assert_eq!(parse_user_mention("123456789"), Some(123456789));
        assert_eq!(parse_user_mention("tg://user?id=123456789"), Some(123456789));
        assert_eq!(parse_user_mention("@username"), None);
    }

    #[test]
    fn test_extract_hashtags() {
        let tags = extract_hashtags("Hello #world #rust #programming!");
        assert_eq!(tags, vec!["world", "rust", "programming!"]);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
        assert_eq!(format_bytes(500), "500 B");
    }
}