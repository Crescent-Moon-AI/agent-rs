//! Tool for sector and industry analysis
//!
//! Provides analysis of market sectors, industry trends, and sector rotation

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::YahooFinanceClient;
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::Result;

/// Market sector definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sector {
    Technology,
    Healthcare,
    Financials,
    ConsumerDiscretionary,
    ConsumerStaples,
    Energy,
    Materials,
    Industrials,
    Utilities,
    RealEstate,
    CommunicationServices,
}

impl Sector {
    /// Get sector ETF ticker (SPDR Select Sector ETFs)
    pub fn etf_ticker(&self) -> &'static str {
        match self {
            Sector::Technology => "XLK",
            Sector::Healthcare => "XLV",
            Sector::Financials => "XLF",
            Sector::ConsumerDiscretionary => "XLY",
            Sector::ConsumerStaples => "XLP",
            Sector::Energy => "XLE",
            Sector::Materials => "XLB",
            Sector::Industrials => "XLI",
            Sector::Utilities => "XLU",
            Sector::RealEstate => "XLRE",
            Sector::CommunicationServices => "XLC",
        }
    }

    /// Get sector name
    pub fn name(&self) -> &'static str {
        match self {
            Sector::Technology => "Technology",
            Sector::Healthcare => "Healthcare",
            Sector::Financials => "Financials",
            Sector::ConsumerDiscretionary => "Consumer Discretionary",
            Sector::ConsumerStaples => "Consumer Staples",
            Sector::Energy => "Energy",
            Sector::Materials => "Materials",
            Sector::Industrials => "Industrials",
            Sector::Utilities => "Utilities",
            Sector::RealEstate => "Real Estate",
            Sector::CommunicationServices => "Communication Services",
        }
    }

    /// Get all sectors
    pub fn all() -> Vec<Sector> {
        vec![
            Sector::Technology,
            Sector::Healthcare,
            Sector::Financials,
            Sector::ConsumerDiscretionary,
            Sector::ConsumerStaples,
            Sector::Energy,
            Sector::Materials,
            Sector::Industrials,
            Sector::Utilities,
            Sector::RealEstate,
            Sector::CommunicationServices,
        ]
    }

    /// Parse sector from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "technology" | "tech" | "xlk" => Some(Sector::Technology),
            "healthcare" | "health" | "xlv" => Some(Sector::Healthcare),
            "financials" | "financial" | "xlf" => Some(Sector::Financials),
            "consumer discretionary" | "discretionary" | "xly" => {
                Some(Sector::ConsumerDiscretionary)
            }
            "consumer staples" | "staples" | "xlp" => Some(Sector::ConsumerStaples),
            "energy" | "xle" => Some(Sector::Energy),
            "materials" | "xlb" => Some(Sector::Materials),
            "industrials" | "industrial" | "xli" => Some(Sector::Industrials),
            "utilities" | "utility" | "xlu" => Some(Sector::Utilities),
            "real estate" | "realestate" | "xlre" => Some(Sector::RealEstate),
            "communication services" | "communication" | "telecom" | "xlc" => {
                Some(Sector::CommunicationServices)
            }
            _ => None,
        }
    }

    /// Get economic sensitivity classification
    pub fn sensitivity(&self) -> &'static str {
        match self {
            Sector::Technology
            | Sector::ConsumerDiscretionary
            | Sector::Financials
            | Sector::Industrials
            | Sector::Materials => "Cyclical",
            Sector::Healthcare
            | Sector::ConsumerStaples
            | Sector::Utilities
            | Sector::RealEstate => "Defensive",
            Sector::Energy | Sector::CommunicationServices => "Mixed",
        }
    }

    /// Get rate sensitivity
    pub fn rate_sensitivity(&self) -> &'static str {
        match self {
            Sector::RealEstate | Sector::Utilities | Sector::Financials => "High",
            Sector::Technology | Sector::ConsumerDiscretionary => "Moderate",
            Sector::Healthcare
            | Sector::ConsumerStaples
            | Sector::Energy
            | Sector::Materials
            | Sector::Industrials
            | Sector::CommunicationServices => "Low",
        }
    }
}

/// Sector performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorPerformance {
    pub sector: String,
    pub etf_ticker: String,
    pub current_price: Option<f64>,
    pub change_1d: Option<f64>,
    pub change_1d_pct: Option<f64>,
    pub change_5d_pct: Option<f64>,
    pub change_1m_pct: Option<f64>,
    pub change_3m_pct: Option<f64>,
    pub change_ytd_pct: Option<f64>,
    pub volume: Option<u64>,
    pub avg_volume: Option<u64>,
    pub volume_ratio: Option<f64>,
}

