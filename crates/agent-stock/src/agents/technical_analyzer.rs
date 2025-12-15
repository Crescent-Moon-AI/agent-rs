//! Technical analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::StockConfig;
use crate::tools::{ChartDataTool, StockDataTool, TechnicalIndicatorTool};

const SYSTEM_PROMPT: &str = r#"You are a technical analysis expert specializing in stock market analysis.

Your expertise includes:
- Technical indicators (RSI, MACD, Moving Averages, Bollinger Bands, etc.)
- Chart patterns and trend analysis
- Support and resistance levels
- Volume analysis and momentum indicators

When analyzing a stock technically:
1. Calculate relevant technical indicators
2. Interpret the indicators in context (overbought/oversold, bullish/bearish)
3. Look for divergences and confirmations across multiple indicators
4. Provide clear buy/sell/hold signals with reasoning
5. Consider multiple timeframes when relevant

Be specific with indicator values and thresholds. Explain your analysis clearly.
Always acknowledge that technical analysis is probabilistic, not deterministic.
"#;

/// Agent specialized in technical analysis
pub struct TechnicalAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl TechnicalAnalyzerAgent {
    /// Create a new technical analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let stock_data_tool = Arc::new(StockDataTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));
        let technical_tool = Arc::new(TechnicalIndicatorTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));
        let chart_tool = Arc::new(ChartDataTool::new(
            Arc::clone(&config),
            cache_mgr.realtime.clone(),
        ));

        // Register tools
        runtime.tools().register(stock_data_tool);
        runtime.tools().register(technical_tool);
        runtime.tools().register(chart_tool);

        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(SYSTEM_PROMPT.to_string()),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 10, // More iterations for comprehensive analysis
        };

        let agent = runtime.create_tool_agent(executor_config, "technical-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for TechnicalAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "TechnicalAnalyzerAgent"
    }
}
