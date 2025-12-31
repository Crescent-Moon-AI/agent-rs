//! OpenAI provider implementation
//!
//! This module implements the LLMProvider trait for OpenAI's GPT models.
//! See: https://platform.openai.com/docs/api-reference/chat
//!
//! # Examples
//!
//! ## Basic usage with environment variable
//!
//! ```no_run
//! use agent_llm::{CompletionRequest, Message, LLMProvider};
//! use agent_llm::providers::OpenAIProvider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create provider from OPENAI_API_KEY environment variable
//!     let provider = OpenAIProvider::from_env()?;
//!
//!     // Build request
//!     let request = CompletionRequest::builder("gpt-4-turbo")
//!         .add_message(Message::user("Hello!"))
//!         .max_tokens(100)
//!         .build();
//!
//!     // Get completion
//!     let response = provider.complete(request).await?;
//!     println!("{}", response.message.text().unwrap());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Custom configuration
//!
//! ```no_run
//! use agent_llm::{CompletionRequest, Message, LLMProvider};
//! use agent_llm::providers::{OpenAIProvider, OpenAIConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create custom configuration
//!     let config = OpenAIConfig::new("sk-...")
//!         .with_api_base("https://api.openai.com/v1")
//!         .with_timeout(60)
//!         .with_supported_models(vec![
//!             "gpt-4-turbo".to_string(),
//!             "gpt-4".to_string(),
//!         ]);
//!
//!     let provider = OpenAIProvider::with_config(config)?;
//!
//!     let request = CompletionRequest::builder("gpt-4-turbo")
//!         .add_message(Message::user("Hello!"))
//!         .max_tokens(100)
//!         .build();
//!
//!     let response = provider.complete(request).await?;
//!     println!("{}", response.message.text().unwrap());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Using with OpenAI-compatible APIs
//!
//! ```no_run
//! use agent_llm::providers::{OpenAIProvider, OpenAIConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // For local LLM deployments (e.g., llama.cpp, vLLM, text-generation-webui)
//! let local_config = OpenAIConfig::new("not-needed")
//!     .with_api_base("http://localhost:8000/v1");
//!
//! // For Azure OpenAI
//! let azure_config = OpenAIConfig::new("your-azure-key")
//!     .with_api_base("https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT");
//!
//! let provider = OpenAIProvider::with_config(local_config)?;
//! # Ok(())
//! # }
//! ```

use crate::{
    CompletionRequest, CompletionResponse, ContentBlock, ImageSource, LLMProvider, Message,
    MessageContent, Result, Role, StopReason, TokenUsage, ToolDefinition,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, instrument};

const DEFAULT_OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Configuration for OpenAI provider
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// API key for authentication
    pub api_key: String,

    /// Base URL for the OpenAI API (default: "https://api.openai.com/v1")
    /// Can be customized for OpenAI-compatible APIs like Azure OpenAI, local deployments, etc.
    pub api_base: String,

    /// Request timeout in seconds (default: 120)
    pub timeout_secs: u64,

    /// Optional list of supported models
    /// If None, any model string is accepted
    pub supported_models: Option<Vec<String>>,
}

impl OpenAIConfig {
    /// Create a new config with the given API key and default settings
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_base: DEFAULT_OPENAI_API_BASE.to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            supported_models: None,
        }
    }

    /// Create config from environment variable
    ///
    /// Reads the API key from `OPENAI_API_KEY` environment variable.
    /// Optionally reads base URL from `OPENAI_API_BASE` if set.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            crate::LLMError::ConfigurationError(
                "OPENAI_API_KEY environment variable not set".to_string(),
            )
        })?;

        let api_base = std::env::var("OPENAI_API_BASE")
            .unwrap_or_else(|_| DEFAULT_OPENAI_API_BASE.to_string());

        Ok(Self {
            api_key,
            api_base,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            supported_models: None,
        })
    }

    /// Set custom API base URL
    ///
    /// Useful for:
    /// - Azure OpenAI: "https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT"
    /// - Local deployments: "http://localhost:8000/v1"
    /// - Other OpenAI-compatible APIs
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// Set request timeout in seconds
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set supported models list
    ///
    /// When set, the provider will validate model names against this list.
    /// When None (default), any model string is accepted.
    pub fn with_supported_models(mut self, models: Vec<String>) -> Self {
        self.supported_models = Some(models);
        self
    }

    /// Add a single supported model
    pub fn add_supported_model(mut self, model: impl Into<String>) -> Self {
        let model = model.into();
        match &mut self.supported_models {
            Some(models) => models.push(model),
            None => self.supported_models = Some(vec![model]),
        }
        self
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: DEFAULT_OPENAI_API_BASE.to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            supported_models: None,
        }
    }
}

