//! Anthropic Claude provider implementation
//!
//! This module implements the LLMProvider trait for Anthropic's Claude models.
//! See: https://docs.anthropic.com/en/api/messages

use crate::{
    CompletionRequest, CompletionResponse, ContentBlock, LLMProvider, Message, MessageContent,
    Result, Role, StopReason, TokenUsage, ToolDefinition,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider
///
/// Supports all Claude models including:
/// - claude-opus-4-5-20251101
/// - claude-sonnet-4-5-20250929
/// - claude-3-5-sonnet-20241022
pub struct AnthropicProvider {
    client: Client,
    api_key: String,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    ///
    /// # Arguments
    ///
    /// * `api_key` - Anthropic API key
    ///
    /// # Returns
    ///
    /// A new Anthropic provider instance
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self { client, api_key })
    }

    /// Create a provider from environment variable
    ///
    /// Reads the API key from the `ANTHROPIC_API_KEY` environment variable.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            crate::LLMError::ConfigurationError(
                "ANTHROPIC_API_KEY environment variable not set".to_string(),
            )
        })?;
        Self::new(api_key)
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    #[instrument(skip(self, request), fields(model = %request.model))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!("Sending request to Anthropic API");

        // Build Anthropic-specific request
        let anthropic_request = AnthropicRequest {
            model: request.model,
            messages: request.messages,
            system: request.system,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            tools: request.tools,
            stop_sequences: request.stop_sequences,
        };

        // Send request
        let response = self
            .client
            .post(format!("{ANTHROPIC_API_BASE}/messages"))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await?;

        // Handle errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;

            return Err(match status.as_u16() {
                401 => crate::LLMError::AuthenticationFailed,
                429 => crate::LLMError::RateLimitExceeded(error_text),
                400 => crate::LLMError::InvalidRequest(error_text),
                404 => crate::LLMError::ModelNotFound(anthropic_request.model),
                _ => crate::LLMError::RequestFailed(format!("HTTP {status}: {error_text}")),
            });
        }

        // Parse response
        let anthropic_response: AnthropicResponse = response.json().await.map_err(|e| {
            crate::LLMError::UnexpectedResponse(format!("Failed to parse response: {e}"))
        })?;

        debug!(
            "Received response - stop_reason: {}, tokens: {}/{}",
            anthropic_response.stop_reason,
            anthropic_response.usage.input_tokens,
            anthropic_response.usage.output_tokens
        );

        // Convert to our format
        Ok(CompletionResponse {
            message: Message {
                role: Role::Assistant,
                content: Some(MessageContent::Blocks(anthropic_response.content)),
            },
            stop_reason: match anthropic_response.stop_reason.as_str() {
                "end_turn" => StopReason::EndTurn,
                "max_tokens" => StopReason::MaxTokens,
                "stop_sequence" => StopReason::StopSequence,
                "tool_use" => StopReason::ToolUse,
                _ => {
                    debug!("Unknown stop reason: {}", anthropic_response.stop_reason);
                    StopReason::EndTurn
                }
            },
            usage: TokenUsage {
                input_tokens: anthropic_response.usage.input_tokens,
                output_tokens: anthropic_response.usage.output_tokens,
            },
        })
    }

    fn name(&self) -> &'static str {
        "anthropic"
    }
}

// Anthropic-specific request/response types
// These match the Anthropic API format exactly

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    stop_reason: String,
    usage: UsageResponse,
}

#[derive(Debug, Deserialize)]
struct UsageResponse {
    input_tokens: usize,
    output_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = AnthropicProvider::new("test-key".to_string());
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().name(), "anthropic");
    }

    #[test]
    fn test_from_env_without_key() {
        // This will fail if ANTHROPIC_API_KEY is not set
        // SAFETY: This is a test that modifies env vars, which is safe in single-threaded test context
        unsafe {
            std::env::remove_var("ANTHROPIC_API_KEY");
        }
        let result = AnthropicProvider::from_env();
        assert!(result.is_err());
    }
}
