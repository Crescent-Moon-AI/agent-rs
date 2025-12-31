//! Tool for fetching stock price data

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::YahooFinanceClient;
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::Result;

/// Tool for fetching stock price and quote data
pub struct StockDataTool {
    yahoo_client: YahooFinanceClient,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

#[derive(Debug, Deserialize)]
struct StockDataParams {
    symbol: String,
    #[serde(default)]
    range: Option<String>,
    #[serde(default)]
    include_historical: Option<bool>,
}

impl StockDataTool {
    /// Create a new stock data tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        Self {
            yahoo_client: YahooFinanceClient::new(),
            cache,
            _config: config,
        }
    }

    /// Fetch stock data with caching and retries
    async fn fetch_stock_data(&self, params: StockDataParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();
        let range = params.range.unwrap_or_else(|| "1d".to_string());
        let include_historical = params.include_historical.unwrap_or(false);

        // Create cache key
        let cache_key = CacheKey::new(
            &symbol,
            "stock_data",
            json!({ "range": &range, "historical": include_historical }),
        );

        // Try to get from cache
        let result = self
            .cache
            .get_or_fetch(cache_key, || async {
                // Fetch current quote
                let quote = self.yahoo_client.get_quote(&symbol).await?;

                let mut result = json!({
                    "symbol": symbol,
                    "current_quote": {
                        "timestamp": quote.timestamp.to_rfc3339(),
                        "open": quote.open,
                        "high": quote.high,
                        "low": quote.low,
                        "close": quote.close,
                        "volume": quote.volume,
                        "adjusted_close": quote.adjclose,
                    }
                });

                // Fetch historical data if requested
                if include_historical {
                    let historical = self
                        .yahoo_client
                        .get_historical_range(&symbol, &range)
                        .await?;

                    let historical_data: Vec<_> = historical
                        .iter()
                        .map(|q| {
                            json!({
                                "timestamp": q.timestamp.to_rfc3339(),
                                "open": q.open,
                                "high": q.high,
                                "low": q.low,
                                "close": q.close,
                                "volume": q.volume,
                                "adjusted_close": q.adjclose,
                            })
                        })
                        .collect();

                    result["historical_data"] = json!(historical_data);
                    result["data_points"] = json!(historical_data.len());
                }

                Ok::<_, crate::error::StockError>(result)
            })
            .await?;

        Ok(result)
    }
}

#[async_trait]
impl Tool for StockDataTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: StockDataParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {e}"))
        })?;

        self.fetch_stock_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "stock_data"
    }

    fn description(&self) -> &'static str {
        "Fetch current and historical stock price data for a given symbol. \
         Returns current quote and optionally historical prices over a specified range."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol (e.g., 'AAPL', 'GOOGL')"
                },
                "range": {
                    "type": "string",
                    "description": "Time range for historical data",
                    "enum": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"],
                    "default": "1d"
                },
                "include_historical": {
                    "type": "boolean",
                    "description": "Whether to include historical price data",
                    "default": false
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

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(60));
        let tool = StockDataTool::new(config, cache);

        assert_eq!(tool.name(), "stock_data");
        assert!(!tool.description().is_empty());

        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["symbol"].is_object());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_execute_current_quote() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(60));
        let tool = StockDataTool::new(config, cache);

        let params = json!({
            "symbol": "AAPL",
            "include_historical": false
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data["symbol"], "AAPL");
        assert!(data["current_quote"].is_object());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_execute_with_historical() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(60));
        let tool = StockDataTool::new(config, cache);

        let params = json!({
            "symbol": "AAPL",
            "range": "1mo",
            "include_historical": true
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data["symbol"], "AAPL");
        assert!(data["historical_data"].is_array());
        assert!(data["data_points"].as_u64().unwrap() > 0);
    }
}
