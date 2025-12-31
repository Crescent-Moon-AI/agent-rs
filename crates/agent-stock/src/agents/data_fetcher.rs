//! Data fetching agent for stock information

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::StockConfig;
use crate::tools::{FundamentalDataTool, StockDataTool};

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

        // Get system prompt from registry
        let system_prompt = config
            .prompt_registry
            .render("stock.data_fetcher", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;

        // Create executor config
        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(system_prompt),
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

    fn name(&self) -> &'static str {
        "DataFetcherAgent"
    }
}
