//! SEC EDGAR API client for fetching company filings and financial reports
//!
//! SEC EDGAR is the Electronic Data Gathering, Analysis, and Retrieval system
//! used by the U.S. Securities and Exchange Commission.
//!
//! Rate limit: 10 requests per second (as per SEC fair access policy)
//! User-Agent requirement: Must include company name and contact email

use crate::error::{Result, StockError};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;

type SharedRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

const SEC_BASE_URL: &str = "https://data.sec.gov";
const SEC_COMPANY_TICKERS_URL: &str = "https://www.sec.gov/files/company_tickers.json";

/// SEC filing type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilingType {
    /// Annual report
    #[serde(rename = "10-K")]
    Form10K,
    /// Quarterly report
    #[serde(rename = "10-Q")]
    Form10Q,
    /// Current report (material events)
    #[serde(rename = "8-K")]
    Form8K,
    /// Proxy statement
    #[serde(rename = "DEF 14A")]
    DefProxy,
    /// Registration statement
    #[serde(rename = "S-1")]
    FormS1,
}

impl FilingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilingType::Form10K => "10-K",
            FilingType::Form10Q => "10-Q",
            FilingType::Form8K => "8-K",
            FilingType::DefProxy => "DEF 14A",
            FilingType::FormS1 => "S-1",
        }
    }
}

/// Company information from SEC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    /// Central Index Key (CIK)
    pub cik: String,
    /// Company name
    pub name: String,
    /// Stock ticker symbol
    pub ticker: String,
    /// Exchange (optional)
    pub exchange: Option<String>,
}

/// SEC filing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFiling {
    /// Accession number (unique filing identifier)
    pub accession_number: String,
    /// Filing type (10-K, 10-Q, 8-K, etc.)
    pub form_type: String,
    /// Filing date
    pub filing_date: String,
    /// Report date (period covered)
    pub report_date: Option<String>,
    /// Primary document filename
    pub primary_document: String,
    /// Primary document description
    pub primary_doc_description: Option<String>,
    /// Filing size in bytes
    pub size: Option<u64>,
    /// Whether filing is XBRL
    pub is_xbrl: bool,
    /// Whether filing is inline XBRL
    pub is_inline_xbrl: bool,
}

/// Financial data from SEC filings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialData {
    /// Revenue / Total Sales
    pub revenue: Option<f64>,
    /// Net Income
    pub net_income: Option<f64>,
    /// Earnings per share (basic)
    pub eps_basic: Option<f64>,
    /// Earnings per share (diluted)
    pub eps_diluted: Option<f64>,
    /// Total Assets
    pub total_assets: Option<f64>,
    /// Total Liabilities
    pub total_liabilities: Option<f64>,
    /// Stockholders' Equity
    pub stockholders_equity: Option<f64>,
    /// Operating Income
    pub operating_income: Option<f64>,
    /// Gross Profit
    pub gross_profit: Option<f64>,
    /// Operating Cash Flow
    pub operating_cash_flow: Option<f64>,
    /// Fiscal year
    pub fiscal_year: String,
    /// Fiscal quarter (if quarterly)
    pub fiscal_quarter: Option<String>,
    /// Filing date
    pub filing_date: String,
}

/// Company facts response from SEC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyFacts {
    pub cik: u64,
    #[serde(rename = "entityName")]
    pub entity_name: String,
    pub facts: Facts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facts {
    #[serde(rename = "us-gaap")]
    pub us_gaap: Option<serde_json::Value>,
    #[serde(rename = "dei")]
    pub dei: Option<serde_json::Value>,
}