/// Parameters for sector analysis
#[derive(Debug, Deserialize)]
struct SectorParams {
    /// Specific sector to analyze (optional, returns all if not specified)
    sector: Option<String>,
    /// Type of analysis: "performance", "rotation", "comparison"
    #[serde(default = "default_analysis_type")]
    analysis_type: String,
    /// Include top holdings for each sector
    #[serde(default)]
    include_holdings: bool,
}

fn default_analysis_type() -> String {
    "performance".to_string()
}

/// Tool for sector analysis
pub struct SectorAnalysisTool {
    yahoo_client: YahooFinanceClient,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

impl SectorAnalysisTool {
    /// Create a new sector analysis tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        let yahoo_client = YahooFinanceClient::new();

        Self {
            yahoo_client,
            cache,
            _config: config,
        }
    }

    /// Fetch sector analysis data
    async fn fetch_sector_data(&self, params: SectorParams) -> Result<Value> {
        // Create cache key
        let cache_key = CacheKey::new(
            "sector",
            &params.analysis_type,
            json!({
                "sector": params.sector,
                "holdings": params.include_holdings
            }),
        );

        // Try to get from cache
        self.cache
            .get_or_fetch(cache_key, || async {
                self.analyze_sectors(&params).await
            })
            .await
    }

    /// Analyze sectors based on parameters
    async fn analyze_sectors(&self, params: &SectorParams) -> Result<Value> {
        match params.analysis_type.to_lowercase().as_str() {
            "performance" => {
                if let Some(ref sector_name) = params.sector {
                    self.get_sector_performance(sector_name).await
                } else {
                    self.get_all_sectors_performance().await
                }
            }
            "rotation" => self.analyze_sector_rotation().await,
            "comparison" => self.compare_sectors().await,
            _ => self.get_all_sectors_performance().await,
        }
    }

    /// Get performance for a specific sector
    async fn get_sector_performance(&self, sector_name: &str) -> Result<Value> {
        let sector = Sector::from_str(sector_name).ok_or_else(|| {
            crate::error::StockError::InvalidSymbol(format!(
                "Unknown sector: {}. Valid sectors: Technology, Healthcare, Financials, etc.",
                sector_name
            ))
        })?;

        let performance = self.fetch_sector_etf_data(sector).await?;

        // Get sector characteristics
        let characteristics = json!({
            "sensitivity": sector.sensitivity(),
            "rate_sensitivity": sector.rate_sensitivity(),
            "description": self.get_sector_description(sector),
        });

        // Analyze current conditions
        let analysis = self.analyze_sector_conditions(&performance);

        Ok(json!({
            "type": "sector_performance",
            "sector": sector.name(),
            "performance": performance,
            "characteristics": characteristics,
            "analysis": analysis,
            "data_source": "Yahoo Finance",
        }))
    }

