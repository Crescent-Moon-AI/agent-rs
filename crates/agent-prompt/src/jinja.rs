//! MiniJinja-based template implementation
//!
//! This module provides a [`JinjaTemplate`] implementation that uses the MiniJinja
//! template engine for variable interpolation and conditional rendering.

use crate::{Language, PromptError, PromptTemplate, Result};
use minijinja::Environment;
use std::collections::HashMap;

/// A prompt template backed by MiniJinja
///
/// `JinjaTemplate` provides a thread-safe, multi-language template implementation
/// using the Jinja2-compatible MiniJinja engine.
///
/// # Template Syntax
///
/// The template uses standard Jinja2 syntax:
/// - Variables: `{{ variable }}`
/// - Filters: `{{ name | upper }}`
/// - Conditionals: `{% if condition %}...{% endif %}`
/// - Loops: `{% for item in items %}...{% endfor %}`
///
/// # Examples
///
/// ```ignore
/// use agent_prompt::{JinjaTemplate, Language};
/// use serde_json::json;
///
/// // Create a bilingual template
/// let template = JinjaTemplate::bilingual(
///     "greeting",
///     "Hello, {{ name }}!",
///     "你好，{{ name }}！",
/// )?;
///
/// // Render with variables
/// let result = template.render(&Language::English, &json!({ "name": "World" }))?;
/// assert_eq!(result, "Hello, World!");
/// ```
pub struct JinjaTemplate {
    name: String,
    templates: HashMap<Language, String>,
}

impl JinjaTemplate {
    /// Create a new template builder
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agent_prompt::{JinjaTemplate, Language};
    ///
    /// let template = JinjaTemplate::builder("my_prompt")
    ///     .english("Hello, {{ name }}!")
    ///     .chinese("你好，{{ name }}！")
    ///     .build()?;
    /// ```
    pub fn builder(name: impl Into<String>) -> JinjaTemplateBuilder {
        JinjaTemplateBuilder::new(name)
    }

    /// Create from a single template (language-agnostic, defaults to English)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agent_prompt::JinjaTemplate;
    ///
    /// let template = JinjaTemplate::new("simple", "Hello, {{ name }}!")?;
    /// ```
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Result<Self> {
        Self::builder(name).english(template).build()
    }

    /// Create with English and Chinese templates
    ///
    /// This is a convenience method for the common bilingual case.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agent_prompt::JinjaTemplate;
    ///
    /// let template = JinjaTemplate::bilingual(
    ///     "greeting",
    ///     "Hello!",
    ///     "你好！",
    /// )?;
    /// ```
    pub fn bilingual(
        name: impl Into<String>,
        english: impl Into<String>,
        chinese: impl Into<String>,
    ) -> Result<Self> {
        Self::builder(name).english(english).chinese(chinese).build()
    }

}

impl PromptTemplate for JinjaTemplate {
    fn name(&self) -> &str {
        &self.name
    }

    fn languages(&self) -> Vec<Language> {
        self.templates.keys().cloned().collect()
    }

    fn render(&self, lang: &Language, vars: &serde_json::Value) -> Result<String> {
        let template_str = self.templates.get(lang).ok_or_else(|| {
            PromptError::TemplateNotFound {
                name: self.name.clone(),
                language: lang.code().to_string(),
                detail: "Language not available".to_string(),
            }
        })?;

        // Create a new environment for each render to avoid lifetime issues
        let mut env = Environment::new();

        // Add built-in filters
        env.add_filter("upper", |s: String| s.to_uppercase());
        env.add_filter("lower", |s: String| s.to_lowercase());
        env.add_filter("trim", |s: String| s.trim().to_string());
        env.add_filter("capitalize", |s: String| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        });

        let value = minijinja::value::Value::from_serialize(vars);

        env.render_str(template_str, value)
            .map_err(|e| PromptError::RenderError {
                name: self.name.clone(),
                detail: e.to_string(),
            })
    }

    fn raw_template(&self, lang: &Language) -> Option<&str> {
        self.templates.get(lang).map(|s| s.as_str())
    }
}

