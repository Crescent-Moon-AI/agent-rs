//! Agent executor for running agent loops
//!
//! The AgentExecutor implements the core agent loop pattern:
//! 1. Call LLM with conversation history and available tools
//! 2. Check stop reason
//! 3. If tool use requested, execute tools and loop back
//! 4. If completed, return final response

use agent_core::Result;
use agent_llm::{
    CompletionRequest, ContentBlock, LLMProvider, Message, StopReason, ToolDefinition,
};
use agent_tools::ToolRegistry;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Configuration for agent execution
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of iterations (prevents infinite loops)
    pub max_iterations: usize,

    /// Model to use
    pub model: String,

    /// System prompt
    pub system_prompt: Option<String>,

    /// Max tokens per completion
    pub max_tokens: usize,

    /// Temperature
    pub temperature: Option<f32>,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            model: "claude-sonnet-4-5-20250929".to_string(),
            system_prompt: None,
            max_tokens: 4096,
            temperature: Some(0.7),
        }
    }
}

/// Executes an agent loop: LLM → tool calls → execution → loop back
///
/// The AgentExecutor orchestrates the interaction between an LLM provider
/// and a tool registry, implementing the agent loop pattern.
pub struct AgentExecutor {
    provider: Arc<dyn LLMProvider>,
    tool_registry: Arc<ToolRegistry>,
    config: ExecutorConfig,
}

impl AgentExecutor {
    /// Create a new agent executor
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        tool_registry: Arc<ToolRegistry>,
        config: ExecutorConfig,
    ) -> Self {
        Self {
            provider,
            tool_registry,
            config,
        }
    }

    /// Execute the agent loop with a user query
    ///
    /// # Arguments
    ///
    /// * `user_message` - The user's input message
    ///
    /// # Returns
    ///
    /// The final response from the agent after all tool calls are complete
    pub async fn run(&self, user_message: String) -> Result<String> {
        let mut conversation = vec![Message::user(user_message)];
        let mut iteration = 0;

        loop {
            iteration += 1;
            if iteration > self.config.max_iterations {
                warn!(
                    "Max iterations ({}) reached, stopping",
                    self.config.max_iterations
                );
                return Ok("Max iterations reached without completion".to_string());
            }

            info!(
                "Agent iteration {}/{}",
                iteration, self.config.max_iterations
            );

            // Build tool definitions from registry
            let tools = self.build_tool_definitions();
            debug!("Available tools: {}", tools.len());

            // Call LLM
            let mut request_builder = CompletionRequest::builder(&self.config.model)
                .messages(conversation.clone())
                .system(
                    self.config
                        .system_prompt
                        .clone()
                        .unwrap_or_else(|| "You are a helpful assistant.".to_string()),
                )
                .max_tokens(self.config.max_tokens)
                .temperature(self.config.temperature.unwrap_or(0.7));

            // Only add tools if we have any
            if !tools.is_empty() {
                request_builder = request_builder.tools(tools);
            }

            let request = request_builder.build();

            let response = self
                .provider
                .complete(request)
                .await
                .map_err(|e| agent_core::Error::ProcessingFailed(e.to_string()))?;

            debug!(
                "LLM response - stop_reason: {:?}, tokens: {:?}",
                response.stop_reason, response.usage
            );

            // Add assistant message to conversation
            conversation.push(response.message.clone());

            // Check stop reason
            match response.stop_reason {
                StopReason::EndTurn => {
                    // Natural completion, extract text and return
                    debug!("Agent completed naturally");
                    let text = response
                        .message
                        .text()
                        .unwrap_or("No response")
                        .to_string();
                    return Ok(text);
                }

                StopReason::ToolUse => {
                    // Extract and execute tool calls
                    debug!("Agent requested tool use");
                    let tool_results = self.execute_tools(&response.message).await?;

                    if tool_results.is_empty() {
                        warn!("No tool results despite ToolUse stop reason");
                        return Ok("Tool execution failed".to_string());
                    }

                    // Add tool results to conversation
                    for result in tool_results {
                        conversation.push(result);
                    }

                    // Continue loop
                    continue;
                }

                StopReason::MaxTokens => {
                    warn!("Hit max tokens in LLM response");
                    return Ok("Response truncated due to token limit".to_string());
                }

                StopReason::StopSequence => {
                    debug!("Stop sequence encountered");
                    let text = response
                        .message
                        .text()
                        .unwrap_or("No response")
                        .to_string();
                    return Ok(text);
                }
            }
        }
    }

    /// Build tool definitions from the registry
    fn build_tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tool_registry
            .list_tools()
            .iter()
            .map(|tool| {
                ToolDefinition::new(tool.name(), tool.description(), tool.input_schema())
            })
            .collect()
    }

    /// Execute tool calls from an assistant message
    async fn execute_tools(&self, message: &Message) -> Result<Vec<Message>> {
        let mut results = Vec::new();

        // Extract tool uses
        let tool_uses = message.tool_uses();
        debug!("Executing {} tool(s)", tool_uses.len());

        for tool_use in tool_uses {
            if let ContentBlock::ToolUse { id, name, input } = tool_use {
                info!("Executing tool: {}", name);

                // Get tool from registry
                let tool = self.tool_registry.get(name).ok_or_else(|| {
                    agent_core::Error::ProcessingFailed(format!("Tool not found: {}", name))
                })?;

                // Execute tool
                match tool.execute(input.clone()).await {
                    Ok(result) => {
                        debug!("Tool {} succeeded", name);
                        // Convert result to string
                        let result_str = serde_json::to_string(&result)
                            .unwrap_or_else(|_| result.to_string());

                        results.push(Message::tool_result(id.clone(), result_str));
                    }
                    Err(e) => {
                        warn!("Tool {} execution failed: {}", name, e);
                        // Return error as tool result
                        results.push(Message::tool_error(id.clone(), format!("Error: {}", e)));
                    }
                }
            }
        }

        Ok(results)
    }
}

/// Builder for AgentExecutor
pub struct AgentExecutorBuilder {
    provider: Option<Arc<dyn LLMProvider>>,
    tool_registry: Arc<ToolRegistry>,
    config: ExecutorConfig,
}

impl AgentExecutorBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            provider: None,
            tool_registry: Arc::new(ToolRegistry::new()),
            config: ExecutorConfig::default(),
        }
    }

    /// Set the LLM provider
    pub fn provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the tool registry
    pub fn tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = registry;
        self
    }

    /// Set the full configuration
    pub fn config(mut self, config: ExecutorConfig) -> Self {
        self.config = config;
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the system prompt
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Set max tokens
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = Some(temperature);
        self
    }

    /// Build the executor
    pub fn build(self) -> Result<AgentExecutor> {
        let provider = self.provider.ok_or_else(|| {
            agent_core::Error::InitializationFailed("Provider not set".to_string())
        })?;

        Ok(AgentExecutor::new(provider, self.tool_registry, self.config))
    }
}

impl Default for AgentExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let builder = AgentExecutorBuilder::new()
            .model("test-model")
            .max_iterations(5)
            .system_prompt("Test prompt");

        assert_eq!(builder.config.model, "test-model");
        assert_eq!(builder.config.max_iterations, 5);
        assert_eq!(builder.config.system_prompt, Some("Test prompt".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    }
}
