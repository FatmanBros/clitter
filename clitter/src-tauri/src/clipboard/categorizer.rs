use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{Category, ClipboardData};

static NUMERIC_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\d,.\-/\s]+$").unwrap());

static SECURE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)(api[_-]?key|apikey)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(password|passwd|pwd)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(secret|token|bearer)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(private[_-]?key)").unwrap(),
        Regex::new(r"(?i)^(sk|pk|api|key)[_-][a-zA-Z0-9]{20,}$").unwrap(),
        Regex::new(r"^[A-Za-z0-9+/]{40,}={0,2}$").unwrap(), // Base64-like long string
        Regex::new(r"^[a-f0-9]{32,}$").unwrap(),            // Hex string (API keys)
        Regex::new(r"(?i)(aws|azure|gcp|github|gitlab)[_-]?(secret|key|token)").unwrap(),
    ]
});

pub struct Categorizer;

impl Categorizer {
    pub fn categorize(data: &ClipboardData) -> Category {
        match data {
            ClipboardData::Image { .. } => Category::Image,
            ClipboardData::Text { text, .. } => Self::categorize_text(text),
        }
    }

    fn categorize_text(text: &str) -> Category {
        let trimmed = text.trim();

        // Check for secure patterns first (highest priority)
        for pattern in SECURE_PATTERNS.iter() {
            if pattern.is_match(trimmed) {
                return Category::Secure;
            }
        }

        // Check for numeric pattern
        if !trimmed.is_empty() && NUMERIC_PATTERN.is_match(trimmed) {
            return Category::Numeric;
        }

        Category::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_numeric() {
        let data = ClipboardData::Text {
            text: "1,234,567".to_string(),
            preview: "1,234,567".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Numeric);

        let data = ClipboardData::Text {
            text: "2024/01/27".to_string(),
            preview: "2024/01/27".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Numeric);
    }

    #[test]
    fn test_categorize_secure() {
        let data = ClipboardData::Text {
            text: "api_key=sk_live_abc123def456".to_string(),
            preview: "api_key=sk_live_abc123def456".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Secure);

        let data = ClipboardData::Text {
            text: "password: mysecretpassword".to_string(),
            preview: "password: mysecretpassword".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Secure);
    }

    #[test]
    fn test_categorize_text() {
        let data = ClipboardData::Text {
            text: "Hello, World!".to_string(),
            preview: "Hello, World!".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Text);
    }

    #[test]
    fn test_categorize_image() {
        let data = ClipboardData::Image {
            base64: "base64data".to_string(),
            width: 100,
            height: 100,
            format: "png".to_string(),
        };
        assert_eq!(Categorizer::categorize(&data), Category::Image);
    }
}