/// OpenAI provider
///
/// Supports GPT models including:
/// - gpt-4-turbo
/// - gpt-4
/// - gpt-3.5-turbo
/// - gpt-4o
///
/// Also compatible with OpenAI-compatible APIs through custom configuration.
pub struct OpenAIProvider {
    client: Client,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - OpenAI configuration
    ///
    /// # Returns
    ///
    /// A new OpenAI provider instance
    ///
    /// # Example
    ///
    /// ```no_run
    /// use agent_llm::providers::{OpenAIProvider, OpenAIConfig};
    ///
    /// let config = OpenAIConfig::new("sk-...")
    ///     .with_api_base("https://api.openai.com/v1")
    ///     .with_timeout(60)
    ///     .with_supported_models(vec![
    ///         "gpt-4-turbo".to_string(),
    ///         "gpt-4".to_string(),
    ///     ]);
    ///
    /// let provider = OpenAIProvider::with_config(config)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn with_config(config: OpenAIConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { client, config })
    }

    /// Create a new OpenAI provider with API key and default settings
    ///
    /// # Arguments
    ///
    /// * `api_key` - OpenAI API key
    ///
    /// # Returns
    ///
    /// A new OpenAI provider instance
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(OpenAIConfig::new(api_key))
    }

    /// Create a provider from environment variable
    ///
    /// Reads the API key from the `OPENAI_API_KEY` environment variable.
    /// Optionally reads base URL from `OPENAI_API_BASE` if set.
    pub fn from_env() -> Result<Self> {
        let config = OpenAIConfig::from_env()?;
        Self::with_config(config)
    }

    /// Get the current configuration
    pub fn config(&self) -> &OpenAIConfig {
        &self.config
    }

    /// Validate model name against supported models list (if configured)
    fn validate_model(&self, model: &str) -> Result<()> {
        if let Some(supported) = &self.config.supported_models {
            if !supported.iter().any(|m| m == model) {
                return Err(crate::LLMError::InvalidRequest(format!(
                    "Model '{model}' is not in the supported models list: {supported:?}"
                )));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    #[instrument(skip(self, request), fields(model = %request.model, api_base = %self.config.api_base))]
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        debug!("Sending request to OpenAI API at {}", self.config.api_base);

        // Validate model if configured
        self.validate_model(&request.model)?;

        // Convert messages (system prompt goes into messages array for OpenAI)
        let openai_messages = build_openai_messages(request.system.clone(), request.messages);

        // Convert tools if present
        let openai_tools = request.tools.as_ref().map(|tools| convert_tools(tools));

        // Build OpenAI-specific request
        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: openai_messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            tools: openai_tools,
            stop: request.stop_sequences,
        };

        // Send request
        let response = self
            .client
            .post(format!("{}/chat/completions", self.config.api_base))
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
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
                404 => crate::LLMError::ModelNotFound(request.model),
                _ => crate::LLMError::RequestFailed(format!("HTTP {status}: {error_text}")),
            });
        }

        // Parse response
        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            crate::LLMError::UnexpectedResponse(format!("Failed to parse response: {e}"))
        })?;

        // Extract first choice (OpenAI can return multiple but we use first)
        let choice = openai_response.choices.into_iter().next().ok_or_else(|| {
            crate::LLMError::UnexpectedResponse("No choices in response".to_string())
        })?;

        debug!(
            "Received response - stop_reason: {}, tokens: {}/{}",
            choice.finish_reason,
            openai_response.usage.prompt_tokens,
            openai_response.usage.completion_tokens
        );

        // Convert response message to our format
        let message = parse_openai_response(choice.message)?;

        // Map stop reason
        let stop_reason = map_stop_reason(&choice.finish_reason);

        // Build response
        Ok(CompletionResponse {
            message,
            stop_reason,
            usage: TokenUsage {
                input_tokens: openai_response.usage.prompt_tokens,
                output_tokens: openai_response.usage.completion_tokens,
            },
        })
    }

    fn name(&self) -> &'static str {
        "openai"
    }
}

// ============================================================================
// OpenAI-specific request types
// ============================================================================

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAIContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum OpenAIContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize, Clone)]
struct ContentPart {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl>,
}

