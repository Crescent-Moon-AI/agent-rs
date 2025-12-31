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
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Event handler for agent execution events
///
/// Implement this trait to receive callbacks during agent execution,
/// useful for streaming tool call status to clients.
#[async_trait]
pub trait ExecutorEventHandler: Send + Sync {
    /// Called when a tool execution starts
    async fn on_tool_start(&self, _id: &str, _name: &str, _input: &Value) {}

    /// Called when a tool execution completes
    async fn on_tool_done(
        &self,
        _id: &str,
        _name: &str,
        _result: std::result::Result<&Value, &str>,
        _duration_ms: u64,
    ) {
    }

    /// Called when the agent completes
    async fn on_complete(&self, _result: &str) {}

    /// Called when an error occurs
    async fn on_error(&self, _error: &str) {}
}

/// No-op event handler for when events are not needed
pub struct NoOpEventHandler;

#[async_trait]
impl ExecutorEventHandler for NoOpEventHandler {}

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
    event_handler: Option<Arc<dyn ExecutorEventHandler>>,
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
            event_handler: None,
        }
    }

    /// Set the event handler for receiving execution events
    pub fn with_event_handler(mut self, handler: Arc<dyn ExecutorEventHandler>) -> Self {
        self.event_handler = Some(handler);
        self
    }

    /// Set the event handler (mutable reference version)
    pub fn set_event_handler(&mut self, handler: Arc<dyn ExecutorEventHandler>) {
        self.event_handler = Some(handler);
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
        let conversation = vec![Message::user(user_message)];
        self.run_conversation(conversation).await
    }

    /// Execute the agent loop with conversation history
    ///
    /// # Arguments
    ///
    /// * `user_message` - The user's input message
    /// * `history` - Previous conversation messages
    ///
    /// # Returns
    ///
    /// The final response from the agent after all tool calls are complete
    pub async fn run_with_history(
        &self,
        user_message: String,
        history: Vec<Message>,
    ) -> Result<String> {
        let mut conversation = history;
        conversation.push(Message::user(user_message));
        self.run_conversation_with_handler(conversation, self.event_handler.clone())
            .await
    }

    /// Execute the agent loop with conversation history and a custom event handler
    ///
    /// This method allows passing a per-request event handler for real-time
    /// streaming of tool call events.
    pub async fn run_with_history_and_handler(
        &self,
        user_message: String,
        history: Vec<Message>,
        handler: Arc<dyn ExecutorEventHandler>,
    ) -> Result<String> {
        let mut conversation = history;
        conversation.push(Message::user(user_message));
        self.run_conversation_with_handler(conversation, Some(handler))
            .await
    }

    /// Internal method to run the agent loop with a conversation
    async fn run_conversation(&self, initial_conversation: Vec<Message>) -> Result<String> {
        self.run_conversation_with_handler(initial_conversation, self.event_handler.clone())
            .await
    }

    /// Internal method to run the agent loop with a conversation and optional handler
    async fn run_conversation_with_handler(
        &self,
        initial_conversation: Vec<Message>,
        event_handler: Option<Arc<dyn ExecutorEventHandler>>,
    ) -> Result<String> {
        let mut conversation = initial_conversation;
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
                iteration = iteration,
                max_iterations = self.config.max_iterations,
                "Agent iteration started"
            );

            // Build tool definitions from registry
            let tools = self.build_tool_definitions();
            debug!(tool_count = tools.len(), "Available tools");

            // Log the user message being processed
            if let Some(last_msg) = conversation.last() {
                let msg_preview: String = last_msg.text().unwrap_or("").chars().take(200).collect();
                debug!(
                    role = ?last_msg.role,
                    message_preview = %msg_preview,
                    "Processing message"
                );
            }

            // Call LLM
            info!(
                model = %self.config.model,
                max_tokens = self.config.max_tokens,
                temperature = ?self.config.temperature,
                tool_count = tools.len(),
                "Sending request to LLM"
            );
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

            // Log detailed response information
            info!(
                stop_reason = ?response.stop_reason,
                input_tokens = response.usage.input_tokens,
                output_tokens = response.usage.output_tokens,
                "LLM response received"
            );

            // Log response preview
            let response_preview: String = response.message.text()
                .unwrap_or("")
                .chars()
                .take(300)
                .collect();
            debug!(
                response_preview = %response_preview,
                "LLM response content preview"
            );

            // Add assistant message to conversation
            conversation.push(response.message.clone());

            // Check stop reason
            match response.stop_reason {
                StopReason::EndTurn => {
                    // Natural completion, extract text and return
                    let text = response.message.text().unwrap_or("No response").to_string();
                    info!(
                        iteration = iteration,
                        response_length = text.len(),
                        "Agent completed naturally"
                    );

                    // Emit complete event
                    if let Some(handler) = &event_handler {
                        handler.on_complete(&text).await;
                    }

                    return Ok(text);
                }

                StopReason::ToolUse => {
                    // Extract and execute tool calls
                    let tool_uses = response.message.tool_uses();
                    info!(
                        tool_count = tool_uses.len(),
                        "Agent requested tool use"
                    );
                    let tool_results = self
                        .execute_tools(&response.message, event_handler.as_ref())
                        .await?;

                    if tool_results.is_empty() {
                        warn!("No tool results despite ToolUse stop reason");
                        return Ok("Tool execution failed".to_string());
                    }

                    info!(
                        result_count = tool_results.len(),
                        "Tool execution completed, continuing agent loop"
                    );

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
                    let text = response.message.text().unwrap_or("No response").to_string();
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
            .map(|tool| ToolDefinition::new(tool.name(), tool.description(), tool.input_schema()))
            .collect()
    }

    /// Execute tool calls from an assistant message
    async fn execute_tools(
        &self,
        message: &Message,
        event_handler: Option<&Arc<dyn ExecutorEventHandler>>,
    ) -> Result<Vec<Message>> {
        let mut results = Vec::new();

        // Extract tool uses
        let tool_uses = message.tool_uses();
        info!(tool_count = tool_uses.len(), "Starting tool execution");

        for tool_use in tool_uses {
            if let ContentBlock::ToolUse { id, name, input } = tool_use {
                // Log tool input (truncated for safety)
                let input_preview: String = input.to_string().chars().take(500).collect();
                info!(
                    tool_name = %name,
                    tool_id = %id,
                    input_preview = %input_preview,
                    "Executing tool"
                );

                // Emit tool start event
                if let Some(handler) = event_handler {
                    handler.on_tool_start(id, name, input).await;
                }

                // Get tool from registry
                let tool = self.tool_registry.get(name).ok_or_else(|| {
                    agent_core::Error::ProcessingFailed(format!("Tool not found: {}", name))
                })?;

                // Execute tool and measure time
                let start_time = std::time::Instant::now();
                match tool.execute(input.clone()).await {
                    Ok(result) => {
                        let duration = start_time.elapsed();
                        let duration_ms = duration.as_millis() as u64;
                        // Convert result to string
                        let result_str =
                            serde_json::to_string(&result).unwrap_or_else(|_| result.to_string());
                        let result_preview: String = result_str.chars().take(500).collect();

                        info!(
                            tool_name = %name,
                            duration_ms = duration_ms,
                            result_length = result_str.len(),
                            result_preview = %result_preview,
                            "Tool execution succeeded"
                        );

                        // Emit tool done event
                        if let Some(handler) = event_handler {
                            handler
                                .on_tool_done(id, name, Ok(&result), duration_ms)
                                .await;
                        }

                        results.push(Message::tool_result(id.clone(), result_str));
                    }
                    Err(e) => {
                        let duration = start_time.elapsed();
                        let duration_ms = duration.as_millis() as u64;
                        let error_str = e.to_string();
                        warn!(
                            tool_name = %name,
                            duration_ms = duration_ms,
                            error = %e,
                            "Tool execution failed"
                        );

                        // Emit tool done event with error
                        if let Some(handler) = event_handler {
                            handler
                                .on_tool_done(id, name, Err(&error_str), duration_ms)
                                .await;
                        }

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

        Ok(AgentExecutor::new(
            provider,
            self.tool_registry,
            self.config,
        ))
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
        assert_eq!(
            builder.config.system_prompt,
            Some("Test prompt".to_string())
        );
    }

    #[test]
    fn test_default_config() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    }
}
