//! Error types for MCP operations

use thiserror::Error;

/// Errors that can occur during MCP operations
#[derive(Error, Debug)]
pub enum MCPError {
    /// MCP connection failed
    #[error("MCP connection failed: {0}")]
    ConnectionFailed(String),

    /// MCP initialization failed
    #[error("MCP initialization failed: {0}")]
    InitializationFailed(String),

    /// Not connected to MCP server
    #[error("Not connected to MCP server")]
    NotConnected,

    /// MCP not initialized
    #[error("MCP not initialized: {0}")]
    NotInitialized(String),

    /// MCP disconnection failed
    #[error("MCP disconnection failed: {0}")]
    DisconnectionFailed(String),

    /// MCP request failed
    #[error("MCP request failed: {0}")]
    RequestFailed(String),

    /// MCP tool call failed
    #[error("MCP tool call failed: {0}")]
    ToolCallFailed(String),

    /// MCP server not found
    #[error("MCP server not found: {0}")]
    ServerNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid MCP URI
    #[error("Invalid MCP URI: {0}")]
    InvalidUri(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Internal error
    #[error("Internal error: {0}")]
    InternalError(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Environment variable error
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),

    /// Invalid pattern error
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Convert MCPError to agent_core::Error
impl From<MCPError> for agent_core::Error {
    fn from(err: MCPError) -> Self {
        agent_core::Error::ProcessingFailed(err.to_string())
    }
}
