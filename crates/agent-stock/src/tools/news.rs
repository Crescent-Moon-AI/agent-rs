//! Tool for fetching stock news and sentiment

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::{AlphaVantageClient, FinnhubClient};
use crate::cache::{CacheKey, StockCache};
use crate::config::{NewsProvider, StockConfig};
use crate::error::Result;

/// Tool for fetching stock news
pub struct NewsTool {
    cache: StockCache,
    config: Arc<StockConfig>,
    finnhub_client: Option<FinnhubClient>,
    alpha_vantage_client: Option<AlphaVantageClient>,
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
        // Initialize Finnhub client if configured
        let finnhub_client = config.finnhub_api_key.as_ref().map(|key| {
            FinnhubClient::new(key.clone(), 60) // Free tier: 60 req/min
        });

        // Initialize Alpha Vantage client if configured
        let alpha_vantage_client = config
            .alpha_vantage_api_key
            .as_ref()
            .map(|key| AlphaVantageClient::new(key.clone(), config.alpha_vantage_rate_limit));

        Self {
            cache,
            config,
            finnhub_client,
            alpha_vantage_client,
        }
    }

    /// Fetch news for a symbol
    async fn fetch_news(&self, params: NewsParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();

        // Create cache key
        let cache_key = CacheKey::new(&symbol, "news", json!({"limit": params.limit}));

        // Try to get from cache
        let result = self
            .cache
            .get_or_fetch(cache_key, || async {
                match self.config.news_provider {
                    NewsProvider::Mock => self.fetch_mock_news(&symbol, params.limit).await,
                    NewsProvider::Finnhub => self.fetch_finnhub_news(&symbol, params.limit).await,
                    NewsProvider::AlphaVantage => {
                        self.fetch_alpha_vantage_news(&symbol, params.limit).await
                    }
                }
            })
            .await?;

        Ok(result)
    }

    /// Fetch mock news data (for testing)
    async fn fetch_mock_news(&self, symbol: &str, limit: usize) -> Result<Value> {
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

        let limited_news: Vec<_> = mock_news.into_iter().take(limit).collect();
        Ok(self.build_news_response(symbol, limited_news))
    }

    /// Fetch news from Finnhub
    async fn fetch_finnhub_news(&self, symbol: &str, limit: usize) -> Result<Value> {
        let client = self.finnhub_client.as_ref().ok_or_else(|| {
            crate::error::StockError::ConfigError("Finnhub API key not configured".to_string())
        })?;

        // Calculate date range (last 30 days)
        let to = chrono::Utc::now();
        let from = to - chrono::Duration::days(30);
        let from_str = from.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        let articles = client.get_company_news(symbol, &from_str, &to_str).await?;

        // Convert Finnhub articles to standardized format
        let mut news: Vec<Value> = articles
            .into_iter()
            .take(limit)
            .map(|article| {
                json!({
                    "title": article.headline,
                    "source": article.source,
                    "published_at": chrono::DateTime::from_timestamp(article.datetime, 0)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default(),
                    "summary": article.summary,
                    "sentiment": "neutral", // Finnhub doesn't provide sentiment in free tier
                    "sentiment_score": 0.0,
                    "url": article.url,
                    "image": article.image,
                })
            })
            .collect();

        // If no company-specific news found, try market news
        if news.is_empty() {
            let market_news = client.get_market_news("general").await?;
            news = market_news
                .into_iter()
                .take(limit)
                .map(|article| {
                    json!({
                        "title": article.headline,
                        "source": article.source,
                        "published_at": chrono::DateTime::from_timestamp(article.datetime, 0)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_default(),
                        "summary": article.summary,
                        "sentiment": "neutral",
                        "sentiment_score": 0.0,
                        "url": article.url,
                        "image": article.image,
                    })
                })
                .collect();
        }

        Ok(self.build_news_response(symbol, news))
    }

    /// Fetch news from Alpha Vantage
    async fn fetch_alpha_vantage_news(&self, symbol: &str, limit: usize) -> Result<Value> {
        let client = self.alpha_vantage_client.as_ref().ok_or_else(|| {
            crate::error::StockError::ConfigError(
                "Alpha Vantage API key not configured".to_string(),
            )
        })?;

        let response = client
            .get_news_sentiment(symbol, None, None, Some(limit as u32))
            .await?;

        // Convert Alpha Vantage articles to standardized format
        let news: Vec<Value> = response
            .feed
            .into_iter()
            .map(|article| {
                // Calculate sentiment label from score
                let (sentiment_label, sentiment_score) =
                    if let Some(score) = article.overall_sentiment_score {
                        let label = if score > 0.15 {
                            "positive"
                        } else if score < -0.15 {
                            "negative"
                        } else {
                            "neutral"
                        };
                        (label, score)
                    } else {
                        ("neutral", 0.0)
                    };

                json!({
                    "title": article.title,
                    "source": article.source,
                    "published_at": article.time_published,
                    "summary": article.summary,
                    "sentiment": sentiment_label,
                    "sentiment_score": sentiment_score,
                    "url": article.url,
                    "image": article.banner_image,
                    "topics": article.topics.iter()
                        .map(|t| json!({"topic": t.topic, "relevance": t.relevance_score}))
                        .collect::<Vec<_>>(),
                    "ticker_sentiment": article.ticker_sentiment.map(|ts| {
                        ts.iter().map(|t| json!({
                            "ticker": t.ticker,
                            "relevance": t.relevance_score,
                            "sentiment_score": t.ticker_sentiment_score,
                            "sentiment_label": t.ticker_sentiment_label,
                        })).collect::<Vec<_>>()
                    }),
                })
            })
            .collect();

        Ok(self.build_news_response(symbol, news))
    }

    /// Build standardized news response with sentiment analysis
    fn build_news_response(&self, symbol: &str, articles: Vec<Value>) -> Value {
        // Calculate overall sentiment
        let sentiments: Vec<&str> = articles
            .iter()
            .filter_map(|n| n.get("sentiment").and_then(|s| s.as_str()))
            .collect();

        let positive_count = sentiments.iter().filter(|&&s| s == "positive").count();
        let negative_count = sentiments.iter().filter(|&&s| s == "negative").count();
        let neutral_count = sentiments.len() - positive_count - negative_count;

        let overall_sentiment = if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        };

        // Calculate average sentiment score
        let avg_score: f64 = articles
            .iter()
            .filter_map(|n| n.get("sentiment_score").and_then(serde_json::Value::as_f64))
            .sum::<f64>()
            / articles.len().max(1) as f64;

        json!({
            "symbol": symbol,
            "news_count": articles.len(),
            "articles": articles,
            "overall_sentiment": overall_sentiment,
            "average_sentiment_score": avg_score,
            "sentiment_breakdown": {
                "positive": positive_count,
                "negative": negative_count,
                "neutral": neutral_count,
            },
            "provider": format!("{:?}", self.config.news_provider),
        })
    }
}

#[async_trait]
impl Tool for NewsTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: NewsParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {e}"))
        })?;

        self.fetch_news(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "news"
    }

    fn description(&self) -> &'static str {
        "Fetch recent news articles and sentiment analysis for a stock symbol. \
         Returns news headlines, summaries, sentiment scores, and overall market sentiment. \
         Supports multiple news providers: Mock (testing), Finnhub (60 req/min), and Alpha Vantage (with sentiment analysis)."
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
    async fn test_fetch_mock_news() {
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
        assert_eq!(data["provider"], "Mock");
    }
}
