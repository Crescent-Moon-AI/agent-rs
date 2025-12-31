//! Tool for fetching and analyzing company earnings reports
//!
//! Uses SEC EDGAR API to retrieve quarterly (10-Q) and annual (10-K) reports

use agent_core::Result as AgentResult;
use agent_tools::Tool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::api::{SecEdgarClient, FilingType, FinancialData};
use crate::cache::{CacheKey, StockCache};
use crate::config::StockConfig;
use crate::error::Result;

/// Parameters for earnings report requests
#[derive(Debug, Deserialize)]
struct EarningsParams {
    /// Stock ticker symbol
    symbol: String,
    /// Report type: "annual" (10-K), "quarterly" (10-Q), or "all"
    #[serde(default = "default_report_type")]
    report_type: String,
    /// Number of periods to retrieve
    #[serde(default = "default_periods")]
    periods: usize,
}

fn default_report_type() -> String {
    "all".to_string()
}

fn default_periods() -> usize {
    4
}

/// Earnings report analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsReport {
    pub symbol: String,
    pub company_name: String,
    pub fiscal_year: String,
    pub fiscal_quarter: Option<String>,
    pub filing_type: String,
    pub filing_date: String,
    pub revenue: Option<f64>,
    pub revenue_formatted: Option<String>,
    pub net_income: Option<f64>,
    pub net_income_formatted: Option<String>,
    pub eps_basic: Option<f64>,
    pub eps_diluted: Option<f64>,
    pub total_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub stockholders_equity: Option<f64>,
    pub operating_income: Option<f64>,
    pub gross_profit: Option<f64>,
    /// Gross margin percentage
    pub gross_margin: Option<f64>,
    /// Operating margin percentage
    pub operating_margin: Option<f64>,
    /// Net profit margin percentage
    pub net_margin: Option<f64>,
    /// Debt to equity ratio
    pub debt_to_equity: Option<f64>,
    /// Return on equity
    pub roe: Option<f64>,
}

/// Tool for fetching company earnings and financial reports
pub struct EarningsReportTool {
    sec_client: SecEdgarClient,
    cache: StockCache,
    _config: Arc<StockConfig>,
}

impl EarningsReportTool {
    /// Create a new earnings report tool
    pub fn new(config: Arc<StockConfig>, cache: StockCache) -> Self {
        let sec_client = SecEdgarClient::new(
            &config.sec_user_agent,
            &config.sec_contact_email,
        );

        Self {
            sec_client,
            cache,
            _config: config,
        }
    }

    /// Fetch earnings reports for a symbol
    async fn fetch_earnings(&self, params: EarningsParams) -> Result<Value> {
        let symbol = params.symbol.to_uppercase();

        // Create cache key
        let cache_key = CacheKey::new(
            &symbol,
            "earnings",
            json!({
                "type": params.report_type,
                "periods": params.periods
            }),
        );

        // Try to get from cache
        let result = self
            .cache
            .get_or_fetch(cache_key, || async {
                self.fetch_from_sec(&symbol, &params.report_type, params.periods)
                    .await
            })
            .await?;

        Ok(result)
    }

    /// Fetch earnings data from SEC EDGAR
    async fn fetch_from_sec(
        &self,
        symbol: &str,
        report_type: &str,
        periods: usize,
    ) -> Result<Value> {
        // Get CIK for the symbol
        let cik = self.sec_client.get_cik(symbol).await?;

        // Determine filing type
        let filing_type = match report_type.to_lowercase().as_str() {
            "annual" | "10-k" | "10k" => Some(FilingType::Form10K),
            "quarterly" | "10-q" | "10q" => Some(FilingType::Form10Q),
            _ => None, // Get both
        };

        // Get filings list
        let filings = self
            .sec_client
            .get_filings(&cik, filing_type, Some(periods * 2))
            .await?;

        // Get financial data from XBRL
        let financial_data = self
            .sec_client
            .get_financial_data(symbol, Some(periods as u32))
            .await
            .unwrap_or_default();

        // Build reports
        let reports: Vec<EarningsReport> = financial_data
            .iter()
            .take(periods)
            .map(|fd| self.build_earnings_report(symbol, fd))
            .collect();

        // Build SEC filings list
        let filing_list: Vec<Value> = filings
            .iter()
            .take(periods)
            .map(|f| {
                json!({
                    "form_type": f.form_type,
                    "filing_date": f.filing_date,
                    "report_date": f.report_date,
                    "document": f.primary_document,
                    "url": self.sec_client.get_filing_url(&cik, &f.accession_number, &f.primary_document),
                })
            })
            .collect();

        // Calculate trends if we have multiple periods
        let trends = if reports.len() >= 2 {
            self.calculate_trends(&reports)
        } else {
            json!({})
        };

        Ok(json!({
            "symbol": symbol,
            "cik": cik,
            "report_type": report_type,
            "periods_returned": reports.len(),
            "reports": reports,
            "filings": filing_list,
            "trends": trends,
            "data_source": "SEC EDGAR",
        }))
    }