    /// Get performance for all sectors
    async fn get_all_sectors_performance(&self) -> Result<Value> {
        let mut performances = Vec::new();

        for sector in Sector::all() {
            if let Ok(perf) = self.fetch_sector_etf_data(sector).await {
                performances.push(perf);
            }
        }

        // Sort by 1-day performance
        performances.sort_by(|a, b| {
            let a_pct = a.get("change_1d_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b_pct = b.get("change_1d_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
            b_pct.partial_cmp(&a_pct).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Identify leaders and laggards
        let leaders: Vec<_> = performances.iter().take(3).collect();
        let laggards: Vec<_> = performances.iter().rev().take(3).collect();

        // Market breadth analysis
        let positive_sectors = performances
            .iter()
            .filter(|p| {
                p.get("change_1d_pct")
                    .and_then(|v| v.as_f64())
                    .map(|v| v > 0.0)
                    .unwrap_or(false)
            })
            .count();

        let breadth = if positive_sectors >= 9 {
            "Strong bullish breadth"
        } else if positive_sectors >= 6 {
            "Positive breadth"
        } else if positive_sectors >= 4 {
            "Mixed breadth"
        } else if positive_sectors >= 2 {
            "Negative breadth"
        } else {
            "Strong bearish breadth"
        };

        // Sector rotation analysis
        let rotation_signal = self.identify_rotation_pattern(&performances);

        Ok(json!({
            "type": "all_sectors_performance",
            "sectors": performances,
            "summary": {
                "leaders": leaders,
                "laggards": laggards,
                "positive_sectors": positive_sectors,
                "negative_sectors": 11 - positive_sectors,
                "market_breadth": breadth,
                "rotation_signal": rotation_signal,
            },
            "as_of_date": chrono::Utc::now().format("%Y-%m-%d").to_string(),
            "data_source": "Yahoo Finance",
        }))
    }

    /// Analyze sector rotation patterns
    async fn analyze_sector_rotation(&self) -> Result<Value> {
        let performances = self.get_all_sectors_performance().await?;
        
        // Analyze which sectors are showing strength
        let cyclical_strength = self.calculate_group_strength(&performances, "Cyclical");
        let defensive_strength = self.calculate_group_strength(&performances, "Defensive");

        let rotation_phase = if cyclical_strength > defensive_strength {
            if cyclical_strength > 1.0 {
                "Early Expansion - Cyclicals leading"
            } else {
                "Late Expansion - Rotation beginning"
            }
        } else {
            if defensive_strength > 1.0 {
                "Defensive rotation - Risk-off environment"
            } else {
                "Mixed signals - Transition period"
            }
        };

        // Rate sensitive sectors
        let rate_sensitive_perf = self.calculate_rate_sensitive_performance(&performances);
        
        let rate_outlook = if rate_sensitive_perf > 0.0 {
            "Rate-sensitive sectors outperforming - Market may expect rate cuts"
        } else {
            "Rate-sensitive sectors underperforming - Higher-for-longer rate expectations"
        };

        Ok(json!({
            "type": "sector_rotation",
            "analysis": {
                "rotation_phase": rotation_phase,
                "cyclical_strength": cyclical_strength,
                "defensive_strength": defensive_strength,
                "rate_outlook": rate_outlook,
                "rate_sensitive_performance": rate_sensitive_perf,
            },
            "recommendations": self.get_rotation_recommendations(rotation_phase),
            "sector_data": performances,
            "data_source": "Yahoo Finance",
        }))
    }

    /// Compare sectors for relative strength
    async fn compare_sectors(&self) -> Result<Value> {
        let performances = self.get_all_sectors_performance().await?;

        // Rank sectors by multiple timeframes
        let rankings = self.calculate_sector_rankings(&performances);

        Ok(json!({
            "type": "sector_comparison",
            "rankings": rankings,
            "sector_data": performances,
            "data_source": "Yahoo Finance",
        }))
    }

    /// Fetch ETF data for a sector
    async fn fetch_sector_etf_data(&self, sector: Sector) -> Result<Value> {
        let ticker = sector.etf_ticker();
        
        // Get quote data
        let quote = self.yahoo_client.get_quote(ticker).await?;
        
        // Get historical data for performance calculations
        let historical = self.yahoo_client.get_historical_range(ticker, "3mo").await?;

        let current_price = quote.close;
        
        // Calculate 1-day change from historical data
        let (change_1d, change_1d_pct) = if historical.len() >= 2 {
            let prev_close = historical[1].close;
            let change = current_price - prev_close;
            let pct = if prev_close != 0.0 { (change / prev_close) * 100.0 } else { 0.0 };
            (Some(change), Some(pct))
        } else {
            (None, None)
        };

        // Calculate period returns from historical data
        let change_5d_pct = self.calculate_period_return_from_quotes(&historical, 5);
        let change_1m_pct = self.calculate_period_return_from_quotes(&historical, 21);
        let change_3m_pct = self.calculate_period_return_from_quotes(&historical, 63);

        let volume = Some(quote.volume);
        // For avg_volume, we'd need to calculate from historical - simplified here
        let avg_volume = if historical.len() >= 20 {
            let sum: u64 = historical.iter().take(20).map(|q| q.volume).sum();
            Some(sum / 20)
        } else {
            None
        };
        let volume_ratio = match (volume, avg_volume) {
            (Some(v), Some(av)) if av > 0 => Some(v as f64 / av as f64),
            _ => None,
        };

        Ok(json!({
            "sector": sector.name(),
            "etf_ticker": ticker,
            "current_price": current_price,
            "change_1d": change_1d,
            "change_1d_pct": change_1d_pct,
            "change_5d_pct": change_5d_pct,
            "change_1m_pct": change_1m_pct,
            "change_3m_pct": change_3m_pct,
            "volume": volume,
            "avg_volume": avg_volume,
            "volume_ratio": volume_ratio,
            "sensitivity": sector.sensitivity(),
            "rate_sensitivity": sector.rate_sensitivity(),
        }))
    }

    /// Calculate period return from Quote vector
    fn calculate_period_return_from_quotes(&self, quotes: &[crate::api::yahoo::Quote], days: usize) -> Option<f64> {
        if quotes.len() < days {
            return None;
        }

        let current = quotes.first()?.close;
        let past = quotes.get(days.min(quotes.len() - 1))?.close;

        if past != 0.0 {
            Some(((current - past) / past) * 100.0)
        } else {
            None
        }
    }

    /// Get sector description
    fn get_sector_description(&self, sector: Sector) -> &'static str {
        match sector {
            Sector::Technology => "Companies in software, hardware, semiconductors, and IT services",
            Sector::Healthcare => "Pharmaceuticals, biotechnology, medical devices, and healthcare services",
            Sector::Financials => "Banks, insurance companies, asset managers, and financial services",
            Sector::ConsumerDiscretionary => "Retail, automobiles, entertainment, and luxury goods",
            Sector::ConsumerStaples => "Food, beverages, household products, and personal care",
            Sector::Energy => "Oil & gas exploration, production, and energy equipment",
            Sector::Materials => "Chemicals, metals, mining, and construction materials",
            Sector::Industrials => "Aerospace, defense, machinery, and transportation",
            Sector::Utilities => "Electric, gas, and water utilities",
            Sector::RealEstate => "REITs and real estate development",
            Sector::CommunicationServices => "Telecom, media, and interactive entertainment",
        }
    }

    /// Analyze current sector conditions
    fn analyze_sector_conditions(&self, performance: &Value) -> Value {
        let change_1d = performance.get("change_1d_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let change_1m = performance.get("change_1m_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let volume_ratio = performance.get("volume_ratio").and_then(|v| v.as_f64()).unwrap_or(1.0);

        let momentum = if change_1m > 5.0 {
            "Strong uptrend"
        } else if change_1m > 0.0 {
            "Mild uptrend"
        } else if change_1m > -5.0 {
            "Mild downtrend"
        } else {
            "Strong downtrend"
        };

        let volume_signal = if volume_ratio > 1.5 {
            "Heavy volume - Significant interest"
        } else if volume_ratio > 1.1 {
            "Above average volume"
        } else if volume_ratio > 0.9 {
            "Normal volume"
        } else {
            "Light volume - Low conviction"
        };

        let daily_signal = if change_1d > 1.0 {
            "Bullish day"
        } else if change_1d > 0.0 {
            "Mildly positive"
        } else if change_1d > -1.0 {
            "Mildly negative"
        } else {
            "Bearish day"
        };

        json!({
            "momentum": momentum,
            "daily_signal": daily_signal,
            "volume_signal": volume_signal,
        })
    }

    /// Identify rotation pattern from sector performance
    fn identify_rotation_pattern(&self, _performances: &[Value]) -> &'static str {
        // Simplified rotation analysis
        // In a full implementation, would compare cyclical vs defensive performance
        "Mixed signals - Monitor for clearer rotation"
    }

    /// Calculate group strength (cyclical vs defensive)
    fn calculate_group_strength(&self, performances: &Value, group: &str) -> f64 {
        let sectors = performances.get("sectors").and_then(|s| s.as_array());
        
        if let Some(sectors) = sectors {
            let group_sectors: Vec<_> = sectors
                .iter()
                .filter(|s| {
                    s.get("sensitivity")
                        .and_then(|v| v.as_str())
                        .map(|v| v == group)
                        .unwrap_or(false)
                })
                .collect();

            if group_sectors.is_empty() {
                return 0.0;
            }

            let avg_perf: f64 = group_sectors
                .iter()
                .filter_map(|s| s.get("change_1m_pct").and_then(|v| v.as_f64()))
                .sum::<f64>()
                / group_sectors.len() as f64;

            avg_perf
        } else {
            0.0
        }
    }

    /// Calculate rate-sensitive sector performance
    fn calculate_rate_sensitive_performance(&self, performances: &Value) -> f64 {
        let sectors = performances.get("sectors").and_then(|s| s.as_array());
        
        if let Some(sectors) = sectors {
            let rate_sensitive: Vec<_> = sectors
                .iter()
                .filter(|s| {
                    s.get("rate_sensitivity")
                        .and_then(|v| v.as_str())
                        .map(|v| v == "High")
                        .unwrap_or(false)
                })
                .collect();

            if rate_sensitive.is_empty() {
                return 0.0;
            }

            rate_sensitive
                .iter()
                .filter_map(|s| s.get("change_1m_pct").and_then(|v| v.as_f64()))
                .sum::<f64>()
                / rate_sensitive.len() as f64
        } else {
            0.0
        }
    }

    /// Get rotation recommendations
    fn get_rotation_recommendations(&self, phase: &str) -> Vec<&'static str> {
        match phase {
            p if p.contains("Early Expansion") => vec![
                "Consider overweight in Technology, Consumer Discretionary",
                "Industrials may benefit from economic acceleration",
                "Reduce defensive exposure gradually",
            ],
            p if p.contains("Late Expansion") => vec![
                "Consider taking profits in high-beta sectors",
                "Gradually increase defensive positions",
                "Energy and Materials may benefit from inflation",
            ],
            p if p.contains("Defensive") => vec![
                "Favor Healthcare, Consumer Staples, Utilities",
                "Reduce cyclical exposure",
                "Consider dividend-paying sectors",
            ],
            _ => vec![
                "Maintain balanced sector allocation",
                "Focus on quality within each sector",
                "Monitor economic indicators for direction",
            ],
        }
    }

    /// Calculate sector rankings
    fn calculate_sector_rankings(&self, performances: &Value) -> Value {
        let sectors = performances.get("sectors").and_then(|s| s.as_array());
        
        if let Some(sectors) = sectors {
            let mut rankings: Vec<_> = sectors
                .iter()
                .filter_map(|s| {
                    let name = s.get("sector")?.as_str()?;
                    let perf_1d = s.get("change_1d_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let perf_1m = s.get("change_1m_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let perf_3m = s.get("change_3m_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    
                    // Composite score (weighted)
                    let score = perf_1d * 0.2 + perf_1m * 0.4 + perf_3m * 0.4;
                    
                    Some(json!({
                        "sector": name,
                        "score": score,
                        "rank_1d": perf_1d,
                        "rank_1m": perf_1m,
                        "rank_3m": perf_3m,
                    }))
                })
                .collect();

            rankings.sort_by(|a, b| {
                let a_score = a.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let b_score = b.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
            });

            json!(rankings)
        } else {
            json!([])
        }
    }
}

#[async_trait]
impl Tool for SectorAnalysisTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: SectorParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {}", e))
        })?;

