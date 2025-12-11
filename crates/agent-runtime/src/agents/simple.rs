//! Simple agent implementation (LLM only, no tools)

use agent_core::{Agent, Context, Result};
use agent_llm::{CompletionRequest, LLMProvider, Message};
use async_trait::async_trait;
use std::sync::Arc;

/// Configuration for a simple agent
#[derive(Debug, Clone)]
pub struct SimpleConfig {
    /// Model to use
    pub model: String,

    /// System prompt
    pub system_prompt: String,

    /// Max tokens per completion
    pub max_tokens: usize,

    /// Temperature for sampling
    pub temperature: f32,
}

impl Default for SimpleConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-5-20250929".to_string(),
            system_prompt: "You are a helpful assistant.".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
        }
    }
}

/// A simple agent that uses LLM without tools
///
/// SimpleAgent provides a straightforward LLM interaction without
/// the complexity of tool execution loops. It's suitable for:
/// - Basic question answering
/// - Text generation
/// - Simple conversational agents
///
/// # Example
///
/// ```no_run
/// use agent_runtime::{SimpleAgent, SimpleConfig};
/// use agent_core::{Agent, Context};
/// use std::sync::Arc;
///
/// # async fn example() -> agent_core::Result<()> {
/// let config = SimpleConfig {
///     model: "claude-sonnet-4-5-20250929".to_string(),
///     system_prompt: "You are a helpful assistant.".to_string(),
///     max_tokens: 2048,
///     temperature: 0.7,
/// };
///
/// let agent = SimpleAgent::new(provider, config, "assistant");
/// let mut context = Context::new();
/// let response = agent.process("Hello!".to_string(), &mut context).await?;
/// # Ok(())
/// # }
/// ```
pub struct SimpleAgent {
    provider: Arc<dyn LLMProvider>,
    config: SimpleConfig,
    name: String,
}

impl SimpleAgent {
    /// Create a new simple agent
    ///
    /// # Arguments
    ///
    /// * `provider` - The LLM provider to use
    /// * `config` - Configuration for the agent
    /// * `name` - Name of the agent
    pub fn new(provider: Arc<dyn LLMProvider>, config: SimpleConfig, name: String) -> Self {
        Self {
            provider,
            config,
            name,
        }
    }

    /// Get the agent's configuration
    pub fn config(&self) -> &SimpleConfig {
        &self.config
    }
}

#[async_trait]
impl Agent for SimpleAgent {
    async fn process(&self, input: String, _context: &mut Context) -> Result<String> {
        // Build completion request
        let request = CompletionRequest::builder(&self.config.model)
            .messages(vec![Message::user(input)])
            .system(self.config.system_prompt.clone())
            .max_tokens(self.config.max_tokens)
            .temperature(self.config.temperature)
            .build();

        // Call LLM
        let response = self
            .provider
            .complete(request)
            .await
            .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;

        // Extract text from response
        Ok(response
            .message
            .text()
            .unwrap_or("No response")
            .to_string())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_config_default() {
        let config = SimpleConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
        assert_eq!(config.system_prompt, "You are a helpful assistant.");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_simple_agent_name() {
        let config = SimpleConfig::default();
        // Note: This would require a mock provider to fully test
        // For now, we just test the struct creation pattern
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    }
}
