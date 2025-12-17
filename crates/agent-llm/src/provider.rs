//! LLM provider trait definition

use crate::{CompletionRequest, CompletionResponse, Result};
use async_trait::async_trait;

/// Trait for LLM providers
///
/// Implementations of this trait provide access to different LLM services
/// (e.g., Anthropic, OpenAI, Ollama).
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Generate a completion from the LLM
    ///
    /// # Arguments
    ///
    /// * `request` - The completion request with messages, tools, and parameters
    ///
    /// # Returns
    ///
    /// The completion response with the assistant's message and metadata
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Get the provider name (e.g., "anthropic", "openai")
    fn name(&self) -> &str;
}
