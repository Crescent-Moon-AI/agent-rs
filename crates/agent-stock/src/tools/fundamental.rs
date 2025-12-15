//! Tool for fetching fundamental data

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::api::alpha_vantage::AlphaVantageClient;
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::{Result, StockError};

/// Tool for fetching fundamental stock data
pub struct FundamentalDataTool {
    alpha_vantage_client: Option<AlphaVantageClient>,
    cache: StockCache,
    config: Arc<StockConfig>,
}

#[derive(Debug, Deserialize)]
struct FundamentalParams {
    symbol: String,
    #[serde(default)]
    metrics: Option<Vec<String>>,
}

impl FundamentalDataTool {
    /// Create a new fundamental data tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        let alpha_vantage_client = config.alpha_vantage_api_key.as_ref()
            .map(|key| AlphaVantageClient::new(key.clone()));

        Self {
            alpha_vantage_client,
            cache,
            config,
        }
    }

    /// Fetch fundamental data
    async fn fetch_fundamental_data(&self, params: FundamentalParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();

        // Create cache key
        let cache_key = CacheKey::new(&symbol, "fundamental", json!({}));

        // Try to get from cache
        let result = self.cache.get_or_fetch(cache_key, || async {
            // Check if we have Alpha Vantage client
            if let Some(ref client) = self.alpha_vantage_client {
                // Fetch company overview from Alpha Vantage
                let overview = client.get_company_overview(&symbol).await?;

                let mut result = json!({
                    "symbol": symbol,
                    "name": overview.name,
                    "exchange": overview.exchange,
                    "sector": overview.sector,
                    "industry": overview.industry,
                });

                // Parse numeric values
                if let Some(market_cap) = overview.market_cap {
                    if let Ok(cap) = market_cap.parse::<f64>() {
                        result["market_cap"] = json!(cap);
                        result["market_cap_formatted"] = json!(format_market_cap(cap));
                    }
                }

                if let Some(pe_ratio) = overview.pe_ratio {
                    if let Ok(pe) = pe_ratio.parse::<f64>() {
                        result["pe_ratio"] = json!(pe);
                        result["pe_interpretation"] = json!(interpret_pe(pe));
                    }
                }

                if let Some(div_yield) = overview.dividend_yield {
                    if let Ok(yield_val) = div_yield.parse::<f64>() {
                        result["dividend_yield"] = json!(yield_val);
                        result["dividend_yield_percent"] = json!(format!("{:.2}%", yield_val * 100.0));
                    }
                }

                if let Some(eps) = overview.eps {
                    if let Ok(eps_val) = eps.parse::<f64>() {
                        result["eps"] = json!(eps_val);
                    }
                }

                if let Some(book_value) = overview.book_value {
                    if let Ok(bv) = book_value.parse::<f64>() {
                        result["book_value"] = json!(bv);

                        // Calculate P/B ratio if we have both
                        if let (Some(market_cap), Some(book_value)) = (
                            result.get("market_cap").and_then(|v| v.as_f64()),
                            result.get("book_value").and_then(|v| v.as_f64())
                        ) {
                            if book_value != 0.0 {
                                result["pb_ratio"] = json!(market_cap / book_value);
                            }
                        }
                    }
                }

                result["data_provider"] = json!("Alpha Vantage");

                Ok::<_, StockError>(result)
            } else {
                // No Alpha Vantage key, return limited data
                Err(StockError::ConfigError(
                    "Alpha Vantage API key required for fundamental data".to_string()
                ))
            }
        }).await?;

        Ok(result)
    }
}

/// Format market cap in human-readable form
fn format_market_cap(cap: f64) -> String {
    if cap >= 1_000_000_000_000.0 {
        format!("${:.2}T", cap / 1_000_000_000_000.0)
    } else if cap >= 1_000_000_000.0 {
        format!("${:.2}B", cap / 1_000_000_000.0)
    } else if cap >= 1_000_000.0 {
        format!("${:.2}M", cap / 1_000_000.0)
    } else {
        format!("${:.2}", cap)
    }
}

/// Interpret P/E ratio
fn interpret_pe(pe: f64) -> &'static str {
    if pe < 0.0 {
        "Negative (company is not profitable)"
    } else if pe < 15.0 {
        "Low (potentially undervalued or slow growth)"
    } else if pe < 25.0 {
        "Moderate (fairly valued)"
    } else if pe < 50.0 {
        "High (potentially overvalued or high growth)"
    } else {
        "Very High (very expensive or very high growth expectations)"
    }
}

#[async_trait]
impl Tool for FundamentalDataTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: FundamentalParams = serde_json::from_value(params)
            .map_err(|e| agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e)))?;

        self.fetch_fundamental_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "fundamental_data"
    }

    fn description(&self) -> &str {
        "Fetch fundamental data and financial metrics for a stock. \
         Includes company information, market cap, P/E ratio, dividend yield, EPS, and book value. \
         Requires Alpha Vantage API key."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol"
                },
                "metrics": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Specific metrics to fetch (optional, returns all if not specified)"
                }
            },
            "required": ["symbol"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_market_cap() {
        assert_eq!(format_market_cap(1_500_000_000_000.0), "$1.50T");
        assert_eq!(format_market_cap(50_000_000_000.0), "$50.00B");
        assert_eq!(format_market_cap(250_000_000.0), "$250.00M");
    }

    #[test]
    fn test_interpret_pe() {
        assert!(interpret_pe(-5.0).contains("Negative"));
        assert!(interpret_pe(10.0).contains("Low"));
        assert!(interpret_pe(20.0).contains("Moderate"));
        assert!(interpret_pe(35.0).contains("High"));
        assert!(interpret_pe(75.0).contains("Very High"));
    }

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(std::time::Duration::from_secs(3600));
        let tool = FundamentalDataTool::new(config, cache);

        assert_eq!(tool.name(), "fundamental_data");
        assert!(!tool.description().is_empty());
    }
}
