//! Prompt template registry
//!
//! This module provides [`PromptRegistry`], a thread-safe registry for managing
//! and accessing prompt templates.

use crate::{Language, PromptError, PromptTemplate, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A thread-safe registry for managing prompt templates
///
/// `PromptRegistry` provides a centralized location for storing and retrieving
/// prompt templates. It supports:
/// - Thread-safe registration and lookup
/// - Default language configuration
/// - Convenient render methods with automatic fallback
///
/// # Examples
///
/// ```ignore
/// use agent_prompt::{PromptRegistry, JinjaTemplate, Language};
/// use serde_json::json;
///
/// let registry = PromptRegistry::with_language(Language::Chinese);
///
/// // Register a template
/// let template = JinjaTemplate::bilingual(
///     "greeting",
///     "Hello, {{ name }}!",
///     "你好，{{ name }}！",
/// )?;
/// registry.register(template);
///
/// // Render using default language (Chinese)
/// let result = registry.render("greeting", &json!({ "name": "World" }))?;
/// assert_eq!(result, "你好，World！");
/// ```
pub struct PromptRegistry {
    templates: RwLock<HashMap<String, Arc<dyn PromptTemplate>>>,
    default_language: RwLock<Language>,
}

impl PromptRegistry {
    /// Create a new empty registry with English as default language
    pub fn new() -> Self {
        Self {
            templates: RwLock::new(HashMap::new()),
            default_language: RwLock::new(Language::English),
        }
    }

    /// Create a registry with a specific default language
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agent_prompt::{PromptRegistry, Language};
    ///
    /// let registry = PromptRegistry::with_language(Language::Chinese);
    /// assert_eq!(registry.default_language(), Language::Chinese);
    /// ```
    pub fn with_language(lang: Language) -> Self {
        Self {
            templates: RwLock::new(HashMap::new()),
            default_language: RwLock::new(lang),
        }
    }

    /// Set the default language
    pub fn set_default_language(&self, lang: Language) {
        if let Ok(mut default) = self.default_language.write() {
            *default = lang;
        }
    }

    /// Get the default language
    pub fn default_language(&self) -> Language {
        self.default_language
            .read()
            .map(|l| l.clone())
            .unwrap_or(Language::English)
    }

    /// Register a template
    ///
    /// If a template with the same name already exists, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use agent_prompt::{PromptRegistry, JinjaTemplate};
    ///
    /// let registry = PromptRegistry::new();
    /// let template = JinjaTemplate::new("greeting", "Hello!")?;
    /// registry.register(template);
    /// ```
    pub fn register<T: PromptTemplate + 'static>(&self, template: T) {
        if let Ok(mut templates) = self.templates.write() {
            templates.insert(template.name().to_string(), Arc::new(template));
        }
    }

    /// Register a template wrapped in Arc
    pub fn register_arc(&self, template: Arc<dyn PromptTemplate>) {
        if let Ok(mut templates) = self.templates.write() {
            templates.insert(template.name().to_string(), template);
        }
    }

    /// Register multiple templates at once
    pub fn register_all<T: PromptTemplate + 'static>(&self, templates: Vec<T>) {
        for template in templates {
            self.register(template);
        }
    }

    /// Get a template by name
    ///
    /// Returns `None` if the template is not registered.
    pub fn get(&self, name: &str) -> Option<Arc<dyn PromptTemplate>> {
        self.templates.read().ok()?.get(name).cloned()
    }

    /// Check if a template is registered
    pub fn contains(&self, name: &str) -> bool {
        self.templates
            .read()
            .map(|t| t.contains_key(name))
            .unwrap_or(false)
    }

    /// Remove a template by name
    ///
    /// Returns the removed template if it existed.
    pub fn remove(&self, name: &str) -> Option<Arc<dyn PromptTemplate>> {
        self.templates.write().ok()?.remove(name)
    }

    /// Render a template with the default language
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template is not registered
    /// - Rendering fails
    pub fn render(&self, name: &str, vars: &serde_json::Value) -> Result<String> {
        let template = self
            .get(name)
            .ok_or_else(|| PromptError::TemplateNotRegistered(name.to_string()))?;

        let lang = self.default_language();
        template.render_with_fallback(&lang, vars)
    }

    /// Render a template with a specific language
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template is not registered
    /// - Rendering fails
    pub fn render_with_lang(
        &self,
        name: &str,
        lang: &Language,
        vars: &serde_json::Value,
    ) -> Result<String> {
        let template = self
            .get(name)
            .ok_or_else(|| PromptError::TemplateNotRegistered(name.to_string()))?;

        template.render_with_fallback(lang, vars)
    }

    /// List all registered template names
    pub fn list(&self) -> Vec<String> {
        self.templates
            .read()
            .map(|t| t.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the number of registered templates
    pub fn len(&self) -> usize {
        self.templates.read().map(|t| t.len()).unwrap_or(0)
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all registered templates
    pub fn clear(&self) {
        if let Ok(mut templates) = self.templates.write() {
            templates.clear();
        }
    }
}

impl Default for PromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PromptRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptRegistry")
            .field("default_language", &self.default_language())
            .field("template_count", &self.len())
            .field("templates", &self.list())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JinjaTemplate;
    use serde_json::json;

    #[test]
    fn test_new_registry() {
        let registry = PromptRegistry::new();
        assert_eq!(registry.default_language(), Language::English);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_with_language() {
        let registry = PromptRegistry::with_language(Language::Chinese);
        assert_eq!(registry.default_language(), Language::Chinese);
    }

    #[test]
    fn test_set_default_language() {
        let registry = PromptRegistry::new();
        registry.set_default_language(Language::Chinese);
        assert_eq!(registry.default_language(), Language::Chinese);
    }

    #[test]
    fn test_register_and_get() {
        let registry = PromptRegistry::new();
        let template = JinjaTemplate::new("test", "Hello").unwrap();

        registry.register(template);

        assert!(registry.contains("test"));
        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_render() {
        let registry = PromptRegistry::new();
        let template = JinjaTemplate::new("greeting", "Hello, {{ name }}!").unwrap();
        registry.register(template);

        let result = registry.render("greeting", &json!({ "name": "World" })).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_with_lang() {
        let registry = PromptRegistry::with_language(Language::English);
        let template =
            JinjaTemplate::bilingual("greeting", "Hello, {{ name }}!", "你好，{{ name }}！")
                .unwrap();
        registry.register(template);

        // Render with explicit Chinese
        let result = registry
            .render_with_lang("greeting", &Language::Chinese, &json!({ "name": "世界" }))
            .unwrap();
        assert_eq!(result, "你好，世界！");
    }

    #[test]
    fn test_render_not_found() {
        let registry = PromptRegistry::new();
        let result = registry.render("nonexistent", &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let registry = PromptRegistry::new();
        registry.register(JinjaTemplate::new("a", "A").unwrap());
        registry.register(JinjaTemplate::new("b", "B").unwrap());

        let list = registry.list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
    }

    #[test]
    fn test_len_and_is_empty() {
        let registry = PromptRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register(JinjaTemplate::new("test", "Hello").unwrap());
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_remove() {
        let registry = PromptRegistry::new();
        registry.register(JinjaTemplate::new("test", "Hello").unwrap());

        assert!(registry.contains("test"));
        let removed = registry.remove("test");
        assert!(removed.is_some());
        assert!(!registry.contains("test"));
    }

    #[test]
    fn test_clear() {
        let registry = PromptRegistry::new();
        registry.register(JinjaTemplate::new("a", "A").unwrap());
        registry.register(JinjaTemplate::new("b", "B").unwrap());

        assert_eq!(registry.len(), 2);
        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_register_all() {
        let registry = PromptRegistry::new();
        let templates = vec![
            JinjaTemplate::new("a", "A").unwrap(),
            JinjaTemplate::new("b", "B").unwrap(),
        ];

        registry.register_all(templates);
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_replace_template() {
        let registry = PromptRegistry::new();
        registry.register(JinjaTemplate::new("test", "Version 1").unwrap());

        let result = registry.render("test", &json!({})).unwrap();
        assert_eq!(result, "Version 1");

        // Replace with new version
        registry.register(JinjaTemplate::new("test", "Version 2").unwrap());

        let result = registry.render("test", &json!({})).unwrap();
        assert_eq!(result, "Version 2");
    }

    #[test]
    fn test_debug() {
        let registry = PromptRegistry::new();
        registry.register(JinjaTemplate::new("test", "Hello").unwrap());

        let debug = format!("{:?}", registry);
        assert!(debug.contains("PromptRegistry"));
        assert!(debug.contains("English"));
    }
}
