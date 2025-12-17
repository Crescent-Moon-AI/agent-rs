//! MCP client implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::Result;
use crate::error::MCPError;

pub mod http;
pub mod manager;
pub mod stdio;

/// MCP client trait - abstracts over different transports
///
/// Note: All methods use &self (not &mut self) to enable use through Arc.
/// Implementations use interior mutability (Arc<Mutex<...>>) for state changes.
#[async_trait]
pub trait MCPClient: Send + Sync {
    /// Initialize connection to MCP server
    async fn connect(&self) -> Result<()>;

    /// Check if client is connected
    fn is_connected(&self) -> bool;

    /// Disconnect from server
    async fn disconnect(&self) -> Result<()>;

    /// List available tools
    async fn list_tools(&self) -> Result<Vec<MCPToolDefinition>>;

    /// Call a tool
    async fn call_tool(&self, name: &str, arguments: Value) -> Result<MCPToolResult>;

    /// List available resources
    async fn list_resources(&self) -> Result<Vec<MCPResourceDefinition>>;

    /// Read a resource
    async fn read_resource(&self, uri: &str) -> Result<MCPResourceContent>;

    /// List available prompts
    async fn list_prompts(&self) -> Result<Vec<MCPPromptDefinition>>;

    /// Get a prompt
    async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<MCPPromptResult>;

    /// Get server info (from initialize response)
    async fn server_info(&self) -> Option<MCPServerInfo>;
}

/// MCP tool definition (from tools/list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value, // JSON Schema
}

/// MCP tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub content: Vec<MCPContent>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isError")]
    pub is_error: Option<bool>,
}

/// MCP content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MCPContent {
    Text {
        text: String,
    },
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    Resource {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
        mime_type: Option<String>,
    },
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResourceDefinition {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// MCP resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResourceContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "mimeType")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>, // base64
}

/// Resource information with server context (used by manager)
#[derive(Debug, Clone)]
pub struct MCPResourceInfo {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub server_name: String,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<MCPPromptArgument>>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// MCP prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub messages: Vec<MCPPromptMessage>,
}

/// MCP prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPPromptMessage {
    pub role: String,
    pub content: MCPContent,
}

/// MCP server info (from initialize)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerInfo {
    pub name: String,
    pub version: String,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: MCPServerCapabilities,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MCPServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

/// Tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
    #[serde(default)]
    pub subscribe: bool,
}

/// Prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(default, rename = "listChanged")]
    pub list_changed: bool,
}

/// Type alias for Arc-wrapped MCP client
pub type ArcMCPClient = Arc<dyn MCPClient>;
