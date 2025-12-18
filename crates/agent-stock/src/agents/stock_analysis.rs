//! Top-level stock analysis agent that delegates to specialists

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, agents::DelegatingAgentBuilder};
use async_trait::async_trait;
use std::sync::Arc;

use super::{
    DataFetcherAgent, EarningsAnalyzerAgent, FundamentalAnalyzerAgent, 
    MacroAnalyzerAgent, NewsAnalyzerAgent, TechnicalAnalyzerAgent,
};
use crate::config::StockConfig;

const _SYSTEM_PROMPT: &str = r#"You are a comprehensive stock analysis assistant with access to specialized sub-agents.

You have six expert sub-agents at your disposal:
1. **DataFetcherAgent**: Retrieves current prices, quotes, and historical data
2. **TechnicalAnalyzerAgent**: Performs technical analysis with indicators (RSI, MACD, etc.)
3. **FundamentalAnalyzerAgent**: Analyzes company fundamentals (P/E, market cap, financials)
4. **NewsAnalyzerAgent**: Analyzes recent news and market sentiment
5. **EarningsAnalyzerAgent**: Analyzes quarterly/annual earnings reports and financial statements
6. **MacroAnalyzerAgent**: Analyzes macroeconomic conditions, Fed policy, and geopolitical risks

**How to route requests:**

- For simple price quotes or historical data → DataFetcherAgent
- For technical analysis, chart patterns, indicators → TechnicalAnalyzerAgent
- For valuation, financial metrics, company fundamentals → FundamentalAnalyzerAgent
- For news, sentiment, recent events → NewsAnalyzerAgent
- For earnings reports, 10-K/10-Q analysis, financial statements → EarningsAnalyzerAgent
- For macro economics, Fed policy, interest rates, geopolitics → MacroAnalyzerAgent
- For comprehensive analysis → Consult ALL relevant agents and synthesize

**When providing comprehensive analysis:**
1. Start with current price and company basics (DataFetcher)
2. Analyze technical indicators and trends (TechnicalAnalyzer)
3. Evaluate fundamentals and valuation (FundamentalAnalyzer)
4. Check recent news and sentiment (NewsAnalyzer)
5. Review earnings trends and financial health (EarningsAnalyzer)
6. Consider macro environment and risks (MacroAnalyzer)
7. Synthesize all perspectives into a coherent recommendation

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

        // Create routing function
        let router = |input: &str, _context: &Context| -> String {
            let input_lower = input.to_lowercase();

            // Check for specific keywords to route to appropriate agent
            // Earnings/Financial reports
            if input_lower.contains("earnings")
                || input_lower.contains("财报")
                || input_lower.contains("10-k")
                || input_lower.contains("10-q")
                || input_lower.contains("季报")
                || input_lower.contains("年报")
                || input_lower.contains("financial report")
                || input_lower.contains("财务报告")
            {
                return "earnings-analyzer".to_string();
            }
            
            // Macro/Fed/Geopolitics
            if input_lower.contains("fed")
                || input_lower.contains("美联储")
                || input_lower.contains("interest rate")
                || input_lower.contains("利率")
                || input_lower.contains("inflation")
                || input_lower.contains("通胀")
                || input_lower.contains("gdp")
                || input_lower.contains("unemployment")
                || input_lower.contains("失业")
                || input_lower.contains("macro")
                || input_lower.contains("宏观")
                || input_lower.contains("geopolitical")
                || input_lower.contains("地缘政治")
                || input_lower.contains("trade war")
                || input_lower.contains("贸易战")
                || input_lower.contains("国际形势")
            {
                return "macro-analyzer".to_string();
            }
            
            // Price/Quote data
            if input_lower.contains("price")
                || input_lower.contains("quote")
                || input_lower.contains("historical")
            {
                return "data-fetcher".to_string();
            }
            
            // Technical analysis
            if input_lower.contains("rsi")
                || input_lower.contains("macd")
                || input_lower.contains("technical")
                || input_lower.contains("indicator")
                || input_lower.contains("chart")
                || input_lower.contains("bollinger")
                || input_lower.contains("moving average")
                || input_lower.contains("sma")
                || input_lower.contains("ema")
            {
                return "technical-analyzer".to_string();
            }
            
            // Fundamental analysis
            if input_lower.contains("p/e")
                || input_lower.contains("fundamental")
                || input_lower.contains("valuation")
                || input_lower.contains("market cap")
                || input_lower.contains("dividend")
            {
                return "fundamental-analyzer".to_string();
            }
            
            // News/Sentiment
            if input_lower.contains("news")
                || input_lower.contains("sentiment")
                || input_lower.contains("events")
            {
                return "news-analyzer".to_string();
            }
            
            // For comprehensive analysis or unclear requests, use technical analyzer as default
            "technical-analyzer".to_string()
        };

        // Build delegating agent with all sub-agents
        let agent = DelegatingAgentBuilder::new(Arc::clone(&runtime), "stock-analysis")
            .add_agent("data-fetcher", data_fetcher)
            .add_agent("technical-analyzer", technical_analyzer)
            .add_agent("fundamental-analyzer", fundamental_analyzer)
            .add_agent("news-analyzer", news_analyzer)
            .add_agent("earnings-analyzer", earnings_analyzer)
            .add_agent("macro-analyzer", macro_analyzer)
            .router(router)
            .build()?;

        Ok(Self { agent })
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
        let mut context = Context::new();
        let input = format!(
            "Perform technical analysis on {} using RSI, MACD, and moving averages.",
            symbol
        );
        self.process(input, &mut context).await
    }

    /// Get fundamental analysis only
    pub async fn analyze_fundamental(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!(
            "Analyze the fundamental metrics and valuation of {}.",
            symbol
        );
        self.process(input, &mut context).await
    }

    /// Get news and sentiment analysis only
    pub async fn analyze_news(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!("Analyze recent news and market sentiment for {}.", symbol);
        self.process(input, &mut context).await
    }

    /// Get earnings analysis
    pub async fn analyze_earnings(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!(
            "Analyze the earnings reports and financial statements for {}.",
            symbol
        );
        self.process(input, &mut context).await
    }

    /// Get macro economic analysis
    pub async fn analyze_macro(&self) -> Result<String> {
        let mut context = Context::new();
        let input = "Analyze the current macroeconomic environment, including Fed policy, inflation, and economic indicators.".to_string();
        self.process(input, &mut context).await
    }

    /// Get geopolitical analysis
    pub async fn analyze_geopolitical(&self) -> Result<String> {
        let mut context = Context::new();
        let input = "Analyze current geopolitical risks and their potential market impact.".to_string();
        self.process(input, &mut context).await
    }

    /// Get comprehensive analysis including macro factors
    pub async fn analyze_comprehensive(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = format!(
            "Provide a comprehensive investment analysis of {} including: \
             1) Current price and technical indicators, \
             2) Fundamental metrics and valuation, \
             3) Recent earnings performance, \
             4) News and sentiment, \
             5) Macroeconomic factors and risks. \
             Synthesize all factors into an investment recommendation.",
            symbol
        );
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
        assert_eq!(
            router("Calculate RSI for GOOGL", &ctx),
            "technical-analyzer"
        );
        assert_eq!(
            router("What's the P/E ratio of MSFT?", &ctx),
            "fundamental-analyzer"
        );
        assert_eq!(router("Latest news on TSLA", &ctx), "news-analyzer");
    }
}
