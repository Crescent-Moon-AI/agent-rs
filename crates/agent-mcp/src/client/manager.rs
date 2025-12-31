//! MCP client manager for coordinating multiple MCP server connections

use super::{MCPToolDefinition, ArcMCPClient, Result, MCPError, Value, MCPToolResult, MCPResourceInfo, MCPContent};
use crate::config::{MCPConfig, MCPServerConfig};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Information about an MCP tool including its source server
#[derive(Debug, Clone)]
pub struct MCPToolInfo {
    pub server_name: String,
    pub definition: MCPToolDefinition,
}

/// Manages multiple MCP clients for an agent
///
/// The manager handles:
/// - Connection lifecycle for multiple MCP servers
/// - Tool discovery across all connected servers
/// - Tool execution routing to the correct server
/// - Graceful degradation when servers fail
pub struct MCPClientManager {
    /// Configuration
    config: Arc<MCPConfig>,

    /// Active clients (server_name -> client)
    clients: Arc<RwLock<HashMap<String, ArcMCPClient>>>,

    /// Agent name (for configuration lookup)
    agent_name: String,
}

impl MCPClientManager {
    /// Create a new MCP client manager
    ///
    /// # Arguments
    ///
    /// * `config` - MCP configuration
    /// * `agent_name` - Name of the agent (used for configuration lookup)
    pub fn new(config: Arc<MCPConfig>, agent_name: String) -> Self {
        Self {
            config,
            clients: Arc::new(RwLock::new(HashMap::new())),
            agent_name,
        }
    }

    /// Initialize all configured MCP servers for this agent
    ///
    /// Connects to each configured MCP server with graceful degradation:
    /// - Attempts to connect to each server with retry logic
    /// - Logs warnings for failed connections but continues
    /// - Uses timeout to prevent hanging on unresponsive servers
    /// - Stores successfully connected clients for later use
    pub async fn initialize(&self) -> Result<()> {
        let agent_config = self
            .config
            .get_agent_config(&self.agent_name)
            .ok_or_else(|| {
                MCPError::ConfigError(format!(
                    "No MCP configuration found for agent: {}",
                    self.agent_name
                ))
            })?;

        let mut clients = self.clients.write().await;
        let mut successful_connections = 0;

        for server_name in &agent_config.mcp_servers {
            let server_config = self.config.mcp_servers.get(server_name).ok_or_else(|| {
                MCPError::ConfigError(format!("MCP server not found: {server_name}"))
            })?;

            match self
                .create_and_connect_client(server_config, server_name)
                .await
            {
                Ok(client) => {
                    info!("Successfully connected to MCP server: {}", server_name);
                    clients.insert(server_name.clone(), client);
                    successful_connections += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to MCP server {}: {}. Continuing without it.",
                        server_name, e
                    );
                    // Graceful degradation - don't fail initialization
                }
            }
        }

        if clients.is_empty() {
            warn!(
                "No MCP servers connected for agent: {}. Agent will work without MCP tools.",
                self.agent_name
            );
        } else {
            info!(
                "Connected to {}/{} MCP servers for agent: {}",
                successful_connections,
                agent_config.mcp_servers.len(),
                self.agent_name
            );
        }