    /// Build earnings report from financial data
    fn build_earnings_report(&self, symbol: &str, fd: &FinancialData) -> EarningsReport {
        // Calculate margins
        let gross_margin = match (fd.gross_profit, fd.revenue) {
            (Some(gp), Some(rev)) if rev != 0.0 => Some((gp / rev) * 100.0),
            _ => None,
        };

        let operating_margin = match (fd.operating_income, fd.revenue) {
            (Some(op), Some(rev)) if rev != 0.0 => Some((op / rev) * 100.0),
            _ => None,
        };

        let net_margin = match (fd.net_income, fd.revenue) {
            (Some(ni), Some(rev)) if rev != 0.0 => Some((ni / rev) * 100.0),
            _ => None,
        };

        // Calculate debt to equity
        let debt_to_equity = match (fd.total_liabilities, fd.stockholders_equity) {
            (Some(debt), Some(equity)) if equity != 0.0 => Some(debt / equity),
            _ => None,
        };

        // Calculate ROE
        let roe = match (fd.net_income, fd.stockholders_equity) {
            (Some(ni), Some(equity)) if equity != 0.0 => Some((ni / equity) * 100.0),
            _ => None,
        };

        EarningsReport {
            symbol: symbol.to_string(),
            company_name: String::new(), // Would need additional lookup
            fiscal_year: fd.fiscal_year.clone(),
            fiscal_quarter: fd.fiscal_quarter.clone(),
            filing_type: if fd.fiscal_quarter.is_some() {
                "10-Q".to_string()
            } else {
                "10-K".to_string()
            },
            filing_date: fd.filing_date.clone(),
            revenue: fd.revenue,
            revenue_formatted: fd.revenue.map(format_currency),
            net_income: fd.net_income,
            net_income_formatted: fd.net_income.map(format_currency),
            eps_basic: fd.eps_basic,
            eps_diluted: fd.eps_diluted,
            total_assets: fd.total_assets,
            total_liabilities: fd.total_liabilities,
            stockholders_equity: fd.stockholders_equity,
            operating_income: fd.operating_income,
            gross_profit: fd.gross_profit,
            gross_margin,
            operating_margin,
            net_margin,
            debt_to_equity,
            roe,
        }
    }

    /// Calculate trends across periods
    fn calculate_trends(&self, reports: &[EarningsReport]) -> Value {
        let latest = &reports[0];
        let previous = &reports[1];

        let revenue_growth = match (latest.revenue, previous.revenue) {
            (Some(curr), Some(prev)) if prev != 0.0 => {
                Some(((curr - prev) / prev) * 100.0)
            }
            _ => None,
        };

        let net_income_growth = match (latest.net_income, previous.net_income) {
            (Some(curr), Some(prev)) if prev != 0.0 => {
                Some(((curr - prev) / prev) * 100.0)
            }
            _ => None,
        };

        let eps_growth = match (latest.eps_diluted, previous.eps_diluted) {
            (Some(curr), Some(prev)) if prev != 0.0 => {
                Some(((curr - prev) / prev) * 100.0)
            }
            _ => None,
        };

        let margin_change = match (latest.net_margin, previous.net_margin) {
            (Some(curr), Some(prev)) => Some(curr - prev),
            _ => None,
        };

        // Determine overall trend
        let trend_assessment = self.assess_trend(
            revenue_growth,
            net_income_growth,
            eps_growth,
        );

        json!({
            "revenue_growth_pct": revenue_growth,
            "net_income_growth_pct": net_income_growth,
            "eps_growth_pct": eps_growth,
            "margin_change_ppt": margin_change,
            "trend_assessment": trend_assessment,
            "comparison_periods": format!("{} Q{} vs {} Q{}",
                latest.fiscal_year,
                latest.fiscal_quarter.as_deref().unwrap_or("FY"),
                previous.fiscal_year,
                previous.fiscal_quarter.as_deref().unwrap_or("FY")
            ),
        })
    }

