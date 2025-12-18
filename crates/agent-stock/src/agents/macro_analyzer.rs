//! Agent specialized in macroeconomic analysis and Fed policy interpretation

use agent_core::{Agent, Context, Result};
use agent_runtime::{AgentRuntime, ExecutorConfig};
use async_trait::async_trait;
use std::sync::Arc;

use crate::cache::StockCache;
use crate::config::StockConfig;
use crate::tools::{GeopoliticalTool, MacroEconomicTool};

/// Agent specialized in macroeconomic analysis
pub struct MacroAnalyzerAgent {
    agent: agent_runtime::agents::ToolAgent,
    config: Arc<StockConfig>,
}

impl MacroAnalyzerAgent {
    /// Create a new macro analyzer agent
    pub async fn new(runtime: Arc<AgentRuntime>, config: Arc<StockConfig>) -> Result<Self> {
        // Create caches
        let macro_cache = StockCache::new(config.cache_ttl_macro);
        let geopolitical_cache = StockCache::new(config.cache_ttl_news);

        // Register macro economic tool
        let macro_tool = Arc::new(MacroEconomicTool::new(Arc::clone(&config), macro_cache));
        runtime.tools().register(macro_tool);

        // Register geopolitical tool
        let geo_tool = Arc::new(GeopoliticalTool::new(Arc::clone(&config), geopolitical_cache));
        runtime.tools().register(geo_tool);

        // Get system prompt from registry
        let system_prompt = config
            .prompt_registry
            .render("stock.macro_analyzer", &serde_json::json!({}))
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
        let agent = runtime.create_tool_agent(executor_config, "macro-analyzer");

        Ok(Self { agent, config })
    }

    /// Get comprehensive economic overview
    pub async fn analyze_economy(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render("stock.user.analyze_economy", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze Federal Reserve policy
    pub async fn analyze_fed_policy(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render("stock.user.analyze_fed_policy", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze interest rate environment
    pub async fn analyze_rates(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render("stock.user.analyze_rates", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze inflation trends
    pub async fn analyze_inflation(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render("stock.user.analyze_inflation", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze geopolitical risks
    pub async fn analyze_geopolitical_risks(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render(
                "stock.user.analyze_geopolitical_risks",
                &serde_json::json!({}),
            )
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Get market outlook based on macro conditions
    pub async fn get_market_outlook(&self) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render("stock.user.get_market_outlook", &serde_json::json!({}))
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }

    /// Analyze impact on a specific stock/sector
    pub async fn analyze_impact(&self, subject: &str) -> Result<String> {
        let mut context = Context::new();
        let input = self
            .config
            .prompt_registry
            .render(
                "stock.user.analyze_impact",
                &serde_json::json!({ "subject": subject }),
            )
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;
        self.process(input, &mut context).await
    }
}

#[async_trait]
impl Agent for MacroAnalyzerAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        self.agent.process(input, context).await
    }

    fn name(&self) -> &str {
        "MacroAnalyzerAgent"
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
        assert!(registry.get("stock.macro_analyzer").is_some());

        // Verify user prompts exist
        assert!(registry.get("stock.user.analyze_economy").is_some());
        assert!(registry.get("stock.user.analyze_fed_policy").is_some());
        assert!(registry.get("stock.user.analyze_rates").is_some());
        assert!(registry.get("stock.user.analyze_inflation").is_some());
        assert!(registry.get("stock.user.analyze_geopolitical_risks").is_some());
        assert!(registry.get("stock.user.get_market_outlook").is_some());
        assert!(registry.get("stock.user.analyze_impact").is_some());
    }
}
