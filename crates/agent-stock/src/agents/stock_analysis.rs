//! Top-level stock analysis agent that delegates to specialists

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, agents::DelegatingAgentBuilder};
use async_trait::async_trait;
use std::sync::Arc;

use crate::config::StockConfig;
use super::{DataFetcherAgent, TechnicalAnalyzerAgent, FundamentalAnalyzerAgent, NewsAnalyzerAgent};

const SYSTEM_PROMPT: &str = r#"You are a comprehensive stock analysis assistant with access to specialized sub-agents.

You have four expert sub-agents at your disposal:
1. **DataFetcherAgent**: Retrieves current prices, quotes, and historical data
2. **TechnicalAnalyzerAgent**: Performs technical analysis with indicators (RSI, MACD, etc.)
3. **FundamentalAnalyzerAgent**: Analyzes company fundamentals (P/E, market cap, financials)
4. **NewsAnalyzerAgent**: Analyzes recent news and market sentiment

**How to route requests:**

- For simple price quotes or historical data → DataFetcherAgent
- For technical analysis, chart patterns, indicators → TechnicalAnalyzerAgent
- For valuation, financial metrics, company fundamentals → FundamentalAnalyzerAgent
- For news, sentiment, recent events → NewsAnalyzerAgent
- For comprehensive analysis → Consult ALL relevant agents and synthesize

**When providing comprehensive analysis:**
1. Start with current price and company basics (DataFetcher)
2. Analyze technical indicators and trends (TechnicalAnalyzer)
3. Evaluate fundamentals and valuation (FundamentalAnalyzer)
4. Check recent news and sentiment (NewsAnalyzer)
5. Synthesize all perspectives into a coherent recommendation

Always be clear about which analysis comes from which perspective.
Acknowledge when different analyses give conflicting signals.
Provide balanced, objective assessments.
"#;

/// Top-level stock analysis agent that delegates to specialists
pub struct StockAnalysisAgent {
    agent: agent_runtime::agents::DelegatingAgent,
}

impl StockAnalysisAgent {
    /// Create a new stock analysis agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create specialist agents
        let data_fetcher = Arc::new(DataFetcherAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let technical_analyzer = Arc::new(TechnicalAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let fundamental_analyzer = Arc::new(FundamentalAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);
        let news_analyzer = Arc::new(NewsAnalyzerAgent::new(Arc::clone(&runtime), Arc::clone(&config)).await?);

        // Create routing function
        let router = |input: &str, _context: &Context| -> String {
            let input_lower = input.to_lowercase();

            // Check for specific keywords to route to appropriate agent
            if input_lower.contains("price") || input_lower.contains("quote") || input_lower.contains("historical") {
                "data-fetcher".to_string()
            } else if input_lower.contains("rsi") || input_lower.contains("macd") || input_lower.contains("technical")
                || input_lower.contains("indicator") || input_lower.contains("chart") || input_lower.contains("bollinger")
                || input_lower.contains("moving average") || input_lower.contains("sma") || input_lower.contains("ema") {
                "technical-analyzer".to_string()
            } else if input_lower.contains("p/e") || input_lower.contains("fundamental") || input_lower.contains("valuation")
                || input_lower.contains("earnings") || input_lower.contains("market cap") || input_lower.contains("dividend") {
                "fundamental-analyzer".to_string()
            } else if input_lower.contains("news") || input_lower.contains("sentiment") || input_lower.contains("events") {
                "news-analyzer".to_string()
            } else {
                // For comprehensive analysis or unclear requests, use technical analyzer as default
                // The LLM will handle coordination
                "technical-analyzer".to_string()
            }
        };

        // Build delegating agent
        let agent = DelegatingAgentBuilder::new(Arc::clone(&runtime), "stock-analysis")
            .add_agent("data-fetcher", data_fetcher)
            .add_agent("technical-analyzer", technical_analyzer)
            .add_agent("fundamental-analyzer", fundamental_analyzer)
            .add_agent("news-analyzer", news_analyzer)
            .router(router)
            .build()?;

        Ok(Self { agent })
    }

    /// Analyze a stock symbol with comprehensive analysis
    pub async fn analyze(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!(
            "Provide a comprehensive analysis of {} including current price, \
             technical indicators, fundamental metrics, and recent news.",
            symbol
        );
        self.process(input, &mut context).await
    }

    /// Get technical analysis only
    pub async fn analyze_technical(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!("Perform technical analysis on {} using RSI, MACD, and moving averages.", symbol);
        self.process(input, &mut context).await
    }

    /// Get fundamental analysis only
    pub async fn analyze_fundamental(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!("Analyze the fundamental metrics and valuation of {}.", symbol);
        self.process(input, &mut context).await
    }

    /// Get news and sentiment analysis only
    pub async fn analyze_news(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!("Analyze recent news and market sentiment for {}.", symbol);
        self.process(input, &mut context).await
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
    fn test_routing_logic() {
        let router = |input: &str, _context: &Context| -> String {
            let input_lower = input.to_lowercase();
            if input_lower.contains("price") {
                "data-fetcher".to_string()
            } else if input_lower.contains("rsi") {
                "technical-analyzer".to_string()
            } else if input_lower.contains("p/e") {
                "fundamental-analyzer".to_string()
            } else if input_lower.contains("news") {
                "news-analyzer".to_string()
            } else {
                "technical-analyzer".to_string()
            }
        };

        let ctx = Context::new();

        assert_eq!(router("What's the price of AAPL?", &ctx), "data-fetcher");
        assert_eq!(router("Calculate RSI for GOOGL", &ctx), "technical-analyzer");
        assert_eq!(router("What's the P/E ratio of MSFT?", &ctx), "fundamental-analyzer");
        assert_eq!(router("Latest news on TSLA", &ctx), "news-analyzer");
    }
}
