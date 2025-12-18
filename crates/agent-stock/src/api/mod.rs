//! API clients for stock data providers

pub mod alpha_vantage;
pub mod fred;
pub mod news_apis;
pub mod sec_edgar;
pub mod yahoo;

pub use alpha_vantage::{
    AlphaVantageClient, NewsArticle, NewsSentimentResponse, NewsTopic, TickerSentiment,
};
pub use fred::{FredClient, EconomicSummary, series as fred_series};
pub use news_apis::FinnhubClient;
pub use sec_edgar::{SecEdgarClient, SecFiling, FinancialData, FilingType};
pub use yahoo::YahooFinanceClient;