#[derive(Debug, Serialize, Clone)]
struct ImageUrl {
    url: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

// ============================================================================
// OpenAI-specific response types
// ============================================================================

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIResponseToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseToolCall {
    id: String,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIResponseFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
}

// ============================================================================
// Conversion functions
// ============================================================================

/// Build OpenAI messages from our generic format
///
/// Key difference from Anthropic: system messages go into the messages array
fn build_openai_messages(system: Option<String>, messages: Vec<Message>) -> Vec<OpenAIMessage> {
    let mut result = Vec::new();

    // Add system message first if present
    if let Some(sys) = system {
        result.push(OpenAIMessage {
            role: "system".to_string(),
            content: Some(OpenAIContent::Text(sys)),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }

    // Convert each message
    for msg in messages {
        result.extend(convert_message(msg));
    }

    result
}

/// Convert a single message to OpenAI format
///
/// This may return multiple OpenAI messages (e.g., tool results become separate messages)
fn convert_message(msg: Message) -> Vec<OpenAIMessage> {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
    };

    match msg.content {
        Some(MessageContent::Text(text)) => {
            vec![OpenAIMessage {
                role: role.to_string(),
                content: Some(OpenAIContent::Text(text)),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }]
        }
        Some(MessageContent::Blocks(blocks)) => convert_blocks(role, blocks),
        None => {
            vec![OpenAIMessage {
                role: role.to_string(),
                content: Some(OpenAIContent::Text(String::new())),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }]
        }
    }
}

/// Convert content blocks to OpenAI messages
fn convert_blocks(role: &str, blocks: Vec<ContentBlock>) -> Vec<OpenAIMessage> {
    let mut messages = Vec::new();
    let mut content_parts = Vec::new();
    let mut tool_calls = Vec::new();

    for block in blocks {
        match block {
            ContentBlock::Text { text } => {
                content_parts.push(ContentPart {
                    content_type: "text".to_string(),
                    text: Some(text),
                    image_url: None,
                });
            }
            ContentBlock::Image { source } => {
                let url = match source {
                    ImageSource::Url { url } => url,
                    ImageSource::Base64 { media_type, data } => {
                        format!("data:{media_type};base64,{data}")
                    }
                };
                content_parts.push(ContentPart {
                    content_type: "image_url".to_string(),
                    text: None,
                    image_url: Some(ImageUrl { url }),
                });
            }
            ContentBlock::ToolUse { id, name, input } => {
                // Tool uses go in the tool_calls array
                let arguments = serde_json::to_string(&input).unwrap_or_default();
                tool_calls.push(OpenAIToolCall {
                    id,
                    tool_type: "function".to_string(),
                    function: OpenAIFunctionCall { name, arguments },
                });
            }
            ContentBlock::ToolResult {
                tool_use_id,
                content,
                ..
            } => {
                // Tool results become separate messages with role="tool"
                messages.push(OpenAIMessage {
                    role: "tool".to_string(),
                    content: Some(OpenAIContent::Text(content)),
                    tool_calls: None,
                    tool_call_id: Some(tool_use_id),
                    name: None,
                });
            }
        }
    }

    // Build the main message if we have content or tool calls
    if !content_parts.is_empty() || !tool_calls.is_empty() {
        let content = if content_parts.is_empty() {
            None
        } else {
            if content_parts.len() == 1 && content_parts[0].content_type == "text" {
                // Single text part - use simple string format
                content_parts[0].text.clone().map(OpenAIContent::Text)
            } else {
                // Multiple parts or contains images - use array format
                Some(OpenAIContent::Parts(content_parts))
            }
        };

        messages.insert(
            0,
            OpenAIMessage {
                role: role.to_string(),
                content,
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls)
                },
                tool_call_id: None,
                name: None,
            },
        );
    }

    messages
}

/// Convert tool definitions to OpenAI format
fn convert_tools(tools: &[ToolDefinition]) -> Vec<OpenAITool> {
    tools
        .iter()
        .map(|tool| OpenAITool {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            },
        })
        .collect()
}

/// Parse OpenAI response message to our format
fn parse_openai_response(msg: OpenAIResponseMessage) -> Result<Message> {
    let mut blocks = Vec::new();

    // Add text content if present
    if let Some(content) = msg.content {
        if !content.is_empty() {
            blocks.push(ContentBlock::Text { text: content });
        }
    }

    // Parse tool calls
    if let Some(tool_calls) = msg.tool_calls {
        for call in tool_calls {
            // Parse arguments from JSON string
            let input: serde_json::Value =
                serde_json::from_str(&call.function.arguments).map_err(|e| {
                    crate::LLMError::UnexpectedResponse(format!(
                        "Failed to parse tool arguments: {e}"
                    ))
                })?;

            blocks.push(ContentBlock::ToolUse {
                id: call.id,
                name: call.function.name,
                input,
            });
        }
    }

    // If no blocks, add empty text
    if blocks.is_empty() {
        blocks.push(ContentBlock::Text {
            text: String::new(),
        });
    }

    Ok(Message {
        role: Role::Assistant,
        content: Some(MessageContent::Blocks(blocks)),
    })
}

