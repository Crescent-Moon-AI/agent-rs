//! Federal Reserve Economic Data (FRED) API client
//!
//! FRED is a database maintained by the Federal Reserve Bank of St. Louis
//! containing over 800,000 economic time series from numerous sources.
//!
//! API Key: Free registration at https://fred.stlouisfed.org/docs/api/api_key.html
//! Rate Limit: 120 requests per minute

use crate::error::{Result, StockError};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

const FRED_BASE_URL: &str = "https://api.stlouisfed.org/fred";

/// Common FRED series IDs for economic indicators
pub mod series {
    /// Federal Funds Effective Rate
    pub const FED_FUNDS_RATE: &str = "FEDFUNDS";
    /// 10-Year Treasury Constant Maturity Rate
    pub const TREASURY_10Y: &str = "DGS10";
    /// 2-Year Treasury Constant Maturity Rate
    pub const TREASURY_2Y: &str = "DGS2";
    /// 3-Month Treasury Bill Rate
    pub const TREASURY_3M: &str = "DTB3";
    /// 10Y-2Y Treasury Spread (Yield Curve)
    pub const YIELD_SPREAD_10Y_2Y: &str = "T10Y2Y";
    /// Unemployment Rate
    pub const UNEMPLOYMENT_RATE: &str = "UNRATE";
    /// Total Nonfarm Payrolls
    pub const NONFARM_PAYROLLS: &str = "PAYEMS";
    /// Consumer Price Index (All Urban)
    pub const CPI: &str = "CPIAUCSL";
    /// Core CPI (Less Food and Energy)
    pub const CORE_CPI: &str = "CPILFESL";
    /// Personal Consumption Expenditures
    pub const PCE: &str = "PCEPI";
    /// Core PCE (Fed's preferred inflation measure)
    pub const CORE_PCE: &str = "PCEPILFE";
    /// Real GDP
    pub const GDP: &str = "GDPC1";
    /// GDP Growth Rate (Quarterly)
    pub const GDP_GROWTH: &str = "A191RL1Q225SBEA";
    /// M2 Money Supply
    pub const M2: &str = "M2SL";
    /// Retail Sales
    pub const RETAIL_SALES: &str = "RSAFS";
    /// Consumer Sentiment (U of Michigan)
    pub const CONSUMER_SENTIMENT: &str = "UMCSENT";
    /// Industrial Production Index
    pub const INDUSTRIAL_PRODUCTION: &str = "INDPRO";
    /// Housing Starts
    pub const HOUSING_STARTS: &str = "HOUST";
    /// S&P 500 Index
    pub const SP500: &str = "SP500";
    /// VIX Volatility Index
    pub const VIX: &str = "VIXCLS";
    /// US Dollar Index
    pub const DOLLAR_INDEX: &str = "DTWEXBGS";
    /// Crude Oil Price (WTI)
    pub const OIL_WTI: &str = "DCOILWTICO";
    /// Gold Price
    pub const GOLD: &str = "GOLDAMGBD228NLBM";
}

/// Economic indicator category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorCategory {
    InterestRates,
    Inflation,
    Employment,
    GDP,
    MonetaryPolicy,
    ConsumerActivity,
    Housing,
    MarketIndicators,
    Commodities,
}

/// Observation data from FRED series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Date of observation (YYYY-MM-DD)
    pub date: String,
    /// Value (can be "." for missing data)
    pub value: String,
}

/// Parsed observation with numeric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedObservation {
    pub date: String,
    pub value: f64,
}

/// FRED series information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesInfo {
    pub id: String,
    pub title: String,
    pub observation_start: String,
    pub observation_end: String,
    pub frequency: String,
    pub frequency_short: String,
    pub units: String,
    pub units_short: String,
    pub seasonal_adjustment: String,
    pub seasonal_adjustment_short: String,
    pub last_updated: String,
    pub notes: Option<String>,
}

/// FRED series response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SeriesResponse {
    seriess: Vec<SeriesInfo>,
}

/// FRED observations response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ObservationsResponse {
    observations: Vec<Observation>,
}

