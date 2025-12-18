//! Agent specialized in analyzing company earnings and financial reports

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::StockCache;
use crate::config::StockConfig;
use crate::tools::EarningsReportTool;

/// Agent specialized in analyzing company earnings reports
pub struct EarningsAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
    config: Arc<StockConfig>,
}

impl EarningsAnalyzerAgent {
    /// Create a new earnings analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create cache for earnings data (24h TTL)
        let cache = StockCache::new(config.cache_ttl_earnings);

        // Register earnings report tool
        let earnings_tool = Arc::new(EarningsReportTool::new(Arc::clone(&config), cache));
        runtime.tools().register(earnings_tool);

        // Get system prompt from registry
        let system_prompt = config
            .prompt_registry
            .render("stock.earnings_analyzer", &serde_json::json!({}))
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
        let agent = runtime.create_tool_agent(executor_config, "earnings-analyzer");

        Ok(Self { agent, config })
    }

    /// Analyze earnings for a specific symbol
    pub async fn analyze_earnings(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render(
                "stock.user.analyze_earnings",
                &serde_json::json!({ "symbol": symbol }),
            )
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Compare earnings across multiple periods
    pub async fn compare_earnings(&self, symbol: &str, periods: usize) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render(
                "stock.user.compare_earnings",
                &serde_json::json!({ "symbol": symbol, "periods": periods }),
            )
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze earnings quality
    pub async fn analyze_quality(&self, symbol: &str) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render(
                "stock.user.analyze_quality",
                &serde_json::json!({ "symbol": symbol }),
            )
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }
}

#[async_trait]
impl Agent for EarningsAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "EarningsAnalyzerAgent"
    }
}

#[cfg(test)]
mod tests {
    use agent_prompt::{Language, PromptRegistry};
    use crate::prompts::register_prompts;

    #[test]
    fn test_prompts_registered() {
        let registry = PromptRegistry::with_language(Language::English);
        register_prompts(&registry).unwrap();

        // Verify system prompt exists
        assert!(registry.get("stock.earnings_analyzer").is_some());

        // Verify user prompts exist
        assert!(registry.get("stock.user.analyze_earnings").is_some());
        assert!(registry.get("stock.user.compare_earnings").is_some());
        assert!(registry.get("stock.user.analyze_quality").is_some());
    }
}
