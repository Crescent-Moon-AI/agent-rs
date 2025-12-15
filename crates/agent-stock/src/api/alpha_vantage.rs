//! Alpha Vantage API client

use crate::error::{Result, StockError};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

const BASE_URL: &str = "https://www.alphavantage.co/query";

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Alpha Vantage API client
#[derive(Debug, Clone)]
pub struct AlphaVantageClient {
    client: Client,
    api_key: String,
    rate_limiter: SharedRateLimiter,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
}

/// Company overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CompanyOverview {
    pub symbol: String,
    pub name: String,
    pub exchange: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    #[serde(rename = "MarketCapitalization")]
    pub market_cap: Option<String>,
    #[serde(rename = "PERatio")]
    pub pe_ratio: Option<String>,
    #[serde(rename = "DividendYield")]
    pub dividend_yield: Option<String>,
    #[serde(rename = "EPS")]
    pub eps: Option<String>,
    #[serde(rename = "BookValue")]
    pub book_value: Option<String>,
}

impl AlphaVantageClient {
    /// Create a new Alpha Vantage client with API key and rate limit
    ///
    /// # Arguments
    /// * `api_key` - Alpha Vantage API key
    /// * `rate_limit` - Maximum requests per minute (default: 5 for free tier)
    pub fn new(api_key: impl Into<String>, rate_limit: u32) -> Self {
        // Create rate limiter quota (requests per minute)
        let quota =
            Quota::per_minute(NonZeroU32::new(rate_limit).unwrap_or(NonZeroU32::new(5).unwrap()));
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            rate_limiter,
        }
    }

    /// Create from environment variable ALPHA_VANTAGE_API_KEY with default rate limit
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ALPHA_VANTAGE_API_KEY").map_err(|_| {
            StockError::ConfigError(
                "ALPHA_VANTAGE_API_KEY environment variable not set".to_string(),
            )
        })?;

        Ok(Self::new(api_key, 5)) // Default to free tier limit
    }

    /// Get intraday time series data
    pub async fn get_intraday(
        &self,
        symbol: &str,
        interval: &str, // 1min, 5min, 15min, 30min, 60min
    ) -> Result<Vec<TimeSeriesData>> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("function", "TIME_SERIES_INTRADAY");
        params.insert("symbol", symbol);
        params.insert("interval", interval);
        params.insert("apikey", &self.api_key);

        let response = self.client.get(BASE_URL).query(&params).send().await?;

        if !response.status().is_success() {
            return Err(StockError::AlphaVantageError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await?;

        // Check for API error messages
        if let Some(error) = data.get("Error Message") {
            return Err(StockError::AlphaVantageError(error.to_string()));
        }

        if let Some(_note) = data.get("Note") {
            return Err(StockError::RateLimitExceeded {
                provider: "Alpha Vantage".to_string(),
            });
        }

        // Parse time series data
        let series_key = format!("Time Series ({})", interval);
        let series = data.get(&series_key).ok_or_else(|| {
            StockError::AlphaVantageError("No time series data found".to_string())
        })?;

        let mut result = Vec::new();
        if let Some(obj) = series.as_object() {
            for (timestamp, values) in obj {
                let open = values["1. open"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let high = values["2. high"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let low = values["3. low"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let close = values["4. close"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let volume = values["5. volume"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                result.push(TimeSeriesData {
                    timestamp: timestamp.clone(),
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }
        }

        Ok(result)
    }

    /// Get daily time series data
    pub async fn get_daily(&self, symbol: &str) -> Result<Vec<TimeSeriesData>> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("function", "TIME_SERIES_DAILY");
        params.insert("symbol", symbol);
        params.insert("apikey", &self.api_key);

        let response = self.client.get(BASE_URL).query(&params).send().await?;

        let data: serde_json::Value = response.json().await?;

        // Check for errors
        if let Some(error) = data.get("Error Message") {
            return Err(StockError::AlphaVantageError(error.to_string()));
        }

        if let Some(_note) = data.get("Note") {
            return Err(StockError::RateLimitExceeded {
                provider: "Alpha Vantage".to_string(),
            });
        }

        // Parse time series data
        let series = data
            .get("Time Series (Daily)")
            .ok_or_else(|| StockError::AlphaVantageError("No daily data found".to_string()))?;

        let mut result = Vec::new();
        if let Some(obj) = series.as_object() {
            for (timestamp, values) in obj {
                let open = values["1. open"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let high = values["2. high"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let low = values["3. low"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let close = values["4. close"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                let volume = values["5. volume"]
                    .as_str()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);

                result.push(TimeSeriesData {
                    timestamp: timestamp.clone(),
                    open,
                    high,
                    low,
                    close,
                    volume,
                });
            }
        }

        Ok(result)
    }

    /// Get company overview and fundamental data
    pub async fn get_company_overview(&self, symbol: &str) -> Result<CompanyOverview> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("function", "OVERVIEW");
        params.insert("symbol", symbol);
        params.insert("apikey", &self.api_key);

        let response = self.client.get(BASE_URL).query(&params).send().await?;

        let data: serde_json::Value = response.json().await?;

        // Check for errors
        if let Some(error) = data.get("Error Message") {
            return Err(StockError::AlphaVantageError(error.to_string()));
        }

        if let Some(_note) = data.get("Note") {
            return Err(StockError::RateLimitExceeded {
                provider: "Alpha Vantage".to_string(),
            });
        }

        // Check if data is empty (symbol not found)
        if data.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            return Err(StockError::InvalidSymbol(symbol.to_string()));
        }

        let overview: CompanyOverview = serde_json::from_value(data)?;
        Ok(overview)
    }

    /// Get global quote (current price data)
    pub async fn get_quote(&self, symbol: &str) -> Result<serde_json::Value> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("function", "GLOBAL_QUOTE");
        params.insert("symbol", symbol);
        params.insert("apikey", &self.api_key);

        let response = self.client.get(BASE_URL).query(&params).send().await?;

        let data: serde_json::Value = response.json().await?;

        // Check for errors
        if let Some(error) = data.get("Error Message") {
            return Err(StockError::AlphaVantageError(error.to_string()));
        }

        if let Some(_note) = data.get("Note") {
            return Err(StockError::RateLimitExceeded {
                provider: "Alpha Vantage".to_string(),
            });
        }

        Ok(data)
    }

    /// Search for symbols
    pub async fn search_symbol(&self, keywords: &str) -> Result<Vec<serde_json::Value>> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("function", "SYMBOL_SEARCH");
        params.insert("keywords", keywords);
        params.insert("apikey", &self.api_key);

        let response = self.client.get(BASE_URL).query(&params).send().await?;

        let data: serde_json::Value = response.json().await?;

        if let Some(matches) = data.get("bestMatches") {
            if let Some(arr) = matches.as_array() {
                return Ok(arr.clone());
            }
        }

        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AlphaVantageClient::new("test_key", 5);
        assert_eq!(client.api_key, "test_key");
    }

    #[tokio::test]
    #[ignore] // Requires API key and network access
    async fn test_get_company_overview() {
        let client = AlphaVantageClient::from_env().unwrap();
        let overview = client.get_company_overview("AAPL").await;
        assert!(overview.is_ok());

        let overview = overview.unwrap();
        assert_eq!(overview.symbol, "AAPL");
        assert!(overview.name.contains("Apple"));
    }

    #[tokio::test]
    #[ignore] // Requires API key and network access
    async fn test_get_daily() {
        let client = AlphaVantageClient::from_env().unwrap();
        let data = client.get_daily("AAPL").await;
        assert!(data.is_ok());

        let data = data.unwrap();
        assert!(!data.is_empty());
    }
}