/// SEC submissions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanySubmissions {
    pub cik: String,
    pub name: String,
    pub tickers: Vec<String>,
    pub exchanges: Vec<String>,
    pub filings: FilingsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilingsData {
    pub recent: RecentFilings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentFilings {
    pub accession_number: Vec<String>,
    pub filing_date: Vec<String>,
    pub report_date: Vec<Option<String>>,
    pub form: Vec<String>,
    pub primary_document: Vec<String>,
    pub primary_doc_description: Vec<Option<String>>,
    pub size: Vec<Option<u64>>,
    pub is_xbrl: Vec<i32>,
    pub is_inline_xbrl: Vec<i32>,
}

/// SEC EDGAR API client
pub struct SecEdgarClient {
    client: Client,
    user_agent: String,
    rate_limiter: SharedRateLimiter,
}

impl SecEdgarClient {
    /// Create a new SEC EDGAR client
    ///
    /// # Arguments
    /// * `company_name` - Your company/application name
    /// * `contact_email` - Contact email (required by SEC)
    ///
    /// # Example
    /// ```ignore
    /// let client = SecEdgarClient::new("MyApp", "contact@example.com");
    /// ```
    pub fn new(company_name: impl Into<String>, contact_email: impl Into<String>) -> Self {
        let user_agent = format!(
            "{} ({})",
            company_name.into(),
            contact_email.into()
        );
        
        // SEC allows 10 requests per second
        let quota = Quota::per_second(NonZeroU32::new(10).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client: Client::new(),
            user_agent,
            rate_limiter,
        }
    }

    /// Create from environment variables
    /// Uses SEC_USER_AGENT or defaults to "agent-stock (agent-stock@example.com)"
    pub fn from_env() -> Self {
        let user_agent = std::env::var("SEC_USER_AGENT")
            .unwrap_or_else(|_| "agent-stock (agent-stock@example.com)".to_string());
        
        let quota = Quota::per_second(NonZeroU32::new(10).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            client: Client::new(),
            user_agent,
            rate_limiter,
        }
    }

    /// Get CIK number from stock ticker
    pub async fn get_cik(&self, ticker: &str) -> Result<String> {
        self.rate_limiter.until_ready().await;

        let response = self
            .client
            .get(SEC_COMPANY_TICKERS_URL)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("SEC request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StockError::ApiError(format!(
                "SEC API error: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response.json().await
            .map_err(|e| StockError::ApiError(format!("Failed to parse SEC response: {}", e)))?;

        // Search for ticker in company list
        let ticker_upper = ticker.to_uppercase();
        if let Some(companies) = data.as_object() {
            for (_, company) in companies {
                if let Some(company_ticker) = company.get("ticker").and_then(|t| t.as_str()) {
                    if company_ticker.to_uppercase() == ticker_upper {
                        if let Some(cik) = company.get("cik_str").and_then(|c| c.as_str()) {
                            return Ok(cik.to_string());
                        }
                    }
                }
            }
        }

        Err(StockError::InvalidSymbol(ticker.to_string()))
    }

    /// Get company submissions (filing history)
    pub async fn get_company_submissions(&self, cik: &str) -> Result<CompanySubmissions> {
        self.rate_limiter.until_ready().await;

        // Pad CIK to 10 digits
        let cik_padded = format!("{:0>10}", cik.trim_start_matches('0'));
        
        let url = format!("{}/submissions/CIK{}.json", SEC_BASE_URL, cik_padded);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("SEC request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StockError::ApiError(format!(
                "SEC API error: {}",
                response.status()
            )));
        }

        let submissions: CompanySubmissions = response.json().await
            .map_err(|e| StockError::ApiError(format!("Failed to parse SEC response: {}", e)))?;

        Ok(submissions)
    }

    /// Get list of filings for a company
    pub async fn get_filings(
        &self,
        cik: &str,
        form_type: Option<FilingType>,
        limit: Option<usize>,
    ) -> Result<Vec<SecFiling>> {
        let submissions = self.get_company_submissions(cik).await?;
        let recent = &submissions.filings.recent;

        let mut filings = Vec::new();
        let limit = limit.unwrap_or(10);

        for i in 0..recent.accession_number.len().min(limit * 2) {
            let form = &recent.form[i];
            
            // Filter by form type if specified
            if let Some(ref ft) = form_type {
                if form != ft.as_str() {
                    continue;
                }
            }

            filings.push(SecFiling {
                accession_number: recent.accession_number[i].clone(),
                form_type: form.clone(),
                filing_date: recent.filing_date[i].clone(),
                report_date: recent.report_date[i].clone(),
                primary_document: recent.primary_document[i].clone(),
                primary_doc_description: recent.primary_doc_description[i].clone(),
                size: recent.size[i],
                is_xbrl: recent.is_xbrl[i] == 1,
                is_inline_xbrl: recent.is_inline_xbrl[i] == 1,
            });

            if filings.len() >= limit {
                break;
            }
        }

        Ok(filings)
    }

    /// Get company facts (XBRL financial data)
    pub async fn get_company_facts(&self, cik: &str) -> Result<CompanyFacts> {
        self.rate_limiter.until_ready().await;

        let cik_padded = format!("{:0>10}", cik.trim_start_matches('0'));
        let url = format!("{}/api/xbrl/companyfacts/CIK{}.json", SEC_BASE_URL, cik_padded);

        let response = self
            .client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| StockError::ApiError(format!("SEC request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(StockError::ApiError(format!(
                "SEC API error: {}",
                response.status()
            )));
        }

        let facts: CompanyFacts = response.json().await
            .map_err(|e| StockError::ApiError(format!("Failed to parse SEC response: {}", e)))?;

        Ok(facts)
    }

    /// Extract financial data from company facts
    pub fn extract_financial_data(
        &self,
        facts: &CompanyFacts,
        years: Option<u32>,
    ) -> Result<Vec<FinancialData>> {
        let us_gaap = facts.facts.us_gaap.as_ref().ok_or_else(|| {
            StockError::ApiError("No US-GAAP data available".to_string())
        })?;

        let mut financials = Vec::new();
        let years_limit = years.unwrap_or(5) as usize;

        // Helper to extract values from XBRL
        let extract_values = |concept: &str| -> Vec<(String, f64, String, Option<String>)> {
            let mut values = Vec::new();
            if let Some(concept_data) = us_gaap.get(concept) {
                if let Some(units) = concept_data.get("units") {
                    // Try USD first
                    let unit_data = units.get("USD")
                        .or_else(|| units.get("USD/shares"))
                        .or_else(|| units.get("shares"));
                    
                    if let Some(entries) = unit_data.and_then(|u| u.as_array()) {
                        for entry in entries {
                            if let (Some(val), Some(fy), Some(filed)) = (
                                entry.get("val").and_then(|v| v.as_f64()),
                                entry.get("fy").and_then(|y| y.as_i64()),
                                entry.get("filed").and_then(|f| f.as_str()),
                            ) {
                                let fp = entry.get("fp").and_then(|f| f.as_str())
                                    .map(|s| s.to_string());
                                values.push((fy.to_string(), val, filed.to_string(), fp));
                            }
                        }
                    }
                }
            }
            values
        };

        // Extract key financial metrics
        let revenues = extract_values("Revenues");
        let net_incomes = extract_values("NetIncomeLoss");
        let eps_basic_vals = extract_values("EarningsPerShareBasic");
        let eps_diluted_vals = extract_values("EarningsPerShareDiluted");
        let total_assets_vals = extract_values("Assets");
        let total_liabilities_vals = extract_values("Liabilities");
        let equity_vals = extract_values("StockholdersEquity");
        let operating_income_vals = extract_values("OperatingIncomeLoss");

        // Group by fiscal year/quarter
        let mut seen_periods: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for (fy, revenue, filed, fp) in revenues.iter() {
            let period_key = format!("{}-{:?}", fy, fp);
            if seen_periods.contains(&period_key) {
                continue;
            }
            seen_periods.insert(period_key);

            // Find matching values for this period
            let find_match = |vals: &[(String, f64, String, Option<String>)]| -> Option<f64> {
                vals.iter()
                    .find(|(y, _, f, q)| y == fy && f == filed && q == fp)
                    .map(|(_, v, _, _)| *v)
            };

            financials.push(FinancialData {
                revenue: Some(*revenue),
                net_income: find_match(&net_incomes),
                eps_basic: find_match(&eps_basic_vals),
                eps_diluted: find_match(&eps_diluted_vals),
                total_assets: find_match(&total_assets_vals),
                total_liabilities: find_match(&total_liabilities_vals),
                stockholders_equity: find_match(&equity_vals),
                operating_income: find_match(&operating_income_vals),
                gross_profit: None, // Often needs calculation
                operating_cash_flow: None, // In different taxonomy
                fiscal_year: fy.clone(),
                fiscal_quarter: fp.clone(),
                filing_date: filed.clone(),
            });

            if financials.len() >= years_limit * 4 {
                break;
            }
        }

        // Sort by date (most recent first)
        financials.sort_by(|a, b| b.filing_date.cmp(&a.filing_date));

        Ok(financials)
    }

    /// Get financial data for a ticker symbol
    pub async fn get_financial_data(
        &self,
        ticker: &str,
        years: Option<u32>,
    ) -> Result<Vec<FinancialData>> {
        let cik = self.get_cik(ticker).await?;
        let facts = self.get_company_facts(&cik).await?;
        self.extract_financial_data(&facts, years)
    }

    /// Build URL to access a filing document
    pub fn get_filing_url(&self, cik: &str, accession_number: &str, document: &str) -> String {
        let cik_padded = format!("{:0>10}", cik.trim_start_matches('0'));
        let accession_clean = accession_number.replace("-", "");
        format!(
            "https://www.sec.gov/Archives/edgar/data/{}/{}/{}",
            cik_padded, accession_clean, document
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = SecEdgarClient::new("TestApp", "test@example.com");
        assert!(client.user_agent.contains("TestApp"));
        assert!(client.user_agent.contains("test@example.com"));
    }

    #[test]
    fn test_filing_type() {
        assert_eq!(FilingType::Form10K.as_str(), "10-K");
        assert_eq!(FilingType::Form10Q.as_str(), "10-Q");
        assert_eq!(FilingType::Form8K.as_str(), "8-K");
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_cik() {
        let client = SecEdgarClient::from_env();
        let cik = client.get_cik("AAPL").await;
        assert!(cik.is_ok());
        // Apple's CIK is 320193
        assert_eq!(cik.unwrap(), "320193");
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_filings() {
        let client = SecEdgarClient::from_env();
        let cik = client.get_cik("AAPL").await.unwrap();
        let filings = client.get_filings(&cik, Some(FilingType::Form10K), Some(3)).await;
        assert!(filings.is_ok());
        let filings = filings.unwrap();
        assert!(!filings.is_empty());
        assert!(filings.iter().all(|f| f.form_type == "10-K"));
    }
}
