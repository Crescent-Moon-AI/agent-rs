//! Top-level stock analysis agent that delegates to specialists
//!
//! This module provides the main entry point for stock analysis, with support for:
//! - Smart routing based on query intent
//! - Parallel execution of multiple agents for comprehensive analysis
//! - Context-aware processing

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, agents::DelegatingAgentBuilder};
use async_trait::async_trait;
use std::sync::Arc;

use super::{
    DataFetcherAgent, EarningsAnalyzerAgent, FundamentalAnalyzerAgent,
    MacroAnalyzerAgent, NewsAnalyzerAgent, TechnicalAnalyzerAgent,
};
use crate::config::StockConfig;
use crate::router::{QueryIntent, SmartRouter};

/// Top-level stock analysis agent that delegates to specialists
pub struct StockAnalysisAgent {
    agent: agent_runtime::agents::DelegatingAgent,
    router: SmartRouter,
    // Store individual agents for parallel execution
    data_fetcher: Arc<DataFetcherAgent>,
    technical_analyzer: Arc<TechnicalAnalyzerAgent>,
    fundamental_analyzer: Arc<FundamentalAnalyzerAgent>,
    news_analyzer: Arc<NewsAnalyzerAgent>,
    earnings_analyzer: Arc<EarningsAnalyzerAgent>,
    macro_analyzer: Arc<MacroAnalyzerAgent>,
}