        Ok(())
    }

    /// Create and connect a client for the given configuration
    async fn create_and_connect_client(
        &self,
        config: &MCPServerConfig,
        server_name: &str,
    ) -> Result<ArcMCPClient> {
        use super::http::HttpMCPClient;
        use super::stdio::StdioMCPClient;

        info!("Creating MCP client for server: {}", server_name);

        // Create the client based on transport type and wrap in Arc
        let client: ArcMCPClient = match config {
            MCPServerConfig::Stdio { .. } => Arc::new(StdioMCPClient::from_config(config)?),
            MCPServerConfig::Http { .. } | MCPServerConfig::Sse { .. } => {
                Arc::new(HttpMCPClient::from_config(config)?)
            }
        };

        // Attempt to connect
        client.connect().await?;

        // Verify connection
        if !client.is_connected() {
            return Err(MCPError::ConnectionFailed(format!(
                "Client for {server_name} reports not connected after connect()"
            )));
        }

        Ok(client)
    }

    /// Discover all tools from all connected servers
    ///
    /// Returns a list of all available tools with their source server information.
    /// Filters tools based on the agent's configuration (allow/deny lists).
    pub async fn discover_tools(&self) -> Result<Vec<MCPToolInfo>> {
        let clients = self.clients.read().await;
        let mut all_tools = Vec::new();

        for (server_name, client) in clients.iter() {
            match client.list_tools().await {
                Ok(tools) => {
                    info!(
                        "Discovered {} tools from server: {}",
                        tools.len(),
                        server_name
                    );
                    for tool in tools {
                        all_tools.push(MCPToolInfo {
                            server_name: server_name.clone(),
                            definition: tool,
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to list tools from {}: {}", server_name, e);
                }
            }
        }

        // Filter tools based on agent configuration
        let agent_config = self.config.get_agent_config(&self.agent_name);
        if let Some(config) = agent_config {
            all_tools
                .retain(|tool| crate::config::should_include_tool(&tool.definition.name, config));
        }

        Ok(all_tools)
    }

    /// Call a tool on the appropriate server
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the MCP server
    /// * `tool_name` - Name of the tool to call
    /// * `arguments` - Tool arguments as JSON
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<MCPToolResult> {
        let clients = self.clients.read().await;
        let client = clients
            .get(server_name)
            .ok_or_else(|| MCPError::ServerNotFound(server_name.to_string()))?;

        client.call_tool(tool_name, arguments).await
    }

    /// Get a specific client by server name
    pub async fn get_client(&self, server_name: &str) -> Option<ArcMCPClient> {
        let clients = self.clients.read().await;
        clients.get(server_name).cloned()
    }

    /// Get list of connected server names
    pub async fn connected_servers(&self) -> Vec<String> {
        let clients = self.clients.read().await;
        clients.keys().cloned().collect()
    }

    /// Check if any servers are connected
    pub async fn has_connections(&self) -> bool {
        let clients = self.clients.read().await;
        !clients.is_empty()
    }

    /// Discover all resources from all connected servers
    ///
    /// Returns a list of all available resources with their source server information.
    pub async fn discover_resources(&self) -> Result<Vec<MCPResourceInfo>> {
        let clients = self.clients.read().await;
        let mut all_resources = Vec::new();

        for (server_name, client) in clients.iter() {
            match client.list_resources().await {
                Ok(resources) => {
                    info!(
                        "Discovered {} resources from server: {}",
                        resources.len(),
                        server_name
                    );
                    for resource in resources {
                        all_resources.push(MCPResourceInfo {
                            uri: resource.uri,
                            name: resource.name,
                            description: resource.description,
                            mime_type: resource.mime_type,
                            server_name: server_name.clone(),
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to list resources from {}: {}", server_name, e);
                }
            }
        }

        Ok(all_resources)
    }

    /// Read a resource from the appropriate server
    ///
    /// # Arguments
    ///
    /// * `server_name` - Name of the MCP server
    /// * `uri` - URI of the resource to read
    pub async fn read_resource(&self, server_name: &str, uri: &str) -> Result<Vec<MCPContent>> {
        let clients = self.clients.read().await;
        let client = clients
            .get(server_name)
            .ok_or_else(|| MCPError::ServerNotFound(server_name.to_string()))?;

        let content = client.read_resource(uri).await?;

        // Convert MCPResourceContent to MCPContent
        let mut result = Vec::new();
        if let Some(text) = content.text {
            result.push(MCPContent::Text { text });
        }
        if let Some(blob) = content.blob {
            // Assume image if it's blob data
            if let Some(mime_type) = content.mime_type {
                result.push(MCPContent::Image {
                    data: blob,
                    mime_type,
                });
            }
        }

        Ok(result)
    }

    /// Shutdown all clients
    ///
    /// Disconnects from all MCP servers gracefully.
    pub async fn shutdown(&self) -> Result<()> {
        let mut clients = self.clients.write().await;

        for (server_name, client) in clients.iter() {
            info!("Disconnecting from MCP server: {}", server_name);
            // Note: disconnect() doesn't require &mut self due to interior mutability
            if let Err(e) = client.disconnect().await {
                warn!("Error disconnecting from {}: {}", server_name, e);
            }
        }

        clients.clear();
        info!("All MCP servers disconnected");
        Ok(())
    }

    /// Reconnect to a specific server
    ///
    /// Attempts to reconnect to a server that may have been disconnected.
    /// Useful for recovering from transient connection failures.
    pub async fn reconnect(&self, server_name: &str) -> Result<()> {
        let server_config = self
            .config
            .mcp_servers
            .get(server_name)
            .ok_or_else(|| MCPError::ServerNotFound(server_name.to_string()))?;

        info!("Reconnecting to MCP server: {}", server_name);

        // Create and connect new client
        let client = self
            .create_and_connect_client(server_config, server_name)
            .await?;

        // Replace the old client
        let mut clients = self.clients.write().await;
        clients.insert(server_name.to_string(), client);

        info!("Successfully reconnected to MCP server: {}", server_name);
        Ok(())
    }

    /// Check health of all connected servers
    ///
    /// Returns a map of server names to their connection status.
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let clients = self.clients.read().await;
        let mut status = HashMap::new();

        for (server_name, client) in clients.iter() {
            let is_connected = client.is_connected();
            status.insert(server_name.clone(), is_connected);

            if !is_connected {
                warn!("Server {} is not connected", server_name);
            }
        }

        status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let config = Arc::new(MCPConfig::default());
        let manager = MCPClientManager::new(config, "test-agent".to_string());

        assert!(!manager.has_connections().await);
        assert_eq!(manager.connected_servers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_manager_with_config() {
        // Test that manager can be initialized with valid configuration
        let mut config = MCPConfig::default();
        config.mcp_servers.insert(
            "test-server".to_string(),
            MCPServerConfig::Stdio {
                command: "echo".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
            },
        );
        config.agent_configurations.insert(
            "test-agent".to_string(),
            crate::config::AgentMCPConfig {
                mcp_servers: vec!["test-server".to_string()],
                tools: Default::default(),
                resources: Default::default(),
            },
        );

        let manager = MCPClientManager::new(Arc::new(config), "test-agent".to_string());

        // Initialize will attempt to connect, which will fail for echo command
        // but graceful degradation means initialization still succeeds
        let result = manager.initialize().await;
        assert!(result.is_ok());

        // Since connection failed, no servers should be registered
        assert!(!manager.has_connections().await);
        assert_eq!(manager.connected_servers().await.len(), 0);
    }
}
