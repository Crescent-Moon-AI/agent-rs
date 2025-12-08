//! Error types for agent-core

use thiserror::Error;

/// Result type alias for agent-core
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for agent operations
#[derive(Error, Debug)]
pub enum Error {
    /// Generic error message
    #[error("{0}")]
    Generic(String),

    /// Agent initialization failed
    #[error("Agent initialization failed: {0}")]
    InitializationFailed(String),

    /// Agent processing failed
    #[error("Agent processing failed: {0}")]
    ProcessingFailed(String),
}