        self.fetch_sector_data(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &str {
        "sector_analysis"
    }

    fn description(&self) -> &str {
        "Analyze market sectors using SPDR Select Sector ETFs. \
         Provides sector performance data, rotation analysis, and relative strength rankings. \
         Includes sector characteristics (cyclical/defensive) and rate sensitivity analysis."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "sector": {
                    "type": "string",
                    "description": "Specific sector to analyze (e.g., 'Technology', 'Healthcare'). Omit for all sectors."
                },
                "analysis_type": {
                    "type": "string",
                    "enum": ["performance", "rotation", "comparison"],
                    "description": "Type of analysis: performance data, rotation patterns, or sector comparison",
                    "default": "performance"
                },
                "include_holdings": {
                    "type": "boolean",
                    "description": "Include top holdings for each sector",
                    "default": false
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
    fn test_sector_etf_tickers() {
        assert_eq!(Sector::Technology.etf_ticker(), "XLK");
        assert_eq!(Sector::Healthcare.etf_ticker(), "XLV");
        assert_eq!(Sector::Financials.etf_ticker(), "XLF");
    }

    #[test]
    fn test_sector_from_str() {
        assert_eq!(Sector::from_str("technology"), Some(Sector::Technology));
        assert_eq!(Sector::from_str("tech"), Some(Sector::Technology));
        assert_eq!(Sector::from_str("XLK"), Some(Sector::Technology));
        assert_eq!(Sector::from_str("unknown"), None);
    }

    #[test]
    fn test_sector_sensitivity() {
        assert_eq!(Sector::Technology.sensitivity(), "Cyclical");
        assert_eq!(Sector::Utilities.sensitivity(), "Defensive");
        assert_eq!(Sector::RealEstate.rate_sensitivity(), "High");
    }

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(1800));
        let tool = SectorAnalysisTool::new(config, cache);

        assert_eq!(tool.name(), "sector_analysis");
        assert!(tool.description().contains("sector"));
    }
}
