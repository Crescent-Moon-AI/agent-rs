//! Stock analysis prompt templates
//!
//! This module contains all prompt templates used by stock analysis agents.
//! Templates are organized into:
//! - `system`: System prompts for each agent type
//! - `user`: User message templates for specific operations

mod system;
mod user;

pub use system::*;
pub use user::*;

use agent_prompt::{PromptRegistry, Result};

/// Register all stock analysis prompts with the given registry
///
/// This function registers all system prompts and user message templates
/// for stock analysis agents.
///
/// # Arguments
///
/// * `registry` - The prompt registry to register templates with
///
/// # Returns
///
/// Returns `Ok(())` if all prompts were registered successfully,
/// or an error if any prompt failed to register.
///
/// # Example
///
/// ```ignore
/// use agent_prompt::{Language, PromptRegistry};
/// use agent_stock::prompts::register_prompts;
///
/// let registry = PromptRegistry::with_language(Language::Chinese);
/// register_prompts(&registry).expect("Failed to register prompts");
/// ```
pub fn register_prompts(registry: &PromptRegistry) -> Result<()> {
    // System prompts for agents
    registry.register(technical_analyzer()?);
    registry.register(fundamental_analyzer()?);
    registry.register(news_analyzer()?);
    registry.register(earnings_analyzer()?);
    registry.register(macro_analyzer()?);
    registry.register(data_fetcher()?);

    // User message templates - Earnings
    registry.register(analyze_earnings_prompt()?);
    registry.register(compare_earnings_prompt()?);
    registry.register(analyze_quality_prompt()?);

    // User message templates - Macro
    registry.register(analyze_economy_prompt()?);
    registry.register(analyze_fed_policy_prompt()?);
    registry.register(analyze_rates_prompt()?);
    registry.register(analyze_inflation_prompt()?);
    registry.register(analyze_geopolitical_risks_prompt()?);
    registry.register(get_market_outlook_prompt()?);
    registry.register(analyze_impact_prompt()?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_prompt::Language;

    #[test]
    fn test_register_all_prompts() {
        let registry = PromptRegistry::with_language(Language::English);
        let result = register_prompts(&registry);
        assert!(result.is_ok());

        // Verify system prompts are registered
        assert!(registry.get("stock.technical_analyzer").is_some());
        assert!(registry.get("stock.fundamental_analyzer").is_some());
        assert!(registry.get("stock.news_analyzer").is_some());
        assert!(registry.get("stock.earnings_analyzer").is_some());
        assert!(registry.get("stock.macro_analyzer").is_some());
        assert!(registry.get("stock.data_fetcher").is_some());

        // Verify user prompts are registered
        assert!(registry.get("stock.user.analyze_earnings").is_some());
        assert!(registry.get("stock.user.compare_earnings").is_some());
        assert!(registry.get("stock.user.analyze_quality").is_some());
        assert!(registry.get("stock.user.analyze_economy").is_some());
        assert!(registry.get("stock.user.analyze_fed_policy").is_some());
        assert!(registry.get("stock.user.analyze_rates").is_some());
        assert!(registry.get("stock.user.analyze_inflation").is_some());
        assert!(registry.get("stock.user.analyze_geopolitical_risks").is_some());
        assert!(registry.get("stock.user.get_market_outlook").is_some());
        assert!(registry.get("stock.user.analyze_impact").is_some());
    }

    #[test]
    fn test_render_system_prompt_from_registry() {
        let registry = PromptRegistry::with_language(Language::Chinese);
        register_prompts(&registry).unwrap();

        let prompt = registry
            .render("stock.technical_analyzer", &serde_json::json!({}))
            .unwrap();
        assert!(prompt.contains("技术分析"));
    }

    #[test]
    fn test_render_user_prompt_from_registry() {
        let registry = PromptRegistry::with_language(Language::English);
        register_prompts(&registry).unwrap();

        let prompt = registry
            .render("stock.user.analyze_earnings", &serde_json::json!({ "symbol": "GOOGL" }))
            .unwrap();
        assert!(prompt.contains("GOOGL"));
        assert!(prompt.contains("financial reports"));
    }
}
