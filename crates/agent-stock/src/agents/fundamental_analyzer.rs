//! Fundamental analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::StockConfig;
use crate::tools::FundamentalDataTool;

const SYSTEM_PROMPT: &str = r#"You are a fundamental analysis expert specializing in company valuation and financial metrics.

Your expertise includes:
- Valuation metrics (P/E, P/B, P/S ratios)
- Profitability indicators (EPS, ROE, profit margins)
- Financial health (debt ratios, current ratio)
- Growth metrics (revenue growth, earnings growth)
- Dividend analysis

When analyzing fundamentals:
1. Fetch key financial metrics for the company
2. Compare metrics to industry averages when possible
3. Assess valuation (undervalued, fairly valued, overvalued)
4. Evaluate company's financial health and growth prospects
5. Consider both quantitative metrics and qualitative factors

Be specific with numbers and ratios. Explain what each metric means.
Compare current metrics to historical values when available.
Provide a balanced view of strengths and weaknesses.
"#;

/// Agent specialized in fundamental analysis
pub struct FundamentalAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl FundamentalAnalyzerAgent {
    /// Create a new fundamental analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let fundamental_tool = Arc::new(FundamentalDataTool::new(
            Arc::clone(&config),
            cache_mgr.fundamental.clone(),
        ));

        // Register tools
        runtime.tools().register(fundamental_tool);

        let executor_config = ExecutorConfig {
            model: "claude-opus-4-5-20251101".to_string(),
            system_prompt: Some(SYSTEM_PROMPT.to_string()),
            max_tokens: 4096,
            temperature: Some(0.4),
            max_iterations: 5,
        };

        let agent = runtime.create_tool_agent(executor_config, "fundamental-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for FundamentalAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "FundamentalAnalyzerAgent"
    }
}