/// Map OpenAI stop reason to our format
fn map_stop_reason(reason: &str) -> StopReason {
    match reason {
        "stop" => StopReason::EndTurn,
        "length" => StopReason::MaxTokens,
        "tool_calls" => StopReason::ToolUse,
        "content_filter" => {
            debug!("Content filtered by OpenAI safety systems");
            StopReason::EndTurn
        }
        _ => {
            debug!("Unknown stop reason: {}", reason);
            StopReason::EndTurn
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_provider_creation() {
        let provider = OpenAIProvider::new("test-key");
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.config().api_key, "test-key");
        assert_eq!(provider.config().api_base, "https://api.openai.com/v1");
    }

    #[test]
    fn test_provider_with_custom_config() {
        let config = OpenAIConfig::new("test-key")
            .with_api_base("https://custom.api.com/v1")
            .with_timeout(60)
            .with_supported_models(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]);

        let provider = OpenAIProvider::with_config(config).unwrap();
        assert_eq!(provider.config().api_base, "https://custom.api.com/v1");
        assert_eq!(provider.config().timeout_secs, 60);
        assert_eq!(
            provider.config().supported_models,
            Some(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()])
        );
    }

    #[test]
    fn test_config_builder() {
        let config = OpenAIConfig::new("test-key")
            .add_supported_model("gpt-4")
            .add_supported_model("gpt-3.5-turbo");

        assert_eq!(
            config.supported_models,
            Some(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()])
        );
    }

    #[test]
    fn test_model_validation() {
        let config = OpenAIConfig::new("test-key")
            .with_supported_models(vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()]);

        let provider = OpenAIProvider::with_config(config).unwrap();

        // Valid model
        assert!(provider.validate_model("gpt-4").is_ok());
        assert!(provider.validate_model("gpt-3.5-turbo").is_ok());

        // Invalid model
        let result = provider.validate_model("invalid-model");
        assert!(result.is_err());
        assert!(matches!(result, Err(crate::LLMError::InvalidRequest(_))));
    }

    #[test]
    fn test_no_model_validation_when_not_configured() {
        let provider = OpenAIProvider::new("test-key").unwrap();

        // Any model should be accepted when no supported_models list is set
        assert!(provider.validate_model("any-model").is_ok());
        assert!(provider.validate_model("custom-model").is_ok());
    }

    #[test]
    fn test_config_from_env() {
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "test-key-from-env");
            std::env::set_var("OPENAI_API_BASE", "https://custom.openai.com/v1");
        }

        let config = OpenAIConfig::from_env().unwrap();
        assert_eq!(config.api_key, "test-key-from-env");
        assert_eq!(config.api_base, "https://custom.openai.com/v1");

        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("OPENAI_API_BASE");
        }
    }

    #[test]
    fn test_from_env_without_key() {
        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
        }
        let result = OpenAIProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_simple_text_message_conversion() {
        let msg = Message::user("Hello");
        let openai_msgs = convert_message(msg);

        assert_eq!(openai_msgs.len(), 1);
        assert_eq!(openai_msgs[0].role, "user");
        match &openai_msgs[0].content {
            Some(OpenAIContent::Text(text)) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_system_message_in_array() {
        let messages = build_openai_messages(Some("You are helpful".to_string()), vec![]);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "system");
        match &messages[0].content {
            Some(OpenAIContent::Text(text)) => assert_eq!(text, "You are helpful"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_tool_definition_conversion() {
        let tool = ToolDefinition {
            name: "search".to_string(),
            description: "Search the web".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                }
            }),
        };

        let openai_tools = convert_tools(&[tool]);

        assert_eq!(openai_tools.len(), 1);
        assert_eq!(openai_tools[0].tool_type, "function");
        assert_eq!(openai_tools[0].function.name, "search");
        assert_eq!(openai_tools[0].function.description, "Search the web");
    }

    #[test]
    fn test_stop_reason_mapping() {
        assert_eq!(map_stop_reason("stop"), StopReason::EndTurn);
        assert_eq!(map_stop_reason("length"), StopReason::MaxTokens);
        assert_eq!(map_stop_reason("tool_calls"), StopReason::ToolUse);
        assert_eq!(map_stop_reason("content_filter"), StopReason::EndTurn);
        assert_eq!(map_stop_reason("unknown"), StopReason::EndTurn);
    }

    #[test]
    fn test_tool_result_conversion() {
        let msg = Message::tool_result("call_123".to_string(), "result data".to_string());
        let openai_msgs = convert_message(msg);

        assert_eq!(openai_msgs.len(), 1);
        assert_eq!(openai_msgs[0].role, "tool");
        assert_eq!(openai_msgs[0].tool_call_id, Some("call_123".to_string()));
        match &openai_msgs[0].content {
            Some(OpenAIContent::Text(text)) => assert_eq!(text, "result data"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_image_url_conversion() {
        let msg = Message {
            role: Role::User,
            content: Some(MessageContent::Blocks(vec![
                ContentBlock::Text {
                    text: "What's this?".to_string(),
                },
                ContentBlock::Image {
                    source: ImageSource::Url {
                        url: "https://example.com/image.jpg".to_string(),
                    },
                },
            ])),
        };

        let openai_msgs = convert_message(msg);

        assert_eq!(openai_msgs.len(), 1);
        assert_eq!(openai_msgs[0].role, "user");
        match &openai_msgs[0].content {
            Some(OpenAIContent::Parts(parts)) => {
                assert_eq!(parts.len(), 2);
                assert_eq!(parts[0].content_type, "text");
                assert_eq!(parts[1].content_type, "image_url");
            }
            _ => panic!("Expected multi-part content"),
        }
    }

    #[test]
    fn test_base64_image_conversion() {
        let msg = Message {
            role: Role::User,
            content: Some(MessageContent::Blocks(vec![ContentBlock::Image {
                source: ImageSource::Base64 {
                    media_type: "image/png".to_string(),
                    data: "abc123".to_string(),
                },
            }])),
        };

        let openai_msgs = convert_message(msg);

        assert_eq!(openai_msgs.len(), 1);
        match &openai_msgs[0].content {
            Some(OpenAIContent::Parts(parts)) => {
                assert_eq!(parts.len(), 1);
                assert_eq!(parts[0].content_type, "image_url");
                if let Some(img_url) = &parts[0].image_url {
                    assert_eq!(img_url.url, "data:image/png;base64,abc123");
                }
            }
            _ => panic!("Expected parts content"),
        }
    }

    #[test]
    fn test_response_with_tool_calls() {
        let response_msg = OpenAIResponseMessage {
            role: "assistant".to_string(),
            content: Some("Let me search for that".to_string()),
            tool_calls: Some(vec![OpenAIResponseToolCall {
                id: "call_123".to_string(),
                tool_type: "function".to_string(),
                function: OpenAIResponseFunctionCall {
                    name: "search".to_string(),
                    arguments: r#"{"query":"test"}"#.to_string(),
                },
            }]),
        };

        let message = parse_openai_response(response_msg).unwrap();

        assert_eq!(message.role, Role::Assistant);
        match message.content {
            Some(MessageContent::Blocks(blocks)) => {
                assert_eq!(blocks.len(), 2); // text + tool use
                assert!(matches!(blocks[0], ContentBlock::Text { .. }));
                match &blocks[1] {
                    ContentBlock::ToolUse { id, name, input } => {
                        assert_eq!(id, "call_123");
                        assert_eq!(name, "search");
                        assert_eq!(input["query"], "test");
                    }
                    _ => panic!("Expected tool use"),
                }
            }
            _ => panic!("Expected blocks"),
        }
    }

    #[test]
    fn test_multiple_tool_results() {
        let msg = Message {
            role: Role::User,
            content: Some(MessageContent::Blocks(vec![
                ContentBlock::ToolResult {
                    tool_use_id: "call_1".to_string(),
                    content: "result 1".to_string(),
                    is_error: None,
                },
                ContentBlock::ToolResult {
                    tool_use_id: "call_2".to_string(),
                    content: "result 2".to_string(),
                    is_error: None,
                },
            ])),
        };

        let openai_msgs = convert_message(msg);

        // Should create 2 separate tool messages
        assert_eq!(openai_msgs.len(), 2);
        assert_eq!(openai_msgs[0].role, "tool");
        assert_eq!(openai_msgs[0].tool_call_id, Some("call_1".to_string()));
        assert_eq!(openai_msgs[1].role, "tool");
        assert_eq!(openai_msgs[1].tool_call_id, Some("call_2".to_string()));
    }
}