impl std::fmt::Debug for JinjaTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JinjaTemplate")
            .field("name", &self.name)
            .field("languages", &self.templates.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Builder for [`JinjaTemplate`]
///
/// Provides a fluent API for constructing templates with multiple language variants.
///
/// # Examples
///
/// ```ignore
/// use agent_prompt::{JinjaTemplate, Language};
///
/// let template = JinjaTemplate::builder("analyzer")
///     .english("Analyze {{ symbol }} stock")
///     .chinese("分析 {{ symbol }} 股票")
///     .template(Language::Other("ja".to_string()), "{{ symbol }}株を分析")
///     .build()?;
/// ```
pub struct JinjaTemplateBuilder {
    name: String,
    templates: HashMap<Language, String>,
}

impl JinjaTemplateBuilder {
    /// Create a new builder with the given template name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            templates: HashMap::new(),
        }
    }

    /// Add a template for a specific language
    pub fn template(mut self, lang: Language, content: impl Into<String>) -> Self {
        self.templates.insert(lang, content.into());
        self
    }

    /// Add English template
    pub fn english(self, content: impl Into<String>) -> Self {
        self.template(Language::English, content)
    }

    /// Add Chinese template
    pub fn chinese(self, content: impl Into<String>) -> Self {
        self.template(Language::Chinese, content)
    }

    /// Build the template
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No templates were provided
    /// - A template fails to parse
    pub fn build(self) -> Result<JinjaTemplate> {
        if self.templates.is_empty() {
            return Err(PromptError::NoTemplatesProvided(self.name));
        }

        // Validate all templates parse correctly
        let env = Environment::new();
        for (lang, content) in &self.templates {
            env.render_str(content, ())
                .map_err(|e| PromptError::TemplateParseFailed {
                    name: self.name.clone(),
                    language: lang.code().to_string(),
                    detail: e.to_string(),
                })?;
        }

        Ok(JinjaTemplate {
            name: self.name,
            templates: self.templates,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_template() {
        let template = JinjaTemplate::new("test", "Hello, {{ name }}!").unwrap();

        let result = template
            .render(&Language::English, &json!({ "name": "World" }))
            .unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_bilingual_template() {
        let template =
            JinjaTemplate::bilingual("greeting", "Hello, {{ name }}!", "你好，{{ name }}！")
                .unwrap();

        let en = template
            .render(&Language::English, &json!({ "name": "World" }))
            .unwrap();
        assert_eq!(en, "Hello, World!");

        let zh = template
            .render(&Language::Chinese, &json!({ "name": "世界" }))
            .unwrap();
        assert_eq!(zh, "你好，世界！");
    }

    #[test]
    fn test_builder() {
        let template = JinjaTemplate::builder("test")
            .english("EN: {{ msg }}")
            .chinese("ZH: {{ msg }}")
            .build()
            .unwrap();

        assert_eq!(template.name(), "test");
        assert!(template.supports_language(&Language::English));
        assert!(template.supports_language(&Language::Chinese));
    }

    #[test]
    fn test_custom_language() {
        let template = JinjaTemplate::builder("test")
            .english("Hello")
            .template(Language::Other("ja".to_string()), "こんにちは")
            .build()
            .unwrap();

        let ja = template
            .render(&Language::Other("ja".to_string()), &json!({}))
            .unwrap();
        assert_eq!(ja, "こんにちは");
    }

    #[test]
    fn test_filters() {
        let template = JinjaTemplate::new("test", "{{ name | upper }}").unwrap();

        let result = template
            .render(&Language::English, &json!({ "name": "hello" }))
            .unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_no_templates_error() {
        let result = JinjaTemplate::builder("test").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_template_error() {
        let result = JinjaTemplate::new("test", "{{ unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn test_render_missing_language() {
        let template = JinjaTemplate::new("test", "Hello").unwrap();

        let result = template.render(&Language::Chinese, &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_raw_template() {
        let template = JinjaTemplate::bilingual("test", "Hello", "你好").unwrap();

        assert_eq!(template.raw_template(&Language::English), Some("Hello"));
        assert_eq!(template.raw_template(&Language::Chinese), Some("你好"));
        assert_eq!(
            template.raw_template(&Language::Other("ja".to_string())),
            None
        );
    }

    #[test]
    fn test_fallback() {
        let template = JinjaTemplate::bilingual("test", "Hello", "你好").unwrap();

        // Japanese not available, should fall back to English
        let result = template
            .render_with_fallback(&Language::Other("ja".to_string()), &json!({}))
            .unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_multiline_template() {
        let template = JinjaTemplate::new(
            "system",
            r#"You are a helpful assistant.

Your name is {{ name }}.
Your role is {{ role }}."#,
        )
        .unwrap();

        let result = template
            .render(
                &Language::English,
                &json!({
                    "name": "Claude",
                    "role": "analyst"
                }),
            )
            .unwrap();

        assert!(result.contains("Claude"));
        assert!(result.contains("analyst"));
    }

    #[test]
    fn test_conditional() {
        let template = JinjaTemplate::new(
            "conditional",
            r#"{% if detailed %}Detailed analysis{% else %}Brief analysis{% endif %}"#,
        )
        .unwrap();

        let detailed = template
            .render(&Language::English, &json!({ "detailed": true }))
            .unwrap();
        assert_eq!(detailed, "Detailed analysis");

        let brief = template
            .render(&Language::English, &json!({ "detailed": false }))
            .unwrap();
        assert_eq!(brief, "Brief analysis");
    }

    #[test]
    fn test_loop() {
        let template = JinjaTemplate::new(
            "loop",
            r#"{% for item in items %}- {{ item }}
{% endfor %}"#,
        )
        .unwrap();

        let result = template
            .render(
                &Language::English,
                &json!({ "items": ["one", "two", "three"] }),
            )
            .unwrap();

        assert!(result.contains("- one"));
        assert!(result.contains("- two"));
        assert!(result.contains("- three"));
    }

    #[test]
    fn test_debug() {
        let template = JinjaTemplate::bilingual("test", "Hello", "你好").unwrap();
        let debug = format!("{:?}", template);
        assert!(debug.contains("JinjaTemplate"));
        assert!(debug.contains("test"));
    }
}
