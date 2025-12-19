//! Error types for stock analysis operations
//!
//! This module provides a comprehensive error handling system for stock analysis,
//! with proper error chaining and context preservation.

use thiserror::Error;

/// Stock analysis specific errors
#[derive(Debug, Error)]
pub enum StockError {
    /// API request failed
    #[error("API error: {0}")]
    ApiError(String),

    /// Invalid stock symbol provided
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    /// Data not available for the requested symbol
    #[error("Data not available for {symbol}: {reason}")]
    DataUnavailable { symbol: String, reason: String },

    /// Rate limit exceeded for API
    #[error("Rate limit exceeded for {provider}")]
    RateLimitExceeded { provider: String },

    /// Network or HTTP error
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Yahoo Finance API error
    #[error("Yahoo Finance error: {0}")]
    YahooFinanceError(String),

    /// Alpha Vantage API error
    #[error("Alpha Vantage error: {0}")]
    AlphaVantageError(String),

    /// Technical indicator calculation error
    #[error("Technical indicator error: {0}")]
    IndicatorError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// Agent routing error
    #[error("Routing error: {0}")]
    RoutingError(String),

    /// Agent execution error
    #[error("Agent execution error [{agent}]: {message}")]
    AgentError { agent: String, message: String },

    /// Conversation context error
    #[error("Conversation error: {0}")]
    ConversationError(String),

    /// Command parsing error
    #[error("Invalid command: {0}")]
    CommandError(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl StockError {
    /// Create an API error with context
    pub fn api(msg: impl Into<String>) -> Self {
        Self::ApiError(msg.into())
    }

    /// Create a data unavailable error
    pub fn data_unavailable(symbol: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::DataUnavailable {
            symbol: symbol.into(),
            reason: reason.into(),
        }
    }

    /// Create a rate limit error
    pub fn rate_limited(provider: impl Into<String>) -> Self {
        Self::RateLimitExceeded {
            provider: provider.into(),
        }
    }

    /// Create an agent error
    pub fn agent(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::AgentError {
            agent: name.into(),
            message: message.into(),
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_) | Self::RateLimitExceeded { .. } | Self::Timeout(_)
        )
    }

    /// Check if the error is due to invalid input
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidSymbol(_) | Self::CommandError(_) | Self::ConfigError(_)
        )
    }
}

/// Result type alias for stock operations
pub type Result<T> = std::result::Result<T, StockError>;

/// Convert StockError to agent_core::Error
impl From<StockError> for agent_core::Error {
    fn from(err: StockError) -> Self {
        agent_core::Error::ProcessingFailed(err.to_string())
    }
}

/// Convert agent_core::Error to StockError
impl From<agent_core::Error> for StockError {
    fn from(err: agent_core::Error) -> Self {
        StockError::Other(err.to_string())
    }
}

/// Convert anyhow::Error to StockError
impl From<anyhow::Error> for StockError {
    fn from(err: anyhow::Error) -> Self {
        StockError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = StockError::InvalidSymbol("INVALID".to_string());
        assert_eq!(err.to_string(), "Invalid symbol: INVALID");

        let err = StockError::DataUnavailable {
            symbol: "AAPL".to_string(),
            reason: "No data found".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Data not available for AAPL: No data found"
        );
    }

    #[test]
    fn test_error_conversion() {
        let stock_err = StockError::ApiError("Test error".to_string());
        let agent_err: agent_core::Error = stock_err.into();

        match agent_err {
            agent_core::Error::ProcessingFailed(msg) => {
                assert!(msg.contains("API error"));
            }
            _ => panic!("Expected ProcessingFailed variant"),
        }
    }
}
