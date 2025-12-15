//! Data fetching agent for stock information

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::StockConfig;
use crate::tools::{FundamentalDataTool, StockDataTool};

const SYSTEM_PROMPT: &str = r#"You are a data fetching specialist for stock market information.

Your job is to efficiently retrieve stock market data including:
- Current prices and quotes
- Historical price data
- Company information and fundamental data

When asked about a stock:
1. Always validate the stock symbol first
2. Fetch the most relevant data for the user's query
3. Present data clearly and concisely
4. Handle errors gracefully and suggest alternatives if a symbol is invalid

Be precise with numbers and always include timestamps when providing data.
"#;

/// Agent specialized in fetching stock data
pub struct DataFetcherAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl DataFetcherAgent {
    /// Create a new data fetcher agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create cache manager
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
        let fundamental_tool = Arc::new(FundamentalDataTool::new(
            Arc::clone(&config),
            cache_mgr.fundamental.clone(),
        ));

        // Register tools
        runtime.tools().register(stock_data_tool);
        runtime.tools().register(fundamental_tool);

        // Create executor config
        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(SYSTEM_PROMPT.to_string()),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 5,
        };

        // Create tool agent
        let agent = runtime.create_tool_agent(executor_config, "data-fetcher");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for DataFetcherAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "DataFetcherAgent"
    }
}
