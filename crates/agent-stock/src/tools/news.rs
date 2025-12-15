//! Tool for fetching stock news and sentiment

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::Result;

/// Tool for fetching stock news
pub struct NewsTool {
    client: reqwest::Client,
    cache: StockCache,
    config: Arc<StockConfig>,
}

#[derive(Debug, Deserialize)]
struct NewsParams {
    symbol: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

impl NewsTool {
    /// Create a new news tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        Self {
            client: reqwest::Client::new(),
            cache,
            config,
        }
    }

    /// Fetch news for a symbol
    async fn fetch_news(&self, params: NewsParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();

        // Create cache key
        let cache_key = CacheKey::new(&symbol, "news", json!({"limit": params.limit}));

        // Try to get from cache
        let result = self.cache.get_or_fetch(cache_key, || async {
            // For now, return mock news data
            // In production, integrate with a news API like NewsAPI, Finnhub, or Alpha Vantage News
            let mock_news = vec![
                json!({
                    "title": format!("{} Stock Analysis Update", symbol),
                    "source": "Market News",
                    "published_at": chrono::Utc::now().to_rfc3339(),
                    "summary": format!("Latest market analysis and trends for {}", symbol),
                    "sentiment": "neutral",
                    "sentiment_score": 0.0,
                    "url": "https://example.com/news1"
                }),
                json!({
                    "title": format!("{} Quarterly Earnings Report", symbol),
                    "source": "Financial Times",
                    "published_at": (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339(),
                    "summary": format!("{} reports quarterly earnings", symbol),
                    "sentiment": "positive",
                    "sentiment_score": 0.6,
                    "url": "https://example.com/news2"
                }),
            ];

            let limited_news: Vec<_> = mock_news.into_iter().take(params.limit).collect();

            // Calculate overall sentiment
            let sentiments: Vec<&str> = limited_news.iter()
                .filter_map(|n| n.get("sentiment").and_then(|s| s.as_str()))
                .collect();

            let positive_count = sentiments.iter().filter(|&&s| s == "positive").count();
            let negative_count = sentiments.iter().filter(|&&s| s == "negative").count();

            let overall_sentiment = if positive_count > negative_count {
                "positive"
            } else if negative_count > positive_count {
                "negative"
            } else {
                "neutral"
            };

            Ok::<_, crate::error::StockError>(json!({
                "symbol": symbol,
                "news_count": limited_news.len(),
                "articles": limited_news,
                "overall_sentiment": overall_sentiment,
                "sentiment_breakdown": {
                    "positive": positive_count,
                    "negative": negative_count,
                    "neutral": sentiments.len() - positive_count - negative_count,
                },
                "note": "News data is currently mocked. Integrate with a real news API for production use."
            }))
        }).await?;

        Ok(result)
    }
}

#[async_trait]
impl Tool for NewsTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: NewsParams = serde_json::from_value(params)
            .map_err(|e| agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e)))?;

        self.fetch_news(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "news"
    }

    fn description(&self) -> &str {
        "Fetch recent news articles and sentiment analysis for a stock symbol. \
         Returns news headlines, summaries, and overall market sentiment."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of news articles to fetch",
                    "default": 10
                }
            },
            "required": ["symbol"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_fetch_news() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(300));
        let tool = NewsTool::new(config, cache);

        let params = json!({
            "symbol": "AAPL",
            "limit": 5
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data["symbol"], "AAPL");
        assert!(data["articles"].is_array());
    }
}
