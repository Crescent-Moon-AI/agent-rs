//! Stdio transport MCP client
//!
//! Communicates with an MCP server via standard input/output by spawning
//! the server as a child process.

use super::*;
use crate::config::MCPServerConfig;
use crate::retry::RetryPolicy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::{debug, info};

/// MCP client using stdio transport
///
/// Communicates with an MCP server via standard input/output by spawning
/// the server as a child process. Uses JSON-RPC 2.0 protocol over stdio.
pub struct StdioMCPClient {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<PathBuf>,

    /// Child process handle
    child: Arc<Mutex<Option<Child>>>,

    /// Stdin writer
    stdin: Arc<Mutex<Option<ChildStdin>>>,

    /// Stdout reader
    stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,

    /// Server info from initialization
    server_info: Arc<Mutex<Option<MCPServerInfo>>>,

    /// Connection state
    connected: Arc<Mutex<bool>>,

    /// Request ID counter
    request_id: Arc<Mutex<u64>>,

    /// Retry policy for connection and requests
    retry_policy: RetryPolicy,
}

impl StdioMCPClient {
    /// Create a new stdio MCP client
    ///
    /// # Arguments
    ///
    /// * `command` - Command to execute
    /// * `args` - Command arguments
    /// * `env` - Environment variables
    /// * `cwd` - Working directory (optional)
    pub fn new(
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<PathBuf>,
    ) -> Self {
        Self {
            command,
            args,
            env,
            cwd,
            child: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            server_info: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            request_id: Arc::new(Mutex::new(0)),
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Create a new stdio MCP client with custom retry policy
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    /// Create from MCPServerConfig
    pub fn from_config(config: &MCPServerConfig) -> Result<Self> {
        match config {
            MCPServerConfig::Stdio {
                command,
                args,
                env,
                cwd,
            } => Ok(Self::new(
                command.clone(),
                args.clone(),
                env.clone(),
                cwd.clone(),
            )),
            _ => Err(MCPError::ConfigError(
                "Expected Stdio transport config".to_string(),
            )),
        }
    }

    /// Get next request ID
    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        *id += 1;
        *id
    }

    /// Send a JSON-RPC request
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_request_id().await;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        debug!("Sending request: {}", method);

        let mut stdin = self.stdin.lock().await;
        let stdin = stdin
            .as_mut()
            .ok_or(MCPError::NotConnected)?;

        let request_str = serde_json::to_string(&request)?;
        stdin.write_all(request_str.as_bytes()).await
            .map_err(|e| MCPError::ConnectionFailed(e.to_string()))?;
        stdin.write_all(b"\n").await
            .map_err(|e| MCPError::ConnectionFailed(e.to_string()))?;
        stdin.flush().await
            .map_err(|e| MCPError::ConnectionFailed(e.to_string()))?;

        // Read response
        let mut stdout = self.stdout.lock().await;
        let stdout = stdout
            .as_mut()
            .ok_or(MCPError::NotConnected)?;

        let mut line = String::new();
        stdout.read_line(&mut line).await
            .map_err(|e| MCPError::ConnectionFailed(e.to_string()))?;

        if line.is_empty() {
            return Err(MCPError::ConnectionFailed("Server closed connection".to_string()));
        }

        let response: Value = serde_json::from_str(&line)?;

        debug!("Received response for: {}", method);

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return Err(MCPError::RequestFailed(format!("{}: {}", method, error)));
        }

        // Return result
        response.get("result")
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
        let capabilities: MCPServerCapabilities = serde_json::from_value(
            result["capabilities"].clone()
        ).unwrap_or_default();

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

        info!("Connected to MCP server: {} v{}", server_info.name, server_info.version);

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        let mut stdin = self.stdin.lock().await;
        if let Some(stdin) = stdin.as_mut() {
            let notification_str = serde_json::to_string(&notification)?;
            let _ = stdin.write_all(notification_str.as_bytes()).await;
            let _ = stdin.write_all(b"\n").await;
            let _ = stdin.flush().await;
        }

        Ok(server_info)
    }
}

#[async_trait]
impl MCPClient for StdioMCPClient {
    async fn connect(&self) -> Result<()> {
        debug!("Starting MCP server: {} {:?}", self.command, self.args);

        // Spawn child process
        let mut command = Command::new(&self.command);
        command.args(&self.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());

        // Set environment variables
        for (key, value) in &self.env {
            command.env(key, value);
        }

        // Set working directory
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }

        let mut child = command.spawn()
            .map_err(|e| MCPError::ConnectionFailed(format!("Failed to spawn process: {}", e)))?;

        // Get stdin/stdout
        let stdin = child.stdin.take()
            .ok_or_else(|| MCPError::ConnectionFailed("Failed to get stdin".to_string()))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| MCPError::ConnectionFailed("Failed to get stdout".to_string()))?;

        // Store handles
        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        *self.child.lock().await = Some(child);

        // Initialize protocol
        let server_info = self.initialize().await?;
        *self.server_info.lock().await = Some(server_info);
        *self.connected.lock().await = true;

        Ok(())
    }

    fn is_connected(&self) -> bool {
        // Non-blocking check using try_lock
        self.connected.try_lock()
            .map(|guard| *guard)
            .unwrap_or(false)
    }

    async fn disconnect(&self) -> Result<()> {
        debug!("Disconnecting from MCP server");

        *self.connected.lock().await = false;

        // Close stdin
        *self.stdin.lock().await = None;
        *self.stdout.lock().await = None;

        // Kill child process
        let mut child = self.child.lock().await;
        if let Some(child) = child.as_mut() {
            let _ = child.kill().await;
        }
        *child = None;

        Ok(())
    }

    async fn list_tools(&self) -> Result<Vec<MCPToolDefinition>> {
        if !self.is_connected() {
            return Err(MCPError::NotConnected);
        }

        let result = self.send_request("tools/list", serde_json::json!({})).await?;

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

        let result = self.send_request("resources/list", serde_json::json!({})).await?;

        let resources: Vec<MCPResourceDefinition> = serde_json::from_value(result["resources"].clone())
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse resources: {}", e)))?;

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

        let result = self.send_request("prompts/list", serde_json::json!({})).await?;

        let prompts: Vec<MCPPromptDefinition> = serde_json::from_value(result["prompts"].clone())
            .map_err(|e| MCPError::RequestFailed(format!("Failed to parse prompts: {}", e)))?;

        Ok(prompts)
    }

    async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<MCPPromptResult> {
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

impl Drop for StdioMCPClient {
    fn drop(&mut self) {
        // Best effort cleanup - kill child process
        if let Ok(mut child) = self.child.try_lock() {
            if let Some(child) = child.as_mut() {
                let _ = child.start_kill();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdio_client_creation() {
        let client = StdioMCPClient::new(
            "echo".to_string(),
            vec!["hello".to_string()],
            HashMap::new(),
            None,
        );

        assert_eq!(client.command, "echo");
        assert_eq!(client.args, vec!["hello"]);
    }

    #[test]
    fn test_from_config() {
        let config = MCPServerConfig::Stdio {
            command: "test-command".to_string(),
            args: vec!["arg1".to_string()],
            env: HashMap::new(),
            cwd: None,
        };

        let client = StdioMCPClient::from_config(&config).unwrap();
        assert_eq!(client.command, "test-command");
    }

    #[test]
    fn test_from_config_wrong_type() {
        let config = MCPServerConfig::Http {
            url: "http://example.com".to_string(),
            headers: HashMap::new(),
            timeout_secs: 30,
        };

        let result = StdioMCPClient::from_config(&config);
        assert!(result.is_err());
    }
}
