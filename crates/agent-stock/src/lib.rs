//! Stock analysis agent framework
//!
//! This crate provides comprehensive stock market analysis capabilities through
//! a multi-agent architecture. It includes:
//!
//! - Data fetching from multiple sources (Yahoo Finance, Alpha Vantage)
//! - Technical analysis with 70+ indicators (RSI, MACD, Bollinger Bands, etc.)
//! - Fundamental analysis (P/E ratios, market cap, financials)
//! - News and sentiment analysis
//! - Multi-agent coordination via delegating agent pattern
//!
//! # Architecture
//!
//! The system uses a delegating agent (`StockAnalysisAgent`) that routes requests
//! to specialized sub-agents:
//! - `DataFetcherAgent`: Retrieves stock data
//! - `TechnicalAnalyzerAgent`: Performs technical analysis
//! - `FundamentalAnalyzerAgent`: Analyzes fundamentals
//! - `NewsAnalyzerAgent`: Analyzes news and sentiment
//!
//! # Example
//!
//! ```rust,no_run
//! use agent_stock::StockAnalysisAgent;
//! use agent_runtime::AgentRuntime;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create runtime with LLM provider
//!     let runtime = AgentRuntime::builder()
//!         .provider(/* your provider */)
//!         .build()?;
//!
//!     // Create stock analysis agent
//!     let agent = StockAnalysisAgent::new(runtime).await?;
//!
//!     // Analyze a stock
//!     let result = agent.analyze("AAPL").await?;
//!     println!("{}", result);
//!
//!     Ok(())
//! }
//! ```

pub mod agents;
pub mod api;
pub mod cache;
pub mod config;
pub mod error;
pub mod tools;

// Re-export main types for convenience
pub use agents::stock_analysis::StockAnalysisAgent;
pub use config::StockConfig;
pub use error::{Result, StockError};
