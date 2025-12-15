//! Configuration for stock analysis operations

use crate::error::{Result, StockError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Data provider for stock information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataProvider {
    /// Yahoo Finance (default, no API key required)
    Yahoo,
    /// Alpha Vantage (requires API key)
    AlphaVantage,
}

impl Default for DataProvider {
    fn default() -> Self {
        Self::Yahoo
    }
}

/// Configuration for stock analysis operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockConfig {
    /// Default data provider to use
    pub default_provider: DataProvider,

    /// Cache TTL for real-time data (quotes, prices)
    pub cache_ttl_realtime: Duration,

    /// Cache TTL for fundamental data
    pub cache_ttl_fundamental: Duration,

    /// Cache TTL for news data
    pub cache_ttl_news: Duration,

    /// Maximum number of retries for API calls
    pub max_retries: u32,

    /// Initial backoff duration for retries
    pub retry_backoff_base: Duration,

    /// Request timeout duration
    pub request_timeout: Duration,

    /// Alpha Vantage API key (optional)
    pub alpha_vantage_api_key: Option<String>,
}

impl Default for StockConfig {
    fn default() -> Self {
        Self {
            default_provider: DataProvider::Yahoo,
            cache_ttl_realtime: Duration::from_secs(60),        // 1 minute
            cache_ttl_fundamental: Duration::from_secs(3600),   // 1 hour
            cache_ttl_news: Duration::from_secs(300),           // 5 minutes
            max_retries: 3,
            retry_backoff_base: Duration::from_secs(1),
            request_timeout: Duration::from_secs(30),
            alpha_vantage_api_key: None,
        }
    }
}

impl StockConfig {
    /// Create a new configuration builder
    pub fn builder() -> StockConfigBuilder {
        StockConfigBuilder::default()
    }

    /// Load Alpha Vantage API key from environment
    pub fn with_env_api_key(mut self) -> Result<Self> {
        if let Ok(key) = std::env::var("ALPHA_VANTAGE_API_KEY") {
            self.alpha_vantage_api_key = Some(key);
        }
        Ok(self)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.default_provider == DataProvider::AlphaVantage
            && self.alpha_vantage_api_key.is_none() {
            return Err(StockError::ConfigError(
                "Alpha Vantage API key required when using AlphaVantage provider".to_string()
            ));
        }

        if self.max_retries == 0 {
            return Err(StockError::ConfigError(
                "max_retries must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    /// Get retry backoff duration for attempt number
    pub fn retry_backoff(&self, attempt: u32) -> Duration {
        self.retry_backoff_base * 2_u32.pow(attempt)
    }
}

/// Builder for StockConfig
#[derive(Debug, Default)]
pub struct StockConfigBuilder {
    default_provider: Option<DataProvider>,
    cache_ttl_realtime: Option<Duration>,
    cache_ttl_fundamental: Option<Duration>,
    cache_ttl_news: Option<Duration>,
    max_retries: Option<u32>,
    retry_backoff_base: Option<Duration>,
    request_timeout: Option<Duration>,
    alpha_vantage_api_key: Option<String>,
}

impl StockConfigBuilder {
    /// Set the default data provider
    pub fn default_provider(mut self, provider: DataProvider) -> Self {
        self.default_provider = Some(provider);
        self
    }

    /// Set cache TTL for real-time data
    pub fn cache_ttl_realtime(mut self, duration: Duration) -> Self {
        self.cache_ttl_realtime = Some(duration);
        self
    }

    /// Set cache TTL for fundamental data
    pub fn cache_ttl_fundamental(mut self, duration: Duration) -> Self {
        self.cache_ttl_fundamental = Some(duration);
        self
    }

    /// Set cache TTL for news data
    pub fn cache_ttl_news(mut self, duration: Duration) -> Self {
        self.cache_ttl_news = Some(duration);
        self
    }

    /// Set maximum retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// Set retry backoff base duration
    pub fn retry_backoff_base(mut self, duration: Duration) -> Self {
        self.retry_backoff_base = Some(duration);
        self
    }

    /// Set request timeout
    pub fn request_timeout(mut self, duration: Duration) -> Self {
        self.request_timeout = Some(duration);
        self
    }

    /// Set Alpha Vantage API key
    pub fn alpha_vantage_api_key(mut self, key: impl Into<String>) -> Self {
        self.alpha_vantage_api_key = Some(key.into());
        self
    }

    /// Load Alpha Vantage API key from environment
    pub fn with_env_api_key(mut self) -> Self {
        if let Ok(key) = std::env::var("ALPHA_VANTAGE_API_KEY") {
            self.alpha_vantage_api_key = Some(key);
        }
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<StockConfig> {
        let defaults = StockConfig::default();

        let config = StockConfig {
            default_provider: self.default_provider.unwrap_or(defaults.default_provider),
            cache_ttl_realtime: self.cache_ttl_realtime.unwrap_or(defaults.cache_ttl_realtime),
            cache_ttl_fundamental: self.cache_ttl_fundamental.unwrap_or(defaults.cache_ttl_fundamental),
            cache_ttl_news: self.cache_ttl_news.unwrap_or(defaults.cache_ttl_news),
            max_retries: self.max_retries.unwrap_or(defaults.max_retries),
            retry_backoff_base: self.retry_backoff_base.unwrap_or(defaults.retry_backoff_base),
            request_timeout: self.request_timeout.unwrap_or(defaults.request_timeout),
            alpha_vantage_api_key: self.alpha_vantage_api_key,
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = StockConfig::default();
        assert_eq!(config.default_provider, DataProvider::Yahoo);
        assert_eq!(config.max_retries, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = StockConfig::builder()
            .default_provider(DataProvider::Yahoo)
            .max_retries(5)
            .request_timeout(Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.request_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_validation_alpha_vantage_no_key() {
        let config = StockConfig {
            default_provider: DataProvider::AlphaVantage,
            alpha_vantage_api_key: None,
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_alpha_vantage_with_key() {
        let config = StockConfig {
            default_provider: DataProvider::AlphaVantage,
            alpha_vantage_api_key: Some("test_key".to_string()),
            ..Default::default()
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_retry_backoff() {
        let config = StockConfig::default();
        assert_eq!(config.retry_backoff(0), Duration::from_secs(1));
        assert_eq!(config.retry_backoff(1), Duration::from_secs(2));
        assert_eq!(config.retry_backoff(2), Duration::from_secs(4));
    }
}
