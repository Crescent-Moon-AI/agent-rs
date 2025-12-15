//! Error types for stock analysis operations

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
    DataUnavailable {
        symbol: String,
        reason: String,
    },

    /// Rate limit exceeded for API
    #[error("Rate limit exceeded for {provider}")]
    RateLimitExceeded {
        provider: String,
    },

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

    /// Generic error
    #[error("{0}")]
    Other(String),
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
        assert_eq!(err.to_string(), "Data not available for AAPL: No data found");
    }

    #[test]
    fn test_error_conversion() {
        let stock_err = StockError::ApiError("Test error".to_string());
        let agent_err: agent_core::Error = stock_err.into();

        match agent_err {
            agent_core::Error::ProcessingFailed(msg) => {
                assert!(msg.contains("API error"));
            },
            _ => panic!("Expected ProcessingFailed variant"),
        }
    }
}
