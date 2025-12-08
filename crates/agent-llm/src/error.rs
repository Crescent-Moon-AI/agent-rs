//! Error types for LLM operations

use thiserror::Error;

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LLMError>;

/// Errors that can occur during LLM operations
#[derive(Error, Debug)]
pub enum LLMError {
    /// API request failed
    #[error("API request failed: {0}")]
    RequestFailed(String),

    /// Invalid API key or authentication failed
    #[error("Invalid API key or authentication failed")]
    AuthenticationFailed,

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Unexpected response format
    #[error("Unexpected response format: {0}")]
    UnexpectedResponse(String),

    /// Provider-specific error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