    /// Assess overall trend based on metrics
    fn assess_trend(
        &self,
        revenue_growth: Option<f64>,
        net_income_growth: Option<f64>,
        eps_growth: Option<f64>,
    ) -> String {
        let positive_signals = [revenue_growth, net_income_growth, eps_growth]
            .iter()
            .filter_map(|&x| x)
            .filter(|&x| x > 0.0)
            .count();

        let total_signals = [revenue_growth, net_income_growth, eps_growth]
            .iter()
            .filter(|x| x.is_some())
            .count();

        if total_signals == 0 {
            return "Insufficient data".to_string();
        }

        match (positive_signals, total_signals) {
            (p, t) if p == t => "Strong growth across all metrics".to_string(),
            (p, t) if p as f64 / t as f64 >= 0.66 => "Positive trend overall".to_string(),
            (p, t) if p as f64 / t as f64 >= 0.33 => "Mixed performance".to_string(),
            _ => "Declining performance".to_string(),
        }
    }
}

/// Format currency in human-readable form
fn format_currency(amount: f64) -> String {
    let abs_amount = amount.abs();
    let sign = if amount < 0.0 { "-" } else { "" };
    
    if abs_amount >= 1_000_000_000_000.0 {
        format!("{}${:.2}T", sign, abs_amount / 1_000_000_000_000.0)
    } else if abs_amount >= 1_000_000_000.0 {
        format!("{}${:.2}B", sign, abs_amount / 1_000_000_000.0)
    } else if abs_amount >= 1_000_000.0 {
        format!("{}${:.2}M", sign, abs_amount / 1_000_000.0)
    } else if abs_amount >= 1_000.0 {
        format!("{}${:.2}K", sign, abs_amount / 1_000.0)
    } else {
        format!("{sign}${abs_amount:.2}")
    }
}

#[async_trait]
impl Tool for EarningsReportTool {
    async fn execute(&self, params: Value) -> AgentResult<Value> {
        let params: EarningsParams = serde_json::from_value(params).map_err(|e| {
            agent_core::Error::ProcessingFailed(format!("Invalid parameters: {e}"))
        })?;

        self.fetch_earnings(params)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "earnings_report"
    }

    fn description(&self) -> &'static str {
        "Fetch and analyze company earnings reports from SEC EDGAR. \
         Returns quarterly (10-Q) and annual (10-K) financial data including revenue, \
         net income, EPS, margins, and financial ratios. Also provides trend analysis \
         comparing periods."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "symbol": {
                    "type": "string",
                    "description": "Stock ticker symbol (e.g., AAPL, MSFT)"
                },
                "report_type": {
                    "type": "string",
                    "enum": ["annual", "quarterly", "all"],
                    "description": "Type of report: 'annual' for 10-K, 'quarterly' for 10-Q, 'all' for both",
                    "default": "all"
                },
                "periods": {
                    "type": "integer",
                    "description": "Number of periods to retrieve (default: 4)",
                    "default": 4,
                    "minimum": 1,
                    "maximum": 20
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
    fn test_format_currency() {
        assert_eq!(format_currency(1_500_000_000_000.0), "$1.50T");
        assert_eq!(format_currency(50_000_000_000.0), "$50.00B");
        assert_eq!(format_currency(250_000_000.0), "$250.00M");
        assert_eq!(format_currency(-1_000_000.0), "-$1.00M");
        assert_eq!(format_currency(5_000.0), "$5.00K");
        assert_eq!(format_currency(100.0), "$100.00");
    }

    #[test]
    fn test_tool_metadata() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(3600 * 24));
        let tool = EarningsReportTool::new(config, cache);

        assert_eq!(tool.name(), "earnings_report");
        assert!(!tool.description().is_empty());
        assert!(tool.input_schema()["properties"]["symbol"].is_object());
    }

    #[test]
    fn test_trend_assessment() {
        let config = Arc::new(StockConfig::default());
        let cache = StockCache::new(Duration::from_secs(3600 * 24));
        let tool = EarningsReportTool::new(config, cache);

        // All positive
        let result = tool.assess_trend(Some(10.0), Some(15.0), Some(20.0));
        assert!(result.contains("Strong growth"));

        // Mostly positive
        let result = tool.assess_trend(Some(10.0), Some(15.0), Some(-5.0));
        assert!(result.contains("Positive"));

        // Mixed
        let result = tool.assess_trend(Some(10.0), Some(-15.0), Some(-5.0));
        assert!(result.contains("Mixed") || result.contains("Declining"));

        // All negative
        let result = tool.assess_trend(Some(-10.0), Some(-15.0), Some(-5.0));
        assert!(result.contains("Declining"));
    }
}