impl StockAnalysisAgent {
    /// Create a new stock analysis agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create specialist agents
        let data_fetcher =
            Arc::new(DataFetcherAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let technical_analyzer =
            Arc::new(TechnicalAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let fundamental_analyzer = Arc::new(
            FundamentalAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?,
        );
        let news_analyzer =
            Arc::new(NewsAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let earnings_analyzer =
            Arc::new(EarningsAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let macro_analyzer =
            Arc::new(MacroAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);

        // Create smart router
        let smart_router = SmartRouter::new();

        // Create routing function using smart router
        let routing_fn = |input: &str, _context: &Context| -> String {
            let router = SmartRouter::new();
            let intent = router.classify(input);
            intent.agent_name().to_string()
        };

        // Build delegating agent with all sub-agents
        // Clone as Arc<dyn Agent> for the delegating agent builder
        let agent = DelegatingAgentBuilder::new(Arc::clone(&runtime), "stock-analysis")
            .add_agent("data-fetcher", Arc::clone(&data_fetcher) as Arc<dyn Agent>)
            .add_agent("technical-analyzer", Arc::clone(&technical_analyzer) as Arc<dyn Agent>)
            .add_agent("fundamental-analyzer", Arc::clone(&fundamental_analyzer) as Arc<dyn Agent>)
            .add_agent("news-analyzer", Arc::clone(&news_analyzer) as Arc<dyn Agent>)
            .add_agent("earnings-analyzer", Arc::clone(&earnings_analyzer) as Arc<dyn Agent>)
            .add_agent("macro-analyzer", Arc::clone(&macro_analyzer) as Arc<dyn Agent>)
            .router(routing_fn)
            .build()?;

        Ok(Self {
            agent,
            router: smart_router,
            data_fetcher,
            technical_analyzer,
            fundamental_analyzer,
            news_analyzer,
            earnings_analyzer,
            macro_analyzer,
        })
    }

    /// Execute parallel analysis across all agents for comprehensive results
    async fn parallel_analysis(&self, symbol: &str) -> Result<ParallelAnalysisResult> {
        tracing::info!("Starting parallel analysis for {}", symbol);

        // Execute all analyses in parallel
        let (technical, fundamental, news, earnings, macro_result) = tokio::join!(
            self.run_technical(symbol),
            self.run_fundamental(symbol),
            self.run_news(symbol),
            self.run_earnings(symbol),
            self.run_macro(),
        );

        Ok(ParallelAnalysisResult {
            symbol: symbol.to_string(),
            technical: technical.ok(),
            fundamental: fundamental.ok(),
            news: news.ok(),
            earnings: earnings.ok(),
            macro_analysis: macro_result.ok(),
        })
    }

    async fn run_technical(&self, symbol: &str) -> Result<String> {
        let mut ctx = Context::new();
        let input = format!("Perform technical analysis on {} using RSI, MACD, and moving averages.", symbol);
        self.technical_analyzer.process(input, &mut ctx).await
    }

    async fn run_fundamental(&self, symbol: &str) -> Result<String> {
        let mut ctx = Context::new();
        let input = format!("Analyze the fundamental metrics and valuation of {}.", symbol);
        self.fundamental_analyzer.process(input, &mut ctx).await
    }

    async fn run_news(&self, symbol: &str) -> Result<String> {
        let mut ctx = Context::new();
        let input = format!("Analyze recent news and market sentiment for {}.", symbol);
        self.news_analyzer.process(input, &mut ctx).await
    }

    async fn run_earnings(&self, symbol: &str) -> Result<String> {
        let mut ctx = Context::new();
        let input = format!("Analyze the earnings reports and financial statements for {}.", symbol);
        self.earnings_analyzer.process(input, &mut ctx).await
    }

    async fn run_macro(&self) -> Result<String> {
        let mut ctx = Context::new();
        let input = "Analyze the current macroeconomic environment, including Fed policy, inflation, and economic indicators.".to_string();
        self.macro_analyzer.process(input, &mut ctx).await
    }

    /// Get the router for external use
    pub fn router(&self) -> &SmartRouter {
        &self.router
    }

    /// Analyze a stock symbol with comprehensive analysis
    pub async fn analyze(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!(
            "Provide a comprehensive analysis of {} including current price, \
             technical indicators, fundamental metrics, recent earnings, and news.",
            symbol
        );
        self.process(input, &mut context).await
    }

    /// Get technical analysis only
    pub async fn analyze_technical(&self, symbol: &str) -> Result<String> {
        self.run_technical(symbol).await
    }

    /// Get fundamental analysis only
    pub async fn analyze_fundamental(&self, symbol: &str) -> Result<String> {
        self.run_fundamental(symbol).await
    }

    /// Get news and sentiment analysis only
    pub async fn analyze_news(&self, symbol: &str) -> Result<String> {
        self.run_news(symbol).await
    }

    /// Get earnings analysis
    pub async fn analyze_earnings(&self, symbol: &str) -> Result<String> {
        self.run_earnings(symbol).await
    }

    /// Get macro economic analysis
    pub async fn analyze_macro(&self) -> Result<String> {
        self.run_macro().await
    }

    /// Get geopolitical analysis
    pub async fn analyze_geopolitical(&self) -> Result<String> {
        let mut context = Context::new();
        let input = "Analyze current geopolitical risks and their potential market impact.".to_string();
        self.macro_analyzer.process(input, &mut context).await
    }

    /// Get comprehensive analysis including macro factors using parallel execution
    ///
    /// This method executes all analyses in parallel for better performance,
    /// then synthesizes the results into a comprehensive report.
    pub async fn analyze_comprehensive(&self, symbol: &str) -> Result<String> {
        let result = self.parallel_analysis(symbol).await?;
        Ok(result.format_report())
    }

    /// Smart process: automatically determines the best way to handle a query
    pub async fn smart_process(&self, query: &str, context: &mut Context) -> Result<String> {
        let intent = self.router.classify(query);

        match intent {
            QueryIntent::ComprehensiveAnalysis => {
                // Extract symbol from query
                let symbols = self.router.extract_symbols(query);
                if let Some(symbol) = symbols.first() {
                    self.analyze_comprehensive(symbol).await
                } else {
                    // No symbol found, use standard processing
                    self.process(query.to_string(), context).await
                }
            }
            QueryIntent::Comparison => {
                let symbols = self.router.extract_symbols(query);
                if symbols.len() >= 2 {
                    self.compare_stocks(&symbols).await
                } else {
                    self.process(query.to_string(), context).await
                }
            }
            _ => {
                // Single agent processing via delegating agent
                self.process(query.to_string(), context).await
            }
        }
    }

    /// Compare multiple stocks
    pub async fn compare_stocks(&self, symbols: &[String]) -> Result<String> {
        if symbols.is_empty() {
            return Err(agent_core::Error::ProcessingFailed(
                "No symbols provided for comparison".to_string(),
            ));
        }

        // Execute analyses in parallel for all symbols
        let futures: Vec<_> = symbols
            .iter()
            .map(|s| self.parallel_analysis(s))
            .collect();

        let results = futures::future::join_all(futures).await;

        // Format comparison report
        let mut report = String::new();
        report.push_str(&format!("# Stock Comparison: {}\n\n", symbols.join(" vs ")));

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(analysis) => {
                    report.push_str(&format!("## {}\n\n", symbols[i]));
                    report.push_str(&analysis.format_summary());
                    report.push_str("\n\n");
                }
                Err(e) => {
                    report.push_str(&format!("## {} (Error)\n\n", symbols[i]));
                    report.push_str(&format!("Failed to analyze: {}\n\n", e));
                }
            }
        }

        Ok(report)
    }
}

/// Result of parallel analysis across multiple agents
#[derive(Debug, Clone)]
pub struct ParallelAnalysisResult {
    /// Stock symbol
    pub symbol: String,
    /// Technical analysis result
    pub technical: Option<String>,
    /// Fundamental analysis result
    pub fundamental: Option<String>,
    /// News analysis result
    pub news: Option<String>,
    /// Earnings analysis result
    pub earnings: Option<String>,
    /// Macro analysis result
    pub macro_analysis: Option<String>,
}

impl ParallelAnalysisResult {
    /// Format results into a comprehensive report
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("# Comprehensive Analysis: {}\n\n", self.symbol));

