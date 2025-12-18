//! Error types for prompt operations

use thiserror::Error;

/// Result type for prompt operations
pub type Result<T> = std::result::Result<T, PromptError>;

/// Errors that can occur during prompt operations
#[derive(Error, Debug)]
pub enum PromptError {
    /// Template not found for the specified language
    #[error("Template '{name}' not found for language '{language}': {detail}")]
    TemplateNotFound {
        name: String,
        language: String,
        detail: String,
    },

    /// Template parsing failed
    #[error("Failed to parse template '{name}' for language '{language}': {detail}")]
    TemplateParseFailed {
        name: String,
        language: String,
        detail: String,
    },

    /// Template rendering failed
    #[error("Failed to render template '{name}': {detail}")]
    RenderError { name: String, detail: String },

    /// No templates provided when building
    #[error("No templates provided for '{0}'")]
    NoTemplatesProvided(String),

    /// No language available for the template
    #[error("No language available for template '{0}'")]
    NoLanguageAvailable(String),

    /// Template not registered in registry
    #[error("Template '{0}' not registered")]
    TemplateNotRegistered(String),

    /// Lock error for thread safety
    #[error("Lock error: {0}")]
    LockError(String),

    /// File loading error
    #[error("Failed to load template file '{path}': {detail}")]
    FileLoadError { path: String, detail: String },

    /// Variable serialization error
    #[error("Failed to serialize variables: {0}")]
    SerializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(feature = "core-integration")]
impl From<PromptError> for agent_core::Error {
    fn from(err: PromptError) -> Self {
        agent_core::Error::ProcessingFailed(err.to_string())
    }
}
