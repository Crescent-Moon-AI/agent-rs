//! News and sentiment analysis agent

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::CacheManager;
use crate::config::StockConfig;
use crate::tools::NewsTool;

/// Agent specialized in news and sentiment analysis
pub struct NewsAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
}

impl NewsAnalyzerAgent {
    /// Create a new news analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        let cache_mgr = CacheManager::new(
            config.cache_ttl_realtime,
            config.cache_ttl_fundamental,
            config.cache_ttl_news,
        );

        // Create tools
        let news_tool = Arc::new(NewsTool::new(Arc::clone(&config), cache_mgr.news.clone()));

        // Register tools
        runtime.tools().register(news_tool);

        // Get system prompt from registry
        let system_prompt = config
            .prompt_registry
            .render("stock.news_analyzer", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;

        let executor_config = ExecutorConfig {
            model: config.model.clone(),
            system_prompt: Some(system_prompt),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            max_iterations: 5,
        };

        let agent = runtime.create_tool_agent(executor_config, "news-analyzer");

        Ok(Self { agent })
    }
}

#[async_trait]
impl Agent for NewsAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "NewsAnalyzerAgent"
    }
}
