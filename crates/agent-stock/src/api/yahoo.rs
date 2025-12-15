//! Yahoo Finance API client

use crate::error::{Result, StockError};
use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use yahoo_finance_api as yahoo;

/// Yahoo Finance API client
pub struct YahooFinanceClient {}

/// Stock quote data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adjclose: f64,
}

/// Company information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub symbol: String,
    pub name: Option<String>,
    pub exchange: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_yield: Option<f64>,
}

impl YahooFinanceClient {
    /// Create a new Yahoo Finance client
    pub fn new() -> Self {
        Self {}
    }

    /// Get the latest quote for a symbol
    pub async fn get_quote(&self, symbol: &str) -> Result<Quote> {
        let provider = yahoo::YahooConnector::new()
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        let response = provider
            .get_latest_quotes(symbol, "1d")
            .await
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        let quote = response
            .last_quote()
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        Ok(Quote {
            symbol: symbol.to_string(),
            timestamp: DateTime::from_timestamp(quote.timestamp as i64, 0)
                .unwrap_or_else(Utc::now),
            open: quote.open,
            high: quote.high,
            low: quote.low,
            close: quote.close,
            volume: quote.volume,
            adjclose: quote.adjclose,
        })
    }

    /// Get historical quotes for a symbol
    pub async fn get_historical_quotes(
        &self,
        symbol: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Quote>> {
        let provider = yahoo::YahooConnector::new()
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        // Convert chrono DateTime to time OffsetDateTime
        let start_odt = OffsetDateTime::from_unix_timestamp(start.timestamp())
            .map_err(|e| StockError::YahooFinanceError(format!("Invalid start timestamp: {}", e)))?;
        let end_odt = OffsetDateTime::from_unix_timestamp(end.timestamp())
            .map_err(|e| StockError::YahooFinanceError(format!("Invalid end timestamp: {}", e)))?;

        let response = provider
            .get_quote_history(symbol, start_odt, end_odt)
            .await
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        let quotes = response
            .quotes()
            .map_err(|e| StockError::YahooFinanceError(e.to_string()))?;

        Ok(quotes
            .iter()
            .map(|q| Quote {
                symbol: symbol.to_string(),
                timestamp: DateTime::from_timestamp(q.timestamp as i64, 0)
                    .unwrap_or_else(Utc::now),
                open: q.open,
                high: q.high,
                low: q.low,
                close: q.close,
                volume: q.volume,
                adjclose: q.adjclose,
            })
            .collect())
    }

    /// Get historical quotes with a specific range
    pub async fn get_historical_range(
        &self,
        symbol: &str,
        range: &str,  // e.g., "1mo", "3mo", "1y"
    ) -> Result<Vec<Quote>> {
        let end = Utc::now();
        let start = match range {
            "1d" => end - chrono::Duration::days(1),
            "5d" => end - chrono::Duration::days(5),
            "1mo" => end - chrono::Duration::days(30),
            "3mo" => end - chrono::Duration::days(90),
            "6mo" => end - chrono::Duration::days(180),
            "1y" => end - chrono::Duration::days(365),
            "2y" => end - chrono::Duration::days(730),
            "5y" => end - chrono::Duration::days(1825),
            "10y" => end - chrono::Duration::days(3650),
            "ytd" => {
                let year = end.year();
                chrono::NaiveDate::from_ymd_opt(year, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            },
            "max" => end - chrono::Duration::days(36500), // ~100 years
            _ => return Err(StockError::InvalidSymbol(format!("Invalid range: {}", range))),
        };

        self.get_historical_quotes(symbol, start, end).await
    }

    /// Get company information (basic implementation - Yahoo Finance API has limited support)
    pub async fn get_company_info(&self, symbol: &str) -> Result<CompanyInfo> {
        // Yahoo Finance API doesn't provide a direct company info endpoint in the rust client
        // We'll return basic info that we can infer
        Ok(CompanyInfo {
            symbol: symbol.to_string(),
            name: None,
            exchange: None,
            sector: None,
            industry: None,
            market_cap: None,
            pe_ratio: None,
            dividend_yield: None,
        })
    }

    /// Validate if a symbol exists by attempting to fetch its quote
    pub async fn validate_symbol(&self, symbol: &str) -> Result<bool> {
        match self.get_quote(symbol).await {
            Ok(_) => Ok(true),
            Err(StockError::YahooFinanceError(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

impl Default for YahooFinanceClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for YahooFinanceClient {
    fn clone(&self) -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_quote() {
        let client = YahooFinanceClient::new();
        let quote = client.get_quote("AAPL").await;
        assert!(quote.is_ok());

        let quote = quote.unwrap();
        assert_eq!(quote.symbol, "AAPL");
        assert!(quote.close > 0.0);
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_historical_range() {
        let client = YahooFinanceClient::new();
        let quotes = client.get_historical_range("AAPL", "1mo").await;
        assert!(quotes.is_ok());

        let quotes = quotes.unwrap();
        assert!(!quotes.is_empty());
        assert_eq!(quotes[0].symbol, "AAPL");
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_validate_symbol() {
        let client = YahooFinanceClient::new();

        assert!(client.validate_symbol("AAPL").await.unwrap());
        assert!(!client.validate_symbol("INVALID_SYMBOL_12345").await.unwrap());
    }
}
