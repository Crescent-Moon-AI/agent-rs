//! Language support for prompt templates
//!
//! This module provides a flexible language enum that supports common languages
//! and allows extension via the `Other` variant.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported languages for prompts
///
/// # Examples
///
/// ```
/// use agent_prompt::Language;
///
/// let lang = Language::Chinese;
/// assert_eq!(lang.code(), "zh");
/// assert_eq!(lang.name(), "Chinese");
///
/// // Parse from string
/// let parsed = Language::from_code("en");
/// assert_eq!(parsed, Language::English);
///
/// // Custom language
/// let custom = Language::Other("ja".to_string());
/// assert_eq!(custom.code(), "ja");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Language {
    /// English
    #[default]
    English,
    /// Chinese (Simplified)
    Chinese,
    /// Other languages (ISO 639-1 code)
    Other(String),
}

impl Language {
    /// Get ISO 639-1 language code
    pub fn code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
            Language::Other(code) => code,
        }
    }

    /// Get language name for display
    pub fn name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Chinese => "Chinese",
            Language::Other(code) => code,
        }
    }

    /// Parse from ISO 639-1 code or common name
    ///
    /// # Examples
    ///
    /// ```
    /// use agent_prompt::Language;
    ///
    /// assert_eq!(Language::from_code("en"), Language::English);
    /// assert_eq!(Language::from_code("english"), Language::English);
    /// assert_eq!(Language::from_code("zh"), Language::Chinese);
    /// assert_eq!(Language::from_code("chinese"), Language::Chinese);
    /// assert_eq!(Language::from_code("中文"), Language::Chinese);
    /// assert_eq!(Language::from_code("ja"), Language::Other("ja".to_string()));
    /// ```
    pub fn from_code(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "en" | "english" => Language::English,
            "zh" | "chinese" | "中文" | "zh-cn" | "zh-hans" => Language::Chinese,
            other => Language::Other(other.to_string()),
        }
    }

    /// Check if this is a known language (not Other)
    pub fn is_known(&self) -> bool {
        !matches!(self, Language::Other(_))
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<&str> for Language {
    fn from(s: &str) -> Self {
        Language::from_code(s)
    }
}

impl From<String> for Language {
    fn from(s: String) -> Self {
        Language::from_code(&s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Chinese.code(), "zh");
        assert_eq!(Language::Other("ja".to_string()).code(), "ja");
    }

    #[test]
    fn test_language_name() {
        assert_eq!(Language::English.name(), "English");
        assert_eq!(Language::Chinese.name(), "Chinese");
        assert_eq!(Language::Other("ja".to_string()).name(), "ja");
    }

    #[test]
    fn test_from_code() {
        assert_eq!(Language::from_code("en"), Language::English);
        assert_eq!(Language::from_code("EN"), Language::English);
        assert_eq!(Language::from_code("english"), Language::English);
        assert_eq!(Language::from_code("English"), Language::English);

        assert_eq!(Language::from_code("zh"), Language::Chinese);
        assert_eq!(Language::from_code("chinese"), Language::Chinese);
        assert_eq!(Language::from_code("中文"), Language::Chinese);
        assert_eq!(Language::from_code("zh-cn"), Language::Chinese);

        assert_eq!(
            Language::from_code("ja"),
            Language::Other("ja".to_string())
        );
    }

    #[test]
    fn test_is_known() {
        assert!(Language::English.is_known());
        assert!(Language::Chinese.is_known());
        assert!(!Language::Other("ja".to_string()).is_known());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Language::English), "English");
        assert_eq!(format!("{}", Language::Chinese), "Chinese");
    }

    #[test]
    fn test_default() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn test_from_string() {
        let lang: Language = "zh".into();
        assert_eq!(lang, Language::Chinese);

        let lang: Language = String::from("english").into();
        assert_eq!(lang, Language::English);
    }

    #[test]
    fn test_serde() {
        let lang = Language::Chinese;
        let json = serde_json::to_string(&lang).unwrap();
        let parsed: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, lang);
    }
}
