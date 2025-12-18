//! Tool for fetching macroeconomic indicators and Fed policy data
//!
//! Uses FRED (Federal Reserve Economic Data) API for economic indicators

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::{FredClient, EconomicSummary, fred_series};
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::{Result, StockError};

/// Parameters for macro economic data requests
#[derive(Debug, Deserialize)]
struct MacroParams {
    /// Type of data: "summary", "rates", "inflation", "employment", "gdp", or specific indicator
    #[serde(default = "default_data_type")]
    data_type: String,
    /// Specific FRED series ID (optional)
    series_id: Option<String>,
    /// Number of observations for historical data
    #[serde(default = "default_observations")]
    observations: usize,
}

fn default_data_type() -> String {
    "summary".to_string()
}

fn default_observations() -> usize {
    12
}

/// Interest rate environment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateEnvironment {
    pub fed_funds_rate: Option<f64>,
    pub treasury_3m: Option<f64>,
    pub treasury_2y: Option<f64>,
    pub treasury_10y: Option<f64>,
    pub yield_spread_10y_2y: Option<f64>,
    pub yield_curve_status: String,
    pub policy_stance: String,
    pub as_of_date: String,
}

/// Inflation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationData {
    pub cpi_current: Option<f64>,
    pub cpi_yoy: Option<f64>,
    pub core_cpi_yoy: Option<f64>,
    pub pce_yoy: Option<f64>,
    pub core_pce_yoy: Option<f64>,
    pub inflation_trend: String,
    pub fed_target: f64,
    pub vs_target: String,
    pub as_of_date: String,
}

/// Employment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentData {
    pub unemployment_rate: Option<f64>,
    pub nonfarm_payrolls: Option<f64>,
    pub nonfarm_payrolls_change: Option<f64>,
    pub labor_market_status: String,
    pub as_of_date: String,
}

/// GDP data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdpData {
    pub gdp_current: Option<f64>,
    pub gdp_growth_rate: Option<f64>,
    pub gdp_trend: String,
    pub economic_cycle_phase: String,
    pub as_of_date: String,
}

