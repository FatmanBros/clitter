use once_cell::sync::Lazy;
use regex::Regex;

use crate::types::{Category, ClipboardData};

static NUMERIC_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[\d,.\-/\s]+$").unwrap());

// URL pattern: matches http:// or https:// URLs
static URL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^https?://[^\s]+$").unwrap()
});

static SECURE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Key-value patterns (api_key=, password:, etc.)
        Regex::new(r"(?i)(api[_-]?key|apikey)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(password|passwd|pwd)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(secret|token|bearer)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(private[_-]?key)").unwrap(),
        Regex::new(r"(?i)(access[_-]?token|auth[_-]?token)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(client[_-]?secret|client[_-]?id)\s*[:=]").unwrap(),
        Regex::new(r"(?i)(credentials?|auth)\s*[:=]").unwrap(),
        // Keyword-only patterns (secrets, confidential, etc.)
        Regex::new(r"(?i)\b(secrets?|confidential|sensitive)\b").unwrap(),
        // API key prefixes (common patterns)
        Regex::new(r"(?i)^(sk|pk|api|key|secret|token)[_-][a-zA-Z0-9]{16,}$").unwrap(),
        // Long base64-like strings (40+ chars, likely tokens)
        Regex::new(r"^[A-Za-z0-9+/]{40,}={0,2}$").unwrap(),
        // Hex strings 32+ chars (API keys, hashes)
        Regex::new(r"^[a-f0-9]{32,}$").unwrap(),
        // Cloud provider patterns
        Regex::new(r"(?i)(aws|azure|gcp|github|gitlab|slack|stripe|twilio|sendgrid)[_-]?(secret|key|token|api)").unwrap(),
        // Database connection strings
        Regex::new(r"(?i)(mongodb|postgres|mysql|redis|mssql)://").unwrap(),
        // JWT tokens
        Regex::new(r"^eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$").unwrap(),
        // SSH/PGP key markers
        Regex::new(r"(?i)-----BEGIN\s+(RSA\s+)?(PRIVATE|ENCRYPTED|OPENSSH)").unwrap(),
        // Environment variable declarations with sensitive names
        Regex::new(r"(?i)^(export\s+)?(DB_|DATABASE_|MONGO_|REDIS_|AWS_|AZURE_|GCP_|API_|SECRET_|TOKEN_|AUTH_|PRIVATE_)[A-Z_]+=").unwrap(),
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

        // Check for URL pattern
        if URL_PATTERN.is_match(trimmed) {
            return Category::Url;
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
