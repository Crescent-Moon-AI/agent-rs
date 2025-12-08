//! Completion request and response types

use crate::{Message, ToolDefinition};
use serde::{Deserialize, Serialize};

/// Request for LLM completion with full conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    /// Model identifier (provider-specific)
    pub model: String,

    /// Conversation history (alternating user/assistant messages)
    pub messages: Vec<Message>,

    /// Optional system prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,

    /// Maximum tokens to generate
    pub max_tokens: usize,

    /// Sampling temperature (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Tools available for the LLM to call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

/// Response from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Generated message from the assistant
    pub message: Message,

    /// Stop reason (completed, max_tokens, tool_use, etc.)
    pub stop_reason: StopReason,

    /// Token usage statistics
    pub usage: TokenUsage,
}

/// Reason the LLM stopped generating
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural completion (end of turn)
    EndTurn,

    /// Hit max tokens limit
    MaxTokens,

    /// Stop sequence encountered
    StopSequence,

    /// Tool use requested
    ToolUse,
}

/// Token usage statistics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens
    pub input_tokens: usize,

    /// Number of output tokens
    pub output_tokens: usize,
}

impl TokenUsage {
    /// Total tokens used (input + output)
    pub fn total(&self) -> usize {
        self.input_tokens + self.output_tokens
    }
}

impl CompletionRequest {
    /// Create a builder for completion requests
    pub fn builder(model: impl Into<String>) -> CompletionRequestBuilder {
        CompletionRequestBuilder::new(model)
    }
}

/// Builder for CompletionRequest
pub struct CompletionRequestBuilder {
    model: String,
    messages: Vec<Message>,
    system: Option<String>,
    max_tokens: usize,
    temperature: Option<f32>,
    tools: Option<Vec<ToolDefinition>>,
    stop_sequences: Option<Vec<String>>,
}

impl CompletionRequestBuilder {
    /// Create a new builder
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            system: None,
            max_tokens: 1024,
            temperature: None,
            tools: None,
            stop_sequences: None,
        }
    }

    /// Set the conversation messages
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Add a single message
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set the maximum tokens
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the available tools
    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set stop sequences
    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }

    /// Build the completion request
    pub fn build(self) -> CompletionRequest {
        CompletionRequest {
            model: self.model,
            messages: self.messages,
            system: self.system,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            tools: self.tools,
            stop_sequences: self.stop_sequences,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Message;

    #[test]
    fn test_builder() {
        let request = CompletionRequest::builder("claude-3-5-sonnet-20241022")
            .add_message(Message::user("Hello"))
            .system("You are a helpful assistant")
            .max_tokens(2048)
            .temperature(0.7)
            .build();

        assert_eq!(request.model, "claude-3-5-sonnet-20241022");
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, 2048);
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
        };
        assert_eq!(usage.total(), 150);
    }
}
