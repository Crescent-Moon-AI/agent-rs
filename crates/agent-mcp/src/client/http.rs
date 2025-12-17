//! HTTP/SSE transport MCP client
//!
//! Communicates with a remote MCP server via HTTP and Server-Sent Events.
//! Uses JSON-RPC 2.0 protocol over HTTP POST requests.

use super::*;
use crate::config::MCPServerConfig;
use crate::retry::RetryPolicy;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// MCP client using HTTP/SSE transport
///
/// Communicates with a remote MCP server via HTTP POST requests.
/// Uses JSON-RPC 2.0 protocol over HTTP.
pub struct HttpMCPClient {
    url: String,
    headers: HashMap<String, String>,

    /// HTTP client
    http_client: reqwest::Client,

    /// Server info from initialization
    server_info: Arc<Mutex<Option<MCPServerInfo>>>,

    /// Connection state
    connected: Arc<Mutex<bool>>,

    /// Request ID counter
    request_id: Arc<Mutex<u64>>,

    /// Retry policy for connection and requests
    retry_policy: RetryPolicy,
}

impl HttpMCPClient {
    /// Create a new HTTP MCP client
    ///
    /// # Arguments
    ///
    /// * `url` - Server URL
    /// * `headers` - HTTP headers
    /// * `timeout` - Request timeout
    pub fn new(url: String, headers: HashMap<String, String>, timeout: Duration) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            url,
            headers,
            http_client,
            server_info: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            request_id: Arc::new(Mutex::new(0)),
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Create a new HTTP MCP client with custom retry policy
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Create from MCPServerConfig
    pub fn from_config(config: &MCPServerConfig) -> Result<Self> {
        match config {
            MCPServerConfig::Http {
                url,
                headers,
                timeout_secs,
            }
            | MCPServerConfig::Sse {
                url,
                headers,
                timeout_secs,
            } => Ok(Self::new(
                url.clone(),
                headers.clone(),
                Duration::from_secs(*timeout_secs),
            )),
            _ => Err(MCPError::ConfigError(
                "Expected HTTP/SSE transport config".to_string(),
            )),
        }
    }

    /// Get next request ID
    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        *id += 1;
        *id
    }

    /// Build HTTP headers
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut header_map = HeaderMap::new();
        header_map.insert("Content-Type", HeaderValue::from_static("application/json"));

        for (key, value) in &self.headers {
            let name = HeaderName::from_str(key).map_err(|e| {
                MCPError::ConfigError(format!("Invalid header name '{}': {}", key, e))
            })?;
            let value = HeaderValue::from_str(value).map_err(|e| {
                MCPError::ConfigError(format!("Invalid header value '{}': {}", value, e))
            })?;
            header_map.insert(name, value);
        }

        Ok(header_map)
    }

    /// Send a JSON-RPC request over HTTP
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_request_id().await;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        debug!("Sending HTTP request to {}: {}", self.url, method);

        let headers = self.build_headers()?;

        let response = self
            .http_client
            .post(&self.url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| MCPError::ConnectionFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(MCPError::RequestFailed(format!(
                "HTTP {} for {}: {}",
                response.status(),
                method,
                response.text().await.unwrap_or_default()
            )));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse response: {}", e)))?;

        debug!("Received response for: {}", method);

        // Check for JSON-RPC error
        if let Some(error) = response_json.get("error") {
            return Err(MCPError::RequestFailed(format!("{}: {}", method, error)));
        }

        // Return result
        response_json
            .get("result")
            .cloned()
            .ok_or_else(|| MCPError::RequestFailed("No result in response".to_string()))
    }

    /// Send initialize request
    async fn initialize(&self) -> Result<MCPServerInfo> {
        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "clientInfo": {
                "name": "agent-rs",
                "version": "0.1.0"
            }
        });

        let result = self.send_request("initialize", params).await?;

        // Parse server info
        let capabilities: MCPServerCapabilities =
            serde_json::from_value(result["capabilities"].clone()).unwrap_or_default();

        let server_info = MCPServerInfo {
            name: result["serverInfo"]["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            version: result["serverInfo"]["version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string(),
            protocol_version: result["protocolVersion"]
                .as_str()
                .unwrap_or("2024-11-05")
                .to_string(),
            capabilities,
        };

        info!(
            "Connected to MCP server: {} v{}",
            server_info.name, server_info.version
        );

        // Send initialized notification (fire and forget)
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let headers = self.build_headers()?;
        let _ = self
            .http_client
            .post(&self.url)
            .headers(headers)
            .json(&notification)
            .send()
            .await;

        Ok(server_info)
    }
}