/// Economic data summary for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicSummary {
    /// Current Federal Funds Rate
    pub fed_funds_rate: Option<f64>,
    /// 10-Year Treasury Yield
    pub treasury_10y: Option<f64>,
    /// 2-Year Treasury Yield
    pub treasury_2y: Option<f64>,
    /// Yield Curve Spread (10Y - 2Y)
    pub yield_spread: Option<f64>,
    /// Is yield curve inverted?
    pub yield_curve_inverted: bool,
    /// Latest CPI (YoY %)
    pub cpi_yoy: Option<f64>,
    /// Latest Core PCE (YoY %)
    pub core_pce_yoy: Option<f64>,
    /// Unemployment Rate
    pub unemployment_rate: Option<f64>,
    /// Latest GDP Growth Rate (%)
    pub gdp_growth: Option<f64>,
    /// Consumer Sentiment
    pub consumer_sentiment: Option<f64>,
    /// VIX (Volatility Index)
    pub vix: Option<f64>,
    /// Data timestamp
    pub as_of_date: String,
    /// Overall economic assessment
    pub assessment: String,
}

/// FRED API client
pub struct FredClient {
    client: Client,
    api_key: String,
    rate_limiter: SharedRateLimiter,
}

impl FredClient {
    /// Create a new FRED client
    ///
    /// # Arguments
    /// * `api_key` - FRED API key
    /// * `rate_limit` - Requests per minute (default 120)
    pub fn new(api_key: impl Into<String>, rate_limit: Option<u32>) -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(rate_limit.unwrap_or(120)).unwrap_or(NonZeroU32::new(120).unwrap()),
        );
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client: Client::new(),
            api_key: api_key.into(),
            rate_limiter,
        }
    }

    /// Create from environment variable FRED_API_KEY
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("FRED_API_KEY").map_err(|_| {
            StockError::ConfigError("FRED_API_KEY environment variable not set".to_string())
        })?;

        Ok(Self::new(api_key, None))
    }

    /// Get series information
    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo> {
        self.rate_limiter.until_ready().await;

        let mut params = HashMap::new();
        params.insert("series_id", series_id);
        params.insert("api_key", &self.api_key);
        params.insert("file_type", "json");

        let url = format!("{}/series", FRED_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("FRED request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StockError::ApiError(format!(
                "FRED API error: {}",
                response.status()
            )));
        }

        let data: SeriesResponse = response
            .json()
            .await
            .map_err(|e| StockError::ApiError(format!("Failed to parse FRED response: {}", e)))?;

        data.seriess
            .into_iter()
            .next()
            .ok_or_else(|| StockError::ApiError("Series not found".to_string()))
    }

    /// Get observations for a series
    pub async fn get_observations(
        &self,
        series_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<Observation>> {
        self.rate_limiter.until_ready().await;

        let api_key = self.api_key.clone();
        let mut params: HashMap<&str, String> = HashMap::new();
        params.insert("series_id", series_id.to_string());
        params.insert("api_key", api_key);
        params.insert("file_type", "json".to_string());
        params.insert("sort_order", "desc".to_string());

        if let Some(start) = start_date {
            params.insert("observation_start", start.to_string());
        }
        if let Some(end) = end_date {
            params.insert("observation_end", end.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit", lim.to_string());
        }

        let url = format!("{}/series/observations", FRED_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("FRED request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StockError::ApiError(format!(
                "FRED API error: {}",
                response.status()
            )));
        }

        let data: ObservationsResponse = response
            .json()
            .await
            .map_err(|e| StockError::ApiError(format!("Failed to parse FRED response: {}", e)))?;

        Ok(data.observations)
    }

    /// Get latest value for a series
    pub async fn get_latest(&self, series_id: &str) -> Result<ParsedObservation> {
        let observations = self.get_observations(series_id, None, None, Some(1)).await?;

        let obs = observations
            .into_iter()
            .next()
            .ok_or_else(|| StockError::ApiError("No observations found".to_string()))?;

        let value = obs
            .value
            .parse::<f64>()
            .map_err(|_| StockError::ApiError("Invalid numeric value".to_string()))?;

        Ok(ParsedObservation {
            date: obs.date,
            value,
        })
    }

    /// Get multiple latest values for efficiency
    pub async fn get_latest_batch(&self, series_ids: &[&str]) -> Result<HashMap<String, ParsedObservation>> {
        let mut results = HashMap::new();

        for series_id in series_ids {
            match self.get_latest(series_id).await {
                Ok(obs) => {
                    results.insert(series_id.to_string(), obs);
                }
                Err(e) => {
                    tracing::warn!("Failed to get {} from FRED: {}", series_id, e);
                }
            }
        }

        Ok(results)
    }

    /// Calculate year-over-year change for a series
    pub async fn get_yoy_change(&self, series_id: &str) -> Result<(f64, f64, f64)> {
        // Get last 13 months of data to ensure we have YoY comparison
        let observations = self
            .get_observations(series_id, None, None, Some(13))
            .await?;

        if observations.len() < 2 {
            return Err(StockError::ApiError(
                "Insufficient data for YoY calculation".to_string(),
            ));
        }

        let current = observations[0]
            .value
            .parse::<f64>()
            .map_err(|_| StockError::ApiError("Invalid numeric value".to_string()))?;

        // Find observation from ~12 months ago
        let year_ago = observations
            .iter()
            .find(|o| {
                if let Ok(v) = o.value.parse::<f64>() {
                    v > 0.0 // Just check it's valid
                } else {
                    false
                }
            })
            .and_then(|o| o.value.parse::<f64>().ok())
            .unwrap_or(current);

        let yoy_change = current - year_ago;
        let yoy_percent = if year_ago != 0.0 {
            (yoy_change / year_ago) * 100.0
        } else {
            0.0
        };

        Ok((current, yoy_change, yoy_percent))
    }

    /// Get comprehensive economic summary
    pub async fn get_economic_summary(&self) -> Result<EconomicSummary> {
        let series_ids = [
            series::FED_FUNDS_RATE,
            series::TREASURY_10Y,
            series::TREASURY_2Y,
            series::YIELD_SPREAD_10Y_2Y,
            series::UNEMPLOYMENT_RATE,
            series::CONSUMER_SENTIMENT,
            series::VIX,
        ];

        let batch = self.get_latest_batch(&series_ids).await?;

        let fed_funds = batch.get(series::FED_FUNDS_RATE).map(|o| o.value);
        let treasury_10y = batch.get(series::TREASURY_10Y).map(|o| o.value);
        let treasury_2y = batch.get(series::TREASURY_2Y).map(|o| o.value);
        let yield_spread = batch.get(series::YIELD_SPREAD_10Y_2Y).map(|o| o.value);
        let unemployment = batch.get(series::UNEMPLOYMENT_RATE).map(|o| o.value);
        let sentiment = batch.get(series::CONSUMER_SENTIMENT).map(|o| o.value);
        let vix = batch.get(series::VIX).map(|o| o.value);

        // Get inflation data (needs YoY calculation)
        let cpi_yoy = self
            .get_yoy_change(series::CPI)
            .await
            .ok()
            .map(|(_, _, pct)| pct);
        let core_pce_yoy = self
            .get_yoy_change(series::CORE_PCE)
            .await
            .ok()
            .map(|(_, _, pct)| pct);

        // Get GDP growth
        let gdp_growth = self.get_latest(series::GDP_GROWTH).await.ok().map(|o| o.value);

        let yield_curve_inverted = yield_spread.map(|s| s < 0.0).unwrap_or(false);

        // Generate assessment
        let assessment = self.generate_assessment(
            fed_funds,
            yield_curve_inverted,
            cpi_yoy,
            unemployment,
            vix,
        );

        Ok(EconomicSummary {
            fed_funds_rate: fed_funds,
            treasury_10y,
            treasury_2y,
            yield_spread,
            yield_curve_inverted,
            cpi_yoy,
            core_pce_yoy,
            unemployment_rate: unemployment,
            gdp_growth,
            consumer_sentiment: sentiment,
            vix,
            as_of_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            assessment,
        })
    }

    /// Generate economic assessment based on indicators
    fn generate_assessment(
        &self,
        fed_funds: Option<f64>,
        yield_curve_inverted: bool,
        inflation: Option<f64>,
        unemployment: Option<f64>,
        vix: Option<f64>,
    ) -> String {
        let mut factors = Vec::new();

        // Fed policy stance
        if let Some(rate) = fed_funds {
            if rate >= 5.0 {
                factors.push("restrictive monetary policy");
            } else if rate >= 2.5 {
                factors.push("neutral monetary policy");
            } else {
                factors.push("accommodative monetary policy");
            }
        }

        // Yield curve
        if yield_curve_inverted {
            factors.push("inverted yield curve (recession signal)");
        }

        // Inflation
        if let Some(inf) = inflation {
            if inf > 4.0 {
                factors.push("elevated inflation");
            } else if inf > 2.5 {
                factors.push("above-target inflation");
            } else if inf >= 1.5 {
                factors.push("stable inflation near target");
            } else {
                factors.push("low inflation");
            }
        }

        // Employment
        if let Some(unemp) = unemployment {
            if unemp < 4.0 {
                factors.push("tight labor market");
            } else if unemp < 5.5 {
                factors.push("healthy labor market");
            } else {
                factors.push("elevated unemployment");
            }
        }

        // Market volatility
        if let Some(v) = vix {
            if v > 30.0 {
                factors.push("high market volatility");
            } else if v > 20.0 {
                factors.push("elevated market uncertainty");
            } else {
                factors.push("low market volatility");
            }
        }

        if factors.is_empty() {
            "Insufficient data for assessment".to_string()
        } else {
            format!("Current economic conditions: {}", factors.join("; "))
        }
    }

    /// Get interest rate environment analysis
    pub async fn get_rate_environment(&self) -> Result<serde_json::Value> {
        let series = [
            (series::FED_FUNDS_RATE, "Federal Funds Rate"),
            (series::TREASURY_3M, "3-Month T-Bill"),
            (series::TREASURY_2Y, "2-Year Treasury"),
            (series::TREASURY_10Y, "10-Year Treasury"),
            (series::YIELD_SPREAD_10Y_2Y, "10Y-2Y Spread"),
        ];

        let mut rates = serde_json::Map::new();
        
        for (id, name) in series {
            if let Ok(obs) = self.get_latest(id).await {
                rates.insert(
                    name.to_string(),
                    serde_json::json!({
                        "value": obs.value,
                        "date": obs.date,
                        "unit": "%"
                    }),
                );
            }
        }

        let spread = rates
            .get("10Y-2Y Spread")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_f64());

        let curve_status = if spread.map(|s| s < 0.0).unwrap_or(false) {
            "Inverted (Recession Warning)"
        } else if spread.map(|s| s < 0.5).unwrap_or(false) {
            "Flat (Caution)"
        } else {
            "Normal (Healthy)"
        };

        Ok(serde_json::json!({
            "rates": rates,
            "yield_curve_status": curve_status,
            "analysis_date": chrono::Utc::now().format("%Y-%m-%d").to_string()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_constants() {
        assert_eq!(series::FED_FUNDS_RATE, "FEDFUNDS");
        assert_eq!(series::CPI, "CPIAUCSL");
        assert_eq!(series::UNEMPLOYMENT_RATE, "UNRATE");
    }

    #[test]
    fn test_client_creation() {
        let client = FredClient::new("test_key", None);
        assert_eq!(client.api_key, "test_key");
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_get_latest() {
        let client = FredClient::from_env().unwrap();
        let result = client.get_latest(series::FED_FUNDS_RATE).await;
        assert!(result.is_ok());
        
        let obs = result.unwrap();
        assert!(!obs.date.is_empty());
        assert!(obs.value >= 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_get_economic_summary() {
        let client = FredClient::from_env().unwrap();
        let summary = client.get_economic_summary().await;
        assert!(summary.is_ok());
        
        let summary = summary.unwrap();
        assert!(!summary.as_of_date.is_empty());
        assert!(!summary.assessment.is_empty());
    }
}
