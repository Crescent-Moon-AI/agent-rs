//! Stock analysis agent framework
//!
//! This crate provides comprehensive stock market analysis capabilities through
//! a multi-agent architecture. It includes:
//!
//! - Data fetching from multiple sources (Yahoo Finance, Alpha Vantage)
//! - Technical analysis with 70+ indicators (RSI, MACD, Bollinger Bands, etc.)
//! - Fundamental analysis (P/E ratios, market cap, financials)
//! - News and sentiment analysis
//! - Earnings report analysis (SEC EDGAR 10-K/10-Q filings)
//! - Macroeconomic analysis (Fed policy, economic indicators)
//! - Geopolitical risk assessment
//! - Multi-agent coordination via delegating agent pattern
//! - Interactive bot with conversation context support
//!
//! # Architecture
//!
//! The system uses a delegating agent (`StockAnalysisAgent`) that routes requests
//! to specialized sub-agents:
//! - `DataFetcherAgent`: Retrieves stock data
//! - `TechnicalAnalyzerAgent`: Performs technical analysis
//! - `FundamentalAnalyzerAgent`: Analyzes fundamentals
//! - `NewsAnalyzerAgent`: Analyzes news and sentiment
//! - `EarningsAnalyzerAgent`: Analyzes SEC filings and earnings reports
//! - `MacroAnalyzerAgent`: Analyzes macroeconomic conditions
//!
//! # Features
//!
//! - **Smart Routing**: Queries are automatically routed to the appropriate agent
//! - **Parallel Execution**: Comprehensive analysis runs multiple agents in parallel
//! - **Conversation Context**: Follow-up questions understand previous context
//! - **Stock Comparison**: Compare multiple stocks side by side
//! - **Watchlist**: Track stocks of interest
//!
//! # Example
//!
//! ```rust,ignore
//! use agent_stock::{StockAnalysisAgent, StockConfig};
//! use agent_runtime::AgentRuntime;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create runtime with LLM provider
//!     let runtime = Arc::new(AgentRuntime::builder()
//!         .provider(/* your provider */)
//!         .build()?);
//!
//!     // Create config
//!     let config = Arc::new(StockConfig::default());
//!
//!     // Create stock analysis agent
//!     let agent = StockAnalysisAgent::new(runtime, config).await?;
//!
//!     // Analyze a stock
//!     let result = agent.analyze("AAPL").await?;
//!     println!("{}", result);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Bot Usage
//!
//! ```rust,ignore
//! use agent_stock::bot::{StockBot, BotConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = BotConfig::from_env()?;
//!     let bot = StockBot::with_provider(provider, config).await?;
//!
//!     // Process commands
//!     let response = bot.process_input("/analyze AAPL").await?;
//!     println!("{}", response);
//!
//!     Ok(())
//! }
//! ```

pub mod agents;
pub mod api;
pub mod bot;
pub mod cache;
pub mod config;
pub mod engine;
pub mod error;
pub mod interface;
pub mod platforms;
pub mod prompts;
pub mod router;
pub mod tools;

// Re-export main types for convenience
pub use agents::{
    DataFetcherAgent, EarningsAnalyzerAgent, FundamentalAnalyzerAgent,
    MacroAnalyzerAgent, NewsAnalyzerAgent, StockAnalysisAgent, TechnicalAnalyzerAgent,
    ParallelAnalysisResult,
};
pub use engine::{
    StockAnalysisEngine, AnalysisContext, AnalysisResult, AnalysisType, ComparisonResult,
};
pub use config::StockConfig;
pub use error::{Result, StockError};
pub use router::{QueryIntent, SmartRouter, RoutingResult};

// Re-export cache utilities
pub use cache::{CacheManager, CacheTtlConfig, CacheStats, shared_cache, init_shared_cache};

// Re-export Language from agent-prompt
pub use agent_prompt::Language;

// Re-export commonly used tools
pub use tools::{
    EarningsReportTool, GeopoliticalTool, MacroEconomicTool,
    SectorAnalysisTool,
};