#[async_trait]
impl MCPClient for HttpMCPClient {
    async fn connect(&self) -> Result<()> {
        debug!("Connecting to MCP server: {}", self.url);

        // Test connection with initialize (with retry)
        let url = self.url.clone();
        let server_info = self
            .retry_policy
            .execute(&format!("connect to {}", url), || async {
                self.initialize().await
            })
            .await?;

        *self.server_info.lock().await = Some(server_info);
        *self.connected.lock().await = true;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Non-blocking check using try_lock
        self.connected
            .try_lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    async fn disconnect(&self) -> Result<()> {
        debug!("Disconnecting from MCP server");
        *self.connected.lock().await = false;
        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<MCPToolDefinition>> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let result = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        let tools: Vec<MCPToolDefinition> = serde_json::from_value(result["tools"].clone())
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse tools: {}", e)))?;

        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<MCPToolResult> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments
        });

        let result = self.send_request("tools/call", params).await?;

        let tool_result: MCPToolResult = serde_json::from_value(result)
            .map_err(|e| MCPError::ToolCallFailed(format!("Failed to parse result: {}", e)))?;

        Ok(tool_result)
    }

    async fn list_resources(&self) -> Result<Vec<MCPResourceDefinition>> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let result = self
            .send_request("resources/list", serde_json::json!({}))
            .await?;

        let resources: Vec<MCPResourceDefinition> =
            serde_json::from_value(result["resources"].clone()).map_err(|e| {
                MCPError::RequestFailed(format!("Failed to parse resources: {}", e))
            })?;

        Ok(resources)
    }

    async fn read_resource(&self, uri: &str) -> Result<MCPResourceContent> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let params = serde_json::json!({
            "uri": uri
        });

        let result = self.send_request("resources/read", params).await?;

        let content: MCPResourceContent = serde_json::from_value(result["contents"].clone())
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse resource: {}", e)))?;

        Ok(content)
    }

    async fn list_prompts(&self) -> Result<Vec<MCPPromptDefinition>> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let result = self
            .send_request("prompts/list", serde_json::json!({}))
            .await?;

        let prompts: Vec<MCPPromptDefinition> =
            serde_json::from_value(result["prompts"].clone())
                .map_err(|e| MCPError::RequestFailed(format!("Failed to parse prompts: {}", e)))?;

        Ok(prompts)
    }

    async fn get_prompt(&self, name: &str, arguments: Option<Value>) -> Result<MCPPromptResult> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let params = serde_json::json!({
            "name": name,
            "arguments": arguments.unwrap_or(serde_json::json!({}))
        });

        let result = self.send_request("prompts/get", params).await?;

        let prompt_result: MCPPromptResult = serde_json::from_value(result)
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse prompt: {}", e)))?;

        Ok(prompt_result)
    }

    async fn server_info(&self) -> Option<MCPServerInfo> {
        self.server_info.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_creation() {
        let client = HttpMCPClient::new(
            "http://localhost:8080/mcp".to_string(),
            HashMap::new(),
            Duration::from_secs(30),
        );

        assert_eq!(client.url, "http://localhost:8080/mcp");
    }

    #[test]
    fn test_from_config_http() {
        let config = MCPServerConfig::Http {
            url: "http://example.com/mcp".to_string(),
            headers: HashMap::new(),
            timeout_secs: 30,
        };

        let client = HttpMCPClient::from_config(&config).unwrap();
        assert_eq!(client.url, "http://example.com/mcp");
    }

    #[test]
    fn test_from_config_sse() {
        let config = MCPServerConfig::Sse {
            url: "http://example.com/sse".to_string(),
            headers: HashMap::new(),
            timeout_secs: 60,
        };

        let client = HttpMCPClient::from_config(&config).unwrap();
        assert_eq!(client.url, "http://example.com/sse");
    }

    #[test]
    fn test_from_config_wrong_type() {
        let config = MCPServerConfig::Stdio {
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
        };

        let result = HttpMCPClient::from_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());
        headers.insert("X-Custom".to_string(), "value".to_string());

        let client = HttpMCPClient::new(
            "http://localhost:8080".to_string(),
            headers,
            Duration::from_secs(30),
        );

        let header_map = client.build_headers().unwrap();

        assert_eq!(header_map.get("Content-Type").unwrap(), "application/json");
        assert_eq!(header_map.get("Authorization").unwrap(), "Bearer token123");
        assert_eq!(header_map.get("X-Custom").unwrap(), "value");
    }
}
