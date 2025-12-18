//! Core prompt template trait
//!
//! This module defines the [`PromptTemplate`] trait that all template implementations must follow.

use crate::{Language, PromptError, Result};

/// Core trait for prompt templates
///
/// Implementations of this trait provide multi-language prompt templating capabilities.
/// Templates can be rendered with variables and support fallback to a default language
/// when the requested language is not available.
///
/// This trait is dyn-compatible, using `serde_json::Value` for variables instead of generics.
///
/// # Examples
///
/// ```ignore
/// use agent_prompt::{JinjaTemplate, Language, PromptTemplate};
/// use serde_json::json;
///
/// let template = JinjaTemplate::bilingual(
///     "greeting",
///     "Hello, {{ name }}!",
///     "你好，{{ name }}！",
/// )?;
///
/// let result = template.render(&Language::Chinese, &json!({ "name": "World" }))?;
/// assert_eq!(result, "你好，World！");
/// ```
pub trait PromptTemplate: Send + Sync {
    /// Get the template name/identifier
    fn name(&self) -> &str;

    /// Get available languages
    fn languages(&self) -> Vec<Language>;

    /// Check if a language is supported
    fn supports_language(&self, lang: &Language) -> bool {
        self.languages().contains(lang)
    }

    /// Render the template with variables for a specific language
    ///
    /// Returns an error if the language is not supported or rendering fails.
    /// Variables are passed as `serde_json::Value` to maintain dyn-compatibility.
    fn render(&self, lang: &Language, vars: &serde_json::Value) -> Result<String>;

    /// Render with fallback to default language
    ///
    /// If the requested language is not available:
    /// 1. Try English as fallback
    /// 2. If English not available, use the first available language
    /// 3. If no languages available, return error
    fn render_with_fallback(
        &self,
        lang: &Language,
        vars: &serde_json::Value,
    ) -> Result<String> {
        if self.supports_language(lang) {
            return self.render(lang, vars);
        }

        // Fallback to English
        if self.supports_language(&Language::English) {
            return self.render(&Language::English, vars);
        }

        // Fallback to first available
        let fallback = self
            .languages()
            .into_iter()
            .next()
            .ok_or_else(|| PromptError::NoLanguageAvailable(self.name().to_string()))?;

        self.render(&fallback, vars)
    }

    /// Get raw template string for a language (for debugging/inspection)
    fn raw_template(&self, lang: &Language) -> Option<&str>;

    /// Get the default language for this template
    ///
    /// Returns the first available language, or English if available.
    fn default_language(&self) -> Option<Language> {
        let langs = self.languages();
        if langs.contains(&Language::English) {
            Some(Language::English)
        } else {
            langs.into_iter().next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    /// A simple test implementation of PromptTemplate
    struct SimpleTemplate {
        name: String,
        templates: HashMap<Language, String>,
    }

    impl SimpleTemplate {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                templates: HashMap::new(),
            }
        }

        fn with_template(mut self, lang: Language, content: &str) -> Self {
            self.templates.insert(lang, content.to_string());
            self
        }
    }

    impl PromptTemplate for SimpleTemplate {
        fn name(&self) -> &str {
            &self.name
        }

        fn languages(&self) -> Vec<Language> {
            self.templates.keys().cloned().collect()
        }

        fn render(&self, lang: &Language, _vars: &serde_json::Value) -> Result<String> {
            self.templates
                .get(lang)
                .cloned()
                .ok_or_else(|| PromptError::TemplateNotFound {
                    name: self.name.clone(),
                    language: lang.code().to_string(),
                    detail: "Language not available".to_string(),
                })
        }

        fn raw_template(&self, lang: &Language) -> Option<&str> {
            self.templates.get(lang).map(|s| s.as_str())
        }
    }

    #[test]
    fn test_supports_language() {
        let template = SimpleTemplate::new("test")
            .with_template(Language::English, "Hello")
            .with_template(Language::Chinese, "你好");

        assert!(template.supports_language(&Language::English));
        assert!(template.supports_language(&Language::Chinese));
        assert!(!template.supports_language(&Language::Other("ja".to_string())));
    }

    #[test]
    fn test_render() {
        let template = SimpleTemplate::new("test").with_template(Language::English, "Hello");

        let result = template.render(&Language::English, &json!({})).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_render_with_fallback_to_english() {
        let template = SimpleTemplate::new("test")
            .with_template(Language::English, "Hello")
            .with_template(Language::Chinese, "你好");

        // Request Japanese, should fallback to English
        let result = template
            .render_with_fallback(&Language::Other("ja".to_string()), &json!({}))
            .unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_render_with_fallback_to_first() {
        let template = SimpleTemplate::new("test").with_template(Language::Chinese, "你好");

        // Request Japanese, no English, should fallback to Chinese
        let result = template
            .render_with_fallback(&Language::Other("ja".to_string()), &json!({}))
            .unwrap();
        assert_eq!(result, "你好");
    }

    #[test]
    fn test_render_with_fallback_no_languages() {
        let template = SimpleTemplate::new("test");

        let result = template.render_with_fallback(&Language::English, &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_default_language() {
        let template = SimpleTemplate::new("test")
            .with_template(Language::Chinese, "你好")
            .with_template(Language::English, "Hello");

        // Should prefer English
        assert_eq!(template.default_language(), Some(Language::English));

        let template2 = SimpleTemplate::new("test2").with_template(Language::Chinese, "你好");

        // No English, should return first available
        assert_eq!(template2.default_language(), Some(Language::Chinese));

        let template3 = SimpleTemplate::new("test3");
        assert_eq!(template3.default_language(), None);
    }
}
