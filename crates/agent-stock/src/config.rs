//! Configuration for stock analysis operations

use crate::error::{Result, StockError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Data provider for stock information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DataProvider {
    /// Yahoo Finance (default, no API key required)
    #[default]
    Yahoo,
    /// Alpha Vantage (requires API key)
    AlphaVantage,
}

/// News provider for market news and sentiment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NewsProvider {
    /// Mock data for testing
    #[default]
    Mock,
    /// Finnhub.io (60 requests/minute free tier)
    Finnhub,
    /// Alpha Vantage News & Sentiment API
    AlphaVantage,
}

/// Language for agent responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResponseLanguage {
    /// Chinese (default)
    #[default]
    Chinese,
    /// English
    English,
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

    /// Cache TTL for earnings data
    pub cache_ttl_earnings: Duration,

    /// Cache TTL for macro economic data
    pub cache_ttl_macro: Duration,

    /// Cache TTL for sector data
    pub cache_ttl_sector: Duration,

    /// Maximum number of retries for API calls
    pub max_retries: u32,

    /// Initial backoff duration for retries
    pub retry_backoff_base: Duration,

    /// Request timeout duration
    pub request_timeout: Duration,

    /// Alpha Vantage API key (optional)
    pub alpha_vantage_api_key: Option<String>,

    /// Alpha Vantage API rate limit (requests per minute)
    pub alpha_vantage_rate_limit: u32,

    /// News data provider
    pub news_provider: NewsProvider,

    /// Finnhub.io API key (optional)
    pub finnhub_api_key: Option<String>,

    /// FRED API key (for macroeconomic data)
    pub fred_api_key: Option<String>,

    /// SEC EDGAR User-Agent (required for SEC API)
    pub sec_user_agent: String,

    /// SEC contact email (required for SEC API)
    pub sec_contact_email: String,

    /// LLM model for analysis
    pub model: String,

    /// Temperature for LLM responses
    pub temperature: f32,

    /// Maximum tokens per response
    pub max_tokens: usize,

    /// Language for agent responses
    pub response_language: ResponseLanguage,
}