        if let Some(ref technical) = self.technical {
            report.push_str("## Technical Analysis\n\n");
            report.push_str(technical);
            report.push_str("\n\n");
        }

        if let Some(ref fundamental) = self.fundamental {
            report.push_str("## Fundamental Analysis\n\n");
            report.push_str(fundamental);
            report.push_str("\n\n");
        }

        if let Some(ref earnings) = self.earnings {
            report.push_str("## Earnings Analysis\n\n");
            report.push_str(earnings);
            report.push_str("\n\n");
        }

        if let Some(ref news) = self.news {
            report.push_str("## News & Sentiment\n\n");
            report.push_str(news);
            report.push_str("\n\n");
        }

        if let Some(ref macro_analysis) = self.macro_analysis {
            report.push_str("## Macro Environment\n\n");
            report.push_str(macro_analysis);
            report.push_str("\n\n");
        }

        report
    }

    /// Format a brief summary for comparison
    pub fn format_summary(&self) -> String {
        let mut summary = String::new();

        if let Some(ref technical) = self.technical {
            // Extract first paragraph or first 200 chars
            let excerpt = technical.lines().next().unwrap_or("").chars().take(200).collect::<String>();
            summary.push_str(&format!("**Technical**: {}\n", excerpt));
        }

        if let Some(ref fundamental) = self.fundamental {
            let excerpt = fundamental.lines().next().unwrap_or("").chars().take(200).collect::<String>();
            summary.push_str(&format!("**Fundamental**: {}\n", excerpt));
        }

        summary
    }

    /// Check if all analyses succeeded
    pub fn is_complete(&self) -> bool {
        self.technical.is_some()
            && self.fundamental.is_some()
            && self.news.is_some()
            && self.earnings.is_some()
            && self.macro_analysis.is_some()
    }

    /// Count successful analyses
    pub fn success_count(&self) -> usize {
        [
            self.technical.is_some(),
            self.fundamental.is_some(),
            self.news.is_some(),
            self.earnings.is_some(),
            self.macro_analysis.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

#[async_trait]
impl Agent for StockAnalysisAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "StockAnalysisAgent"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_routing() {
        let router = SmartRouter::new();

        // Test price query routing
        let intent = router.classify("What's the price of AAPL?");
        assert_eq!(intent.agent_name(), "data-fetcher");

        // Test technical analysis routing
        let intent = router.classify("Calculate RSI for GOOGL");
        assert_eq!(intent.agent_name(), "technical-analyzer");

        // Test fundamental analysis routing
        let intent = router.classify("What's the P/E ratio of MSFT?");
        assert_eq!(intent.agent_name(), "fundamental-analyzer");

        // Test news routing (using explicit news keyword)
        let intent = router.classify("Show me news about TSLA");
        assert_eq!(intent.agent_name(), "news-analyzer");
    }

    #[test]
    fn test_comprehensive_detection() {
        let router = SmartRouter::new();

        let intent = router.classify("Give me a comprehensive analysis of AAPL");
        assert_eq!(intent, QueryIntent::ComprehensiveAnalysis);

        let intent = router.classify("全面分析特斯拉");
        assert_eq!(intent, QueryIntent::ComprehensiveAnalysis);
    }

    #[test]
    fn test_parallel_analysis_result() {
        let result = ParallelAnalysisResult {
            symbol: "AAPL".to_string(),
            technical: Some("RSI: 55".to_string()),
            fundamental: Some("P/E: 28".to_string()),
            news: None,
            earnings: Some("Q4 beat estimates".to_string()),
            macro_analysis: None,
        };

        assert!(!result.is_complete());
        assert_eq!(result.success_count(), 3);

        let report = result.format_report();
        assert!(report.contains("AAPL"));
        assert!(report.contains("Technical Analysis"));
        assert!(report.contains("RSI: 55"));
    }
}
