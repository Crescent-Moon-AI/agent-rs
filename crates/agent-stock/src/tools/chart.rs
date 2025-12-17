//! Tool for preparing chart data

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

/// Tool for preparing chart data
pub struct ChartDataTool {
    yahoo_client: YahooFinanceClient,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

#[derive(Debug, Deserialize)]
struct ChartParams {
    symbol: String,
    #[serde(default = "default_range")]
    range: String,
    #[serde(default)]
    indicators: Option<Vec<String>>,
}

fn default_range() -> String {
    "3mo".to_string()
}

impl ChartDataTool {
    /// Create a new chart data tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        Self {
            yahoo_client: YahooFinanceClient::new(),
            cache,
            _config: config,
        }
    }

    /// Prepare chart data
    async fn prepare_chart_data(&self, params: ChartParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();

        // Create cache key
        let cache_key = CacheKey::new(
            &symbol,
            "chart",
            json!({"range": &params.range, "indicators": &params.indicators}),
        );

        // Try to get from cache
        let result = self.cache.get_or_fetch(cache_key, || async {
            // Fetch historical data
            let quotes = self.yahoo_client.get_historical_range(&symbol, &params.range).await?;

            // Prepare candlestick data
            let candlestick_data: Vec<_> = quotes.iter().map(|q| json!({
                "timestamp": q.timestamp.to_rfc3339(),
                "open": q.open,
                "high": q.high,
                "low": q.low,
                "close": q.close,
                "volume": q.volume,
            })).collect();

            // Prepare line chart data (closing prices)
            let line_data: Vec<_> = quotes.iter().map(|q| json!({
                "timestamp": q.timestamp.to_rfc3339(),
                "value": q.close,
            })).collect();

            // Calculate simple moving averages if requested
            let mut indicator_data = json!({});

            if let Some(ref indicators) = params.indicators {
                let closes: Vec<f64> = quotes.iter().map(|q| q.close).collect();

                for indicator in indicators {
                    if indicator.starts_with("SMA_") {
                        if let Some(period_str) = indicator.strip_prefix("SMA_") {
                            if let Ok(period) = period_str.parse::<usize>() {
                                if period > 0 && period <= closes.len() {
                                    // Use ta crate for SMA calculation
                                    use ta::{Next, indicators::SimpleMovingAverage};

                                    let mut sma = SimpleMovingAverage::new(period).unwrap_or_else(|_| {
                                        SimpleMovingAverage::new(14).unwrap()
                                    });
                                    let mut sma_values = Vec::new();
                                    for &close in &closes {
                                        sma_values.push(sma.next(close));
                                    }

                                    let sma_data: Vec<_> = quotes.iter()
                                        .zip(sma_values.iter())
                                        .map(|(q, &val)| json!({
                                            "timestamp": q.timestamp.to_rfc3339(),
                                            "value": val,
                                        }))
                                        .collect();

                                    indicator_data[indicator] = json!(sma_data);
                                }
                            }
                        }
                    }
                }
            }

            Ok::<_, crate::error::StockError>(json!({
                "symbol": symbol,
                "range": params.range,
                "data_points": quotes.len(),
                "candlestick": candlestick_data,
                "line": line_data,
                "indicators": indicator_data,
                "chart_metadata": {
                    "start_date": quotes.first().map(|q| q.timestamp.to_rfc3339()),
                    "end_date": quotes.last().map(|q| q.timestamp.to_rfc3339()),
                    "min_price": quotes.iter().map(|q| q.low).fold(f64::INFINITY, f64::min),
                    "max_price": quotes.iter().map(|q| q.high).fold(f64::NEG_INFINITY, f64::max),
                }
            }))
        }).await?;

        Ok(result)
    }
}

#[async_trait]
impl Tool for ChartDataTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: ChartParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e))
        })?;

        self.prepare_chart_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "chart_data"
    }

    fn description(&self) -> &str {
        "Prepare chart data for visualization. \
         Returns candlestick data, line chart data, and optional technical indicator overlays. \
         Ready for use with charting libraries."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol"
                },
                "range": {
                    "type": "string",
                    "description": "Time range for chart data",
                    "enum": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y"],
                    "default": "3mo"
                },
                "indicators": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Technical indicators to include (e.g., ['SMA_20', 'SMA_50'])"
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
        let tool = ChartDataTool::new(config, cache);

        assert_eq!(tool.name(), "chart_data");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_prepare_chart_data() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(60));
        let tool = ChartDataTool::new(config, cache);

        let params = json!({
            "symbol": "AAPL",
            "range": "1mo",
            "indicators": ["SMA_20", "SMA_50"]
        });

        let result = tool.execute(params).await;
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data["symbol"], "AAPL");
        assert!(data["candlestick"].is_array());
        assert!(data["line"].is_array());
    }
}
