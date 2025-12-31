//! News API clients for market news and sentiment data

use crate::error::{Result, StockError};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Finnhub news article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinnhubNewsArticle {
    /// Article category
    pub category: String,
    /// Publish time (UNIX timestamp)
    pub datetime: i64,
    /// News headline
    pub headline: String,
    /// Unique article ID
    pub id: i64,
    /// Thumbnail image URL
    pub image: String,
    /// Related symbols
    pub related: String,
    /// News source
    pub source: String,
    /// Article summary
    pub summary: String,
    /// Article URL
    pub url: String,
}

/// Finnhub client for news API
pub struct FinnhubClient {
    client: Client,
    api_key: String,
    rate_limiter: SharedRateLimiter,
}

impl FinnhubClient {
    /// Create a new Finnhub client with rate limiting
    ///
    /// # Arguments
    /// * `api_key` - Finnhub API key
    /// * `rate_limit` - Requests per minute (free tier: 60, premium: 300+)
    pub fn new(api_key: impl Into<String>, rate_limit: u32) -> Self {
        let quota =
            Quota::per_minute(NonZeroU32::new(rate_limit).unwrap_or(NonZeroU32::new(60).unwrap()));
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            rate_limiter,
        }
    }

    /// Get company news for a specific symbol
    ///
    /// # Arguments
    /// * `symbol` - Stock symbol (e.g., "AAPL")
    /// * `from` - Start date (YYYY-MM-DD)
    /// * `to` - End date (YYYY-MM-DD)
    pub async fn get_company_news(
        &self,
        symbol: &str,
        from: &str,
        to: &str,
    ) -> Result<Vec<FinnhubNewsArticle>> {
        self.rate_limiter.until_ready().await;

        let url = format!(
            "https://finnhub.io/api/v1/company-news?symbol={}&from={}&to={}&token={}",
            symbol, from, to, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("Finnhub request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(StockError::ApiError(format!(
                "Finnhub API error {status}: {body}"
            )));
        }

        response
            .json::<Vec<FinnhubNewsArticle>>()
            .await
            .map_err(|e| StockError::ApiError(format!("Failed to parse Finnhub response: {e}")))
    }

    /// Get general market news
    ///
    /// # Arguments
    /// * `category` - News category (general, forex, crypto, merger)
    pub async fn get_market_news(&self, category: &str) -> Result<Vec<FinnhubNewsArticle>> {
        self.rate_limiter.until_ready().await;

        let url = format!(
            "https://finnhub.io/api/v1/news?category={}&token={}",
            category, self.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("Finnhub request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(StockError::ApiError(format!(
                "Finnhub API error {status}: {body}"
            )));
        }

        response
            .json::<Vec<FinnhubNewsArticle>>()
            .await
            .map_err(|e| StockError::ApiError(format!("Failed to parse Finnhub response: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finnhub_client_creation() {
        let client = FinnhubClient::new("test_key", 60);
        assert_eq!(client.api_key, "test_key");
    }
}