impl Default for StockConfig {
    fn default() -> Self {
        Self {
            default_provider: DataProvider::Yahoo,
            cache_ttl_realtime: Duration::from_secs(60), // 1 minute
            cache_ttl_fundamental: Duration::from_secs(3600), // 1 hour
            cache_ttl_news: Duration::from_secs(300),    // 5 minutes
            cache_ttl_earnings: Duration::from_secs(86400), // 24 hours
            cache_ttl_macro: Duration::from_secs(3600),  // 1 hour
            cache_ttl_sector: Duration::from_secs(1800), // 30 minutes
            max_retries: 3,
            retry_backoff_base: Duration::from_secs(1),
            request_timeout: Duration::from_secs(30),
            alpha_vantage_api_key: None,
            alpha_vantage_rate_limit: 5, // Free tier: 5 requests/minute
            news_provider: NewsProvider::Mock,
            finnhub_api_key: None,
            fred_api_key: None,
            sec_user_agent: "agent-stock".to_string(),
            sec_contact_email: "agent-stock@example.com".to_string(),
            model: "claude-opus-4-5-20251101".to_string(),
            temperature: 0.5,
            max_tokens: 4096,
            response_language: ResponseLanguage::Chinese,
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
            && self.alpha_vantage_api_key.is_none()
        {
            return Err(StockError::ConfigError(
                "Alpha Vantage API key required when using AlphaVantage provider".to_string(),
            ));
        }

        if self.news_provider == NewsProvider::Finnhub && self.finnhub_api_key.is_none() {
            return Err(StockError::ConfigError(
                "Finnhub API key required when using Finnhub provider. Set FINNHUB_API_KEY environment variable.".to_string(),
            ));
        }

        if self.news_provider == NewsProvider::AlphaVantage && self.alpha_vantage_api_key.is_none()
        {
            return Err(StockError::ConfigError(
                "Alpha Vantage API key required when using AlphaVantage news provider".to_string(),
            ));
        }

        if self.max_retries == 0 {
            return Err(StockError::ConfigError(
                "max_retries must be greater than 0".to_string(),
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
    cache_ttl_earnings: Option<Duration>,
    cache_ttl_macro: Option<Duration>,
    cache_ttl_sector: Option<Duration>,
    max_retries: Option<u32>,
    retry_backoff_base: Option<Duration>,
    request_timeout: Option<Duration>,
    alpha_vantage_api_key: Option<String>,
    alpha_vantage_rate_limit: Option<u32>,
    news_provider: Option<NewsProvider>,
    finnhub_api_key: Option<String>,
    fred_api_key: Option<String>,
    sec_user_agent: Option<String>,
    sec_contact_email: Option<String>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    response_language: Option<ResponseLanguage>,
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

    /// Set Alpha Vantage rate limit (requests per minute)
    pub fn alpha_vantage_rate_limit(mut self, limit: u32) -> Self {
        self.alpha_vantage_rate_limit = Some(limit);
        self
    }

    /// Set news provider
    pub fn news_provider(mut self, provider: NewsProvider) -> Self {
        self.news_provider = Some(provider);
        self
    }

    /// Set Finnhub API key
    pub fn finnhub_api_key(mut self, key: impl Into<String>) -> Self {
        self.finnhub_api_key = Some(key.into());
        self
    }

    /// Load Finnhub API key from environment
    pub fn with_env_finnhub_key(mut self) -> Self {
        if let Ok(key) = std::env::var("FINNHUB_API_KEY") {
            self.finnhub_api_key = Some(key);
        }
        self
    }

    /// Set FRED API key
    pub fn fred_api_key(mut self, key: impl Into<String>) -> Self {
        self.fred_api_key = Some(key.into());
        self
    }

    /// Load FRED API key from environment
    pub fn with_env_fred_key(mut self) -> Self {
        if let Ok(key) = std::env::var("FRED_API_KEY") {
            self.fred_api_key = Some(key);
        }
        self
    }

    /// Set SEC User-Agent
    pub fn sec_user_agent(mut self, agent: impl Into<String>) -> Self {
        self.sec_user_agent = Some(agent.into());
        self
    }

    /// Set SEC contact email
    pub fn sec_contact_email(mut self, email: impl Into<String>) -> Self {
        self.sec_contact_email = Some(email.into());
        self
    }

    /// Load all API keys from environment variables
    pub fn with_env_all_keys(self) -> Self {
        self.with_env_api_key()
            .with_env_finnhub_key()
            .with_env_fred_key()
    }

    /// Load news provider from environment (NEWS_PROVIDER=Mock|Finnhub|AlphaVantage)
    pub fn with_env_news_provider(mut self) -> Self {
        if let Ok(provider) = std::env::var("NEWS_PROVIDER") {
            self.news_provider = match provider.to_lowercase().as_str() {
                "finnhub" => Some(NewsProvider::Finnhub),
                "alphavantage" | "alpha_vantage" => Some(NewsProvider::AlphaVantage),
                "mock" => Some(NewsProvider::Mock),
                _ => None,
            };
        }
        self
    }

    /// Set the LLM model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set the maximum tokens
    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set the response language
    pub fn response_language(mut self, language: ResponseLanguage) -> Self {
        self.response_language = Some(language);
        self
    }

    /// Load model configuration from environment variables
    pub fn from_env_model(mut self) -> Self {
        if let Ok(model) = std::env::var("STOCK_MODEL") {
            self.model = Some(model);
        }
        if let Ok(temp) = std::env::var("STOCK_TEMPERATURE") {
            if let Ok(temp_val) = temp.parse() {
                self.temperature = Some(temp_val);
            }
        }
        if let Ok(tokens) = std::env::var("STOCK_MAX_TOKENS") {
            if let Ok(token_val) = tokens.parse() {
                self.max_tokens = Some(token_val);
            }
        }
        if let Ok(lang) = std::env::var("STOCK_RESPONSE_LANGUAGE") {
            self.response_language = match lang.to_lowercase().as_str() {
                "chinese" | "zh" | "中文" => Some(ResponseLanguage::Chinese),
                "english" | "en" => Some(ResponseLanguage::English),
                _ => None,
            };
        }
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<StockConfig> {
        let defaults = StockConfig::default();

        let config = StockConfig {
            default_provider: self.default_provider.unwrap_or(defaults.default_provider),
            cache_ttl_realtime: self
                .cache_ttl_realtime
                .unwrap_or(defaults.cache_ttl_realtime),
            cache_ttl_fundamental: self
                .cache_ttl_fundamental
                .unwrap_or(defaults.cache_ttl_fundamental),
            cache_ttl_news: self.cache_ttl_news.unwrap_or(defaults.cache_ttl_news),
            cache_ttl_earnings: self.cache_ttl_earnings.unwrap_or(defaults.cache_ttl_earnings),
            cache_ttl_macro: self.cache_ttl_macro.unwrap_or(defaults.cache_ttl_macro),
            cache_ttl_sector: self.cache_ttl_sector.unwrap_or(defaults.cache_ttl_sector),
            max_retries: self.max_retries.unwrap_or(defaults.max_retries),
            retry_backoff_base: self
                .retry_backoff_base
                .unwrap_or(defaults.retry_backoff_base),
            request_timeout: self.request_timeout.unwrap_or(defaults.request_timeout),
            alpha_vantage_api_key: self.alpha_vantage_api_key,
            alpha_vantage_rate_limit: self
                .alpha_vantage_rate_limit
                .unwrap_or(defaults.alpha_vantage_rate_limit),
            news_provider: self.news_provider.unwrap_or(defaults.news_provider),
            finnhub_api_key: self.finnhub_api_key,
            fred_api_key: self.fred_api_key,
            sec_user_agent: self.sec_user_agent.unwrap_or(defaults.sec_user_agent),
            sec_contact_email: self.sec_contact_email.unwrap_or(defaults.sec_contact_email),
            model: self.model.unwrap_or(defaults.model),
            temperature: self.temperature.unwrap_or(defaults.temperature),
            max_tokens: self.max_tokens.unwrap_or(defaults.max_tokens),
            response_language: self.response_language.unwrap_or(defaults.response_language),
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
