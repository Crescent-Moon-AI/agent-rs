//! MCPTool wrapper that implements the Tool trait

use agent_tools::Tool;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

use crate::client::MCPContent;
use crate::client::manager::{MCPClientManager, MCPToolInfo};

/// Wrapper that implements agent-tools::Tool for MCP tools
///
/// This wrapper bridges MCP tools to the agent-rs tool system by:
/// - Implementing the `Tool` trait from agent-tools
/// - Delegating execution to the MCPClientManager
/// - Converting MCP results to agent-rs format
pub struct MCPTool {
    /// Tool information (name, schema, server)
    info: MCPToolInfo,

    /// Client manager for calling the tool
    client_manager: Arc<MCPClientManager>,
}

impl MCPTool {
    /// Create a new MCPTool
    ///
    /// # Arguments
    ///
    /// * `info` - Tool information from MCP server
    /// * `client_manager` - Manager to route tool calls
    pub fn new(info: MCPToolInfo, client_manager: Arc<MCPClientManager>) -> Self {
        Self {
            info,
            client_manager,
        }
    }

    /// Get the server name this tool belongs to
    pub fn server_name(&self) -> &str {
        &self.info.server_name
    }

    /// Convert MCP content blocks to a JSON value suitable for agent-rs
    ///
    /// Extracts text from content blocks and includes metadata about other content types.
    fn convert_mcp_result(content: Vec<MCPContent>) -> Value {
        let mut text_parts = Vec::new();
        let mut images = Vec::new();
        let mut resources = Vec::new();

        for block in content {
            match block {
                MCPContent::Text { text } => {
                    text_parts.push(text);
                }
                MCPContent::Image { data, mime_type } => {
                    images.push(serde_json::json!({
                        "type": "image",
                        "mimeType": mime_type,
                        "dataLength": data.len(),
                    }));
                }
                MCPContent::Resource { uri, mime_type } => {
                    resources.push(serde_json::json!({
                        "type": "resource",
                        "uri": uri,
                        "mimeType": mime_type,
                    }));
                }
            }
        }

        // Build result JSON
        let mut result = serde_json::json!({
            "text": text_parts.join("\n"),
        });

        if !images.is_empty() {
            result["images"] = serde_json::json!(images);
        }

        if !resources.is_empty() {
            result["resources"] = serde_json::json!(resources);
        }

        result
    }
}

#[async_trait]
impl Tool for MCPTool {
    async fn execute(&self, params: Value) -> agent_core::Result<Value> {
        // Call the tool through the client manager
        let result = self
            .client_manager
            .call_tool(&self.info.server_name, &self.info.definition.name, params)
            .await
            .map_err(|e| {
                agent_core::Error::ProcessingFailed(format!("MCP tool call failed: {e}"))
            })?;

        // Check if the tool returned an error
        if result.is_error.unwrap_or(false) {
            return Err(agent_core::Error::ProcessingFailed(format!(
                "MCP tool '{}' returned error: {:?}",
                self.info.definition.name, result.content
            )));
        }

        // Convert MCP result to agent-rs format
        Ok(Self::convert_mcp_result(result.content))
    }

    fn name(&self) -> &str {
        &self.info.definition.name
    }

    fn description(&self) -> &str {
        self.info
            .definition
            .description
            .as_deref()
            .unwrap_or("No description available")
    }

    fn input_schema(&self) -> Value {
        self.info.definition.input_schema.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::MCPToolDefinition;

    #[test]
    fn test_convert_mcp_result_text_only() {
        let content = vec![MCPContent::Text {
            text: "Hello, world!".to_string(),
        }];

        let result = MCPTool::convert_mcp_result(content);

        assert_eq!(result["text"], "Hello, world!");
        assert!(result.get("images").is_none());
        assert!(result.get("resources").is_none());
    }

    #[test]
    fn test_convert_mcp_result_multiple_text() {
        let content = vec![
            MCPContent::Text {
                text: "First line".to_string(),
            },
            MCPContent::Text {
                text: "Second line".to_string(),
            },
        ];

        let result = MCPTool::convert_mcp_result(content);

        assert_eq!(result["text"], "First line\nSecond line");
    }

    #[test]
    fn test_convert_mcp_result_mixed_content() {
        let content = vec![
            MCPContent::Text {
                text: "Some text".to_string(),
            },
            MCPContent::Image {
                data: "base64data".to_string(),
                mime_type: "image/png".to_string(),
            },
            MCPContent::Resource {
                uri: "file:///test.txt".to_string(),
                mime_type: Some("text/plain".to_string()),
            },
        ];

        let result = MCPTool::convert_mcp_result(content);

        assert_eq!(result["text"], "Some text");
        assert!(result["images"].is_array());
        assert_eq!(result["images"].as_array().unwrap().len(), 1);
        assert!(result["resources"].is_array());
        assert_eq!(result["resources"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_mcp_tool_metadata() {
        let config = Arc::new(crate::config::MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));

        let tool_info = MCPToolInfo {
            server_name: "test-server".to_string(),
            definition: MCPToolDefinition {
                name: "test_tool".to_string(),
                description: Some("A test tool".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "param": {"type": "string"}
                    }
                }),
            },
        };

        let tool = MCPTool::new(tool_info, manager);

        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
        assert_eq!(tool.server_name(), "test-server");
        assert!(tool.input_schema().is_object());
    }

    #[test]
    fn test_mcp_tool_no_description() {
        let config = Arc::new(crate::config::MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));

        let tool_info = MCPToolInfo {
            server_name: "test-server".to_string(),
            definition: MCPToolDefinition {
                name: "test_tool".to_string(),
                description: None,
                input_schema: serde_json::json!({}),
            },
        };

        let tool = MCPTool::new(tool_info, manager);

        assert_eq!(tool.description(), "No description available");
    }
}