/// Tool for fetching macroeconomic data
pub struct MacroEconomicTool {
    fred_client: Option<FredClient>,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

impl MacroEconomicTool {
    /// Create a new macro economic tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        let fred_client = config.fred_api_key.as_ref().map(|key| {
            FredClient::new(key.clone(), None)
        });

        Self {
            fred_client,
            cache,
            _config: config,
        }
    }

    /// Fetch macro economic data
    async fn fetch_macro_data(&self, params: MacroParams) -> Result<Value> {
        // Create cache key
        let cache_key = CacheKey::new(
            "macro",
            &params.data_type,
            json!({
                "series": params.series_id,
                "obs": params.observations
            }),
        );

        // Try to get from cache
        self.cache
            .get_or_fetch(cache_key, || async {
                self.fetch_from_fred(&params).await
            })
            .await
    }

    /// Fetch data from FRED API
    async fn fetch_from_fred(&self, params: &MacroParams) -> Result<Value> {
        let client = self.fred_client.as_ref().ok_or_else(|| {
            StockError::ConfigError(
                "FRED API key not configured. Set FRED_API_KEY environment variable.".to_string(),
            )
        })?;

        match params.data_type.to_lowercase().as_str() {
            "summary" => self.get_economic_summary(client).await,
            "rates" | "interest_rates" => self.get_rate_environment(client).await,
            "inflation" => self.get_inflation_data(client).await,
            "employment" | "jobs" => self.get_employment_data(client).await,
            "gdp" | "growth" => self.get_gdp_data(client).await,
            "market" => self.get_market_indicators(client).await,
            "custom" | "series" => {
                if let Some(ref series_id) = params.series_id {
                    self.get_series_data(client, series_id, params.observations).await
                } else {
                    Err(StockError::ConfigError(
                        "series_id required for custom data type".to_string(),
                    ))
                }
            }
            _ => self.get_economic_summary(client).await,
        }
    }

    /// Get comprehensive economic summary
    async fn get_economic_summary(&self, client: &FredClient) -> Result<Value> {
        let summary = client.get_economic_summary().await?;

        // Build market outlook
        let outlook = self.generate_market_outlook(&summary);

        Ok(json!({
            "type": "economic_summary",
            "data": {
                "interest_rates": {
                    "fed_funds_rate": summary.fed_funds_rate,
                    "treasury_10y": summary.treasury_10y,
                    "treasury_2y": summary.treasury_2y,
                    "yield_spread": summary.yield_spread,
                    "yield_curve_inverted": summary.yield_curve_inverted,
                },
                "inflation": {
                    "cpi_yoy": summary.cpi_yoy,
                    "core_pce_yoy": summary.core_pce_yoy,
                    "fed_target": 2.0,
                },
                "employment": {
                    "unemployment_rate": summary.unemployment_rate,
                },
                "growth": {
                    "gdp_growth": summary.gdp_growth,
                },
                "sentiment": {
                    "consumer_sentiment": summary.consumer_sentiment,
                    "vix": summary.vix,
                },
            },
            "assessment": summary.assessment,
            "market_outlook": outlook,
            "as_of_date": summary.as_of_date,
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get interest rate environment
    async fn get_rate_environment(&self, client: &FredClient) -> Result<Value> {
        let rate_data = client.get_rate_environment().await?;

        let fed_funds = rate_data
            .get("rates")
            .and_then(|r| r.get("Federal Funds Rate"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_f64());

        let treasury_10y = rate_data
            .get("rates")
            .and_then(|r| r.get("10-Year Treasury"))
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_f64());

        // Determine policy stance
        let policy_stance = if fed_funds.unwrap_or(0.0) >= 5.0 {
            "Restrictive - Fed is actively fighting inflation"
        } else if fed_funds.unwrap_or(0.0) >= 2.5 {
            "Neutral - Policy is balanced"
        } else {
            "Accommodative - Supporting economic growth"
        };

        Ok(json!({
            "type": "rate_environment",
            "data": rate_data,
            "policy_stance": policy_stance,
            "implications": self.get_rate_implications(fed_funds, treasury_10y),
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get inflation data
    async fn get_inflation_data(&self, client: &FredClient) -> Result<Value> {
        let series = [
            (fred_series::CPI, "CPI"),
            (fred_series::CORE_CPI, "Core CPI"),
            (fred_series::PCE, "PCE"),
            (fred_series::CORE_PCE, "Core PCE"),
        ];

        let mut inflation_data = serde_json::Map::new();

        for (series_id, name) in series {
            if let Ok((current, _, yoy_pct)) = client.get_yoy_change(series_id).await {
                inflation_data.insert(
                    name.to_string(),
                    json!({
                        "current": current,
                        "yoy_change_pct": yoy_pct,
                    }),
                );
            }
        }

        let core_pce_yoy = inflation_data
            .get("Core PCE")
            .and_then(|v| v.get("yoy_change_pct"))
            .and_then(|v| v.as_f64());

        let vs_target = match core_pce_yoy {
            Some(pce) if pce <= 2.0 => "At or below Fed's 2% target",
            Some(pce) if pce <= 2.5 => "Slightly above target",
            Some(pce) if pce <= 3.5 => "Moderately elevated",
            Some(pce) if pce <= 5.0 => "Elevated - Fed likely to maintain restrictive policy",
            Some(_) => "Significantly elevated - High inflation concern",
            None => "Data unavailable",
        };

        let trend = match core_pce_yoy {
            Some(pce) if pce < 2.5 => "Inflation cooling towards target",
            Some(pce) if pce < 4.0 => "Inflation moderating but above target",
            Some(_) => "Inflation remains elevated",
            None => "Trend unavailable",
        };

        Ok(json!({
            "type": "inflation",
            "data": inflation_data,
            "fed_target": 2.0,
            "vs_target": vs_target,
            "trend": trend,
            "as_of_date": chrono::Utc::now().format("%Y-%m-%d").to_string(),
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get employment data
    async fn get_employment_data(&self, client: &FredClient) -> Result<Value> {
        let unemployment = client.get_latest(fred_series::UNEMPLOYMENT_RATE).await.ok();
        let payrolls = client.get_latest(fred_series::NONFARM_PAYROLLS).await.ok();

        let unemployment_rate = unemployment.as_ref().map(|o| o.value);

        let labor_market = match unemployment_rate {
            Some(rate) if rate < 4.0 => "Very tight labor market",
            Some(rate) if rate < 5.0 => "Healthy labor market",
            Some(rate) if rate < 6.0 => "Moderate labor market slack",
            Some(_) => "Elevated unemployment",
            None => "Data unavailable",
        };

        Ok(json!({
            "type": "employment",
            "data": {
                "unemployment_rate": {
                    "value": unemployment.as_ref().map(|o| o.value),
                    "date": unemployment.as_ref().map(|o| o.date.clone()),
                },
                "nonfarm_payrolls": {
                    "value": payrolls.as_ref().map(|o| o.value),
                    "date": payrolls.as_ref().map(|o| o.date.clone()),
                    "unit": "thousands"
                },
            },
            "labor_market_status": labor_market,
            "implications": self.get_employment_implications(unemployment_rate),
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get GDP data
    async fn get_gdp_data(&self, client: &FredClient) -> Result<Value> {
        let gdp = client.get_latest(fred_series::GDP).await.ok();
        let gdp_growth = client.get_latest(fred_series::GDP_GROWTH).await.ok();

        let growth_rate = gdp_growth.as_ref().map(|o| o.value);

        let cycle_phase = match growth_rate {
            Some(rate) if rate > 3.0 => "Expansion - Strong growth",
            Some(rate) if rate > 1.5 => "Moderate growth",
            Some(rate) if rate > 0.0 => "Slow growth",
            Some(rate) if rate > -0.5 => "Stagnation / Near-recession",
            Some(_) => "Contraction / Recession",
            None => "Data unavailable",
        };

        Ok(json!({
            "type": "gdp",
            "data": {
                "real_gdp": {
                    "value": gdp.as_ref().map(|o| o.value),
                    "date": gdp.as_ref().map(|o| o.date.clone()),
                    "unit": "billions of chained 2017 dollars"
                },
                "gdp_growth_rate": {
                    "value": gdp_growth.as_ref().map(|o| o.value),
                    "date": gdp_growth.as_ref().map(|o| o.date.clone()),
                    "unit": "percent, seasonally adjusted annual rate"
                },
            },
            "economic_cycle_phase": cycle_phase,
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get market indicators
    async fn get_market_indicators(&self, client: &FredClient) -> Result<Value> {
        let vix = client.get_latest(fred_series::VIX).await.ok();
        let sp500 = client.get_latest(fred_series::SP500).await.ok();
        let dollar = client.get_latest(fred_series::DOLLAR_INDEX).await.ok();
        let oil = client.get_latest(fred_series::OIL_WTI).await.ok();
        let gold = client.get_latest(fred_series::GOLD).await.ok();

        let vix_value = vix.as_ref().map(|o| o.value);
        let market_sentiment = match vix_value {
            Some(v) if v < 15.0 => "Low volatility - Complacency",
            Some(v) if v < 20.0 => "Normal volatility - Calm markets",
            Some(v) if v < 25.0 => "Elevated volatility - Caution",
            Some(v) if v < 30.0 => "High volatility - Fear",
            Some(_) => "Extreme volatility - Panic",
            None => "Unknown",
        };

        Ok(json!({
            "type": "market_indicators",
            "data": {
                "vix": {
                    "value": vix.as_ref().map(|o| o.value),
                    "date": vix.as_ref().map(|o| o.date.clone()),
                    "interpretation": market_sentiment,
                },
                "sp500": {
                    "value": sp500.as_ref().map(|o| o.value),
                    "date": sp500.as_ref().map(|o| o.date.clone()),
                },
                "dollar_index": {
                    "value": dollar.as_ref().map(|o| o.value),
                    "date": dollar.as_ref().map(|o| o.date.clone()),
                },
                "oil_wti": {
                    "value": oil.as_ref().map(|o| o.value),
                    "date": oil.as_ref().map(|o| o.date.clone()),
                    "unit": "USD/barrel"
                },
                "gold": {
                    "value": gold.as_ref().map(|o| o.value),
                    "date": gold.as_ref().map(|o| o.date.clone()),
                    "unit": "USD/troy oz"
                },
            },
            "market_sentiment": market_sentiment,
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Get specific series data
    async fn get_series_data(
        &self,
        client: &FredClient,
        series_id: &str,
        limit: usize,
    ) -> Result<Value> {
        let info = client.get_series_info(series_id).await?;
        let observations = client
            .get_observations(series_id, None, None, Some(limit as u32))
            .await?;

        let parsed: Vec<_> = observations
            .iter()
            .filter_map(|o| {
                o.value.parse::<f64>().ok().map(|v| {
                    json!({
                        "date": o.date,
                        "value": v
                    })
                })
            })
            .collect();

        Ok(json!({
            "type": "custom_series",
            "series_id": series_id,
            "title": info.title,
            "units": info.units,
            "frequency": info.frequency,
            "observations": parsed,
            "last_updated": info.last_updated,
            "data_source": "Federal Reserve Economic Data (FRED)",
        }))
    }

    /// Generate market outlook based on economic summary
    fn generate_market_outlook(&self, summary: &EconomicSummary) -> Value {
        let mut factors = Vec::new();
        let mut risks = Vec::new();
        let mut opportunities = Vec::new();

        // Analyze yield curve
        if summary.yield_curve_inverted {
            risks.push("Inverted yield curve historically precedes recessions");
            factors.push("Recession risk elevated");
        }

        // Analyze inflation
        if let Some(pce) = summary.core_pce_yoy {
            if pce > 3.0 {
                risks.push("Elevated inflation may lead to prolonged high rates");
                factors.push("Inflation remains above Fed target");
            } else if pce < 2.5 {
                opportunities.push("Cooling inflation may allow Fed to ease policy");
                factors.push("Inflation trending towards target");
            }
        }

        // Analyze employment
        if let Some(unemp) = summary.unemployment_rate {
            if unemp < 4.0 {
                factors.push("Tight labor market supports consumer spending");
                risks.push("Wage pressures may keep inflation elevated");
            } else if unemp > 5.0 {
                risks.push("Rising unemployment may signal economic weakness");
            }
        }

        // Analyze VIX
        if let Some(vix) = summary.vix {
            if vix > 25.0 {
                risks.push("Elevated market volatility");
            } else if vix < 15.0 {
                risks.push("Low VIX may indicate complacency");
            }
        }

        // Overall assessment
        let outlook = if risks.len() > opportunities.len() + 1 {
            "Cautious - Multiple risk factors present"
        } else if opportunities.len() > risks.len() {
            "Constructive - Favorable conditions emerging"
        } else {
            "Neutral - Mixed signals, selectivity recommended"
        };

        json!({
            "outlook": outlook,
            "key_factors": factors,
            "risks": risks,
            "opportunities": opportunities,
        })
    }

    /// Get rate implications for markets
    fn get_rate_implications(&self, fed_funds: Option<f64>, treasury_10y: Option<f64>) -> Value {
        let mut implications = Vec::new();

        if let Some(rate) = fed_funds {
            if rate >= 5.0 {
                implications.push("High rates pressure growth stocks and valuations");
                implications.push("Financial sector may benefit from higher net interest margins");
                implications.push("Real estate and utilities face headwinds from higher borrowing costs");
            } else if rate >= 3.0 {
                implications.push("Moderate rates - balanced environment for equities");
            } else {
                implications.push("Low rates support risk assets and growth stocks");
            }
        }

        if let Some(yield_10y) = treasury_10y {
            if yield_10y >= 4.5 {
                implications.push("Attractive bond yields compete with equities for capital");
            } else if yield_10y >= 3.0 {
                implications.push("Bonds offer reasonable income alternative");
            } else {
                implications.push("Low yields favor equities for income seekers");
            }
        }

        json!(implications)
    }

    /// Get employment implications
    fn get_employment_implications(&self, unemployment: Option<f64>) -> Value {
        let mut implications = Vec::new();

        if let Some(rate) = unemployment {
            if rate < 4.0 {
                implications.push("Tight labor market supports consumer spending");
                implications.push("Wage growth may keep inflation elevated");
                implications.push("Consumer discretionary sector favored");
            } else if rate < 5.5 {
                implications.push("Healthy employment supports economic stability");
            } else {
                implications.push("Rising unemployment may pressure consumer spending");
                implications.push("Defensive sectors may outperform");
            }
        }

        json!(implications)
    }
}

#[async_trait]
impl Tool for MacroEconomicTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: MacroParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e))
        })?;

        self.fetch_macro_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "macro_economic"
    }

    fn description(&self) -> &str {
        "Fetch macroeconomic indicators and Fed policy data from FRED (Federal Reserve Economic Data). \
         Returns data on interest rates, inflation (CPI, PCE), employment, GDP growth, and market indicators. \
         Provides economic assessment and market implications. Requires FRED API key."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "data_type": {
                    "type": "string",
                    "enum": ["summary", "rates", "inflation", "employment", "gdp", "market", "custom"],
                    "description": "Type of economic data to fetch",
                    "default": "summary"
                },
                "series_id": {
                    "type": "string",
                    "description": "Specific FRED series ID (required for 'custom' data_type)"
                },
                "observations": {
                    "type": "integer",
                    "description": "Number of historical observations (for custom series)",
                    "default": 12,
                    "minimum": 1,
                    "maximum": 100
                }
            }
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
        let cache = StockCache::new(Duration::from_secs(3600));
        let tool = MacroEconomicTool::new(config, cache);

        assert_eq!(tool.name(), "macro_economic");
        assert!(tool.description().contains("FRED"));
        assert!(tool.input_schema()["properties"]["data_type"].is_object());
    }
}
