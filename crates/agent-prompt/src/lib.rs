//! Prompt template management for agent-rs
//!
//! This crate provides a flexible and type-safe system for managing prompt templates
//! with multi-language support, variable interpolation, and template composition.
//!
//! # Features
//!
//! - **Multi-language support**: Templates can have variants for different languages
//! - **Variable interpolation**: Use Jinja2 syntax (`{{ variable }}`) for dynamic content
//! - **Template registry**: Centralized management of templates with thread-safe access
//! - **File loading**: Optional support for loading templates from files
//! - **Builder pattern**: Fluent API for constructing prompts programmatically
//!
//! # Quick Start
//!
//! ```
//! use agent_prompt::{JinjaTemplate, Language, PromptRegistry, PromptTemplate};
//! use serde_json::json;
//!
//! // Create a bilingual template
//! let template = JinjaTemplate::bilingual(
//!     "greeting",
//!     "Hello, {{ name }}!",
//!     "你好，{{ name }}！",
//! ).unwrap();
//!
//! // Render for different languages
//! let en = template.render(&Language::English, &json!({ "name": "World" })).unwrap();
//! assert_eq!(en, "Hello, World!");
//!
//! let zh = template.render(&Language::Chinese, &json!({ "name": "世界" })).unwrap();
//! assert_eq!(zh, "你好，世界！");
//! ```
//!
//! # Using the Registry
//!
//! ```
//! use agent_prompt::{JinjaTemplate, Language, PromptRegistry};
//! use serde_json::json;
//!
//! let registry = PromptRegistry::with_language(Language::Chinese);
//!
//! // Register templates
//! let template = JinjaTemplate::bilingual(
//!     "analyzer",
//!     "Analyze {{ symbol }}",
//!     "分析 {{ symbol }}",
//! ).unwrap();
//! registry.register(template);
//!
//! // Render using default language
//! let prompt = registry.render("analyzer", &json!({ "symbol": "AAPL" })).unwrap();
//! assert_eq!(prompt, "分析 AAPL");
//! ```
//!
//! # Using the Builder
//!
//! ```
//! use agent_prompt::PromptBuilder;
//!
//! let prompt = PromptBuilder::new()
//!     .text("You are a helpful assistant.")
//!     .blank_line()
//!     .section("Your Capabilities")
//!     .bullet("Answer questions")
//!     .bullet("Provide explanations")
//!     .build();
//!
//! assert!(prompt.contains("## Your Capabilities"));
//! ```
//!
//! # Feature Flags
//!
//! - `file-loader`: Enable loading templates from files
//! - `core-integration`: Integration with agent-core error types

mod builder;
mod error;
mod jinja;
mod language;
mod registry;
mod template;

#[cfg(feature = "file-loader")]
mod loader;

// Re-export core types
pub use builder::PromptBuilder;
pub use error::{PromptError, Result};
pub use jinja::{JinjaTemplate, JinjaTemplateBuilder};
pub use language::Language;
pub use registry::PromptRegistry;
pub use template::PromptTemplate;

#[cfg(feature = "file-loader")]
pub use loader::FileLoader;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::builder::PromptBuilder;
    pub use crate::error::{PromptError, Result};
    pub use crate::jinja::JinjaTemplate;
    pub use crate::language::Language;
    pub use crate::registry::PromptRegistry;
    pub use crate::template::PromptTemplate;

    #[cfg(feature = "file-loader")]
    pub use crate::loader::FileLoader;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_basic_usage() {
        let template =
            JinjaTemplate::bilingual("test", "Hello, {{ name }}!", "你好，{{ name }}！").unwrap();

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
    fn test_registry_usage() {
        let registry = PromptRegistry::with_language(Language::Chinese);

        let template =
            JinjaTemplate::bilingual("analyzer", "Analyze {{ symbol }}", "分析 {{ symbol }}")
                .unwrap();

        registry.register(template);

        let prompt = registry
            .render("analyzer", &json!({ "symbol": "AAPL" }))
            .unwrap();
        assert_eq!(prompt, "分析 AAPL");
    }

    #[test]
    fn test_builder_usage() {
        let prompt = PromptBuilder::new()
            .text("Line 1")
            .newline()
            .text("Line 2")
            .build();

        assert_eq!(prompt, "Line 1\nLine 2");
    }

    #[test]
    fn test_fallback() {
        let template = JinjaTemplate::new("test", "English only").unwrap();

        // Request Chinese but fallback to English
        let result = template
            .render_with_fallback(&Language::Chinese, &json!({}))
            .unwrap();
        assert_eq!(result, "English only");
    }
}
