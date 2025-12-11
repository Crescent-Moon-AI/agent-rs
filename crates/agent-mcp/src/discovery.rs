//! Tool discovery and registration
//!
//! This module provides functionality to discover tools from MCP servers
//! and register them into agent-rs ToolRegistry.

use agent_tools::ToolRegistry;
use std::sync::Arc;
use tracing::{debug, info};

use crate::client::manager::MCPClientManager;
use crate::config::AgentMCPConfig;
use crate::tool::MCPTool;
use crate::Result;

/// Discover and register MCP tools into a ToolRegistry
///
/// This function:
/// 1. Discovers all available tools from connected MCP servers
/// 2. Filters tools based on agent configuration (allow/deny lists)
/// 3. Wraps MCP tools as MCPTool instances
/// 4. Registers them in the provided ToolRegistry
///
/// # Arguments
///
/// * `client_manager` - Manager coordinating MCP server connections
/// * `registry` - ToolRegistry to register discovered tools into
/// * `agent_config` - Agent-specific MCP configuration for filtering
///
/// # Returns
///
/// Number of tools successfully registered
///
/// # Example
///
/// ```no_run
/// use agent_mcp::discovery::discover_and_register_tools;
/// use agent_mcp::client::manager::MCPClientManager;
/// use agent_mcp::config::{MCPConfig, AgentMCPConfig};
/// use agent_tools::ToolRegistry;
/// use std::sync::Arc;
///
/// # async fn example() -> agent_mcp::Result<()> {
/// let config = Arc::new(MCPConfig::default());
/// let manager = Arc::new(MCPClientManager::new(config.clone(), "my-agent".to_string()));
/// let mut registry = ToolRegistry::new();
///
/// // Initialize connections
/// manager.initialize().await?;
///
/// // Discover and register tools
/// let agent_config = config.get_agent_config("my-agent").unwrap();
/// let count = discover_and_register_tools(
///     manager,
///     &mut registry,
///     agent_config
/// ).await?;
///
/// println!("Registered {} MCP tools", count);
/// # Ok(())
/// # }
/// ```
pub async fn discover_and_register_tools(
    client_manager: Arc<MCPClientManager>,
    registry: &mut ToolRegistry,
    agent_config: &AgentMCPConfig,
) -> Result<usize> {
    info!("Discovering MCP tools for agent configuration");

    // Discover all available tools (already filtered by manager)
    let tools = client_manager.discover_tools().await?;

    debug!("Found {} tools from MCP servers", tools.len());

    let mut registered_count = 0;

    // Wrap each tool and register it
    for tool_info in tools {
        let tool_name = tool_info.definition.name.clone();

        // Double-check filtering (manager already filters, but be explicit)
        if !crate::config::should_include_tool(&tool_name, agent_config) {
            debug!("Skipping tool '{}' (filtered by config)", tool_name);
            continue;
        }

        // Create MCPTool wrapper
        let mcp_tool = MCPTool::new(tool_info.clone(), client_manager.clone());

        // Register in the tool registry
        registry.register(Arc::new(mcp_tool));
        registered_count += 1;

        debug!(
            "Registered MCP tool '{}' from server '{}'",
            tool_name, tool_info.server_name
        );
    }

    info!(
        "Successfully registered {} MCP tools into registry",
        registered_count
    );

    Ok(registered_count)
}

/// Discover tools without registering them
///
/// Useful for inspecting available tools without modifying a registry.
///
/// # Arguments
///
/// * `client_manager` - Manager coordinating MCP server connections
///
/// # Returns
///
/// List of tool names discovered
pub async fn list_available_tools(client_manager: Arc<MCPClientManager>) -> Result<Vec<String>> {
    let tools = client_manager.discover_tools().await?;
    Ok(tools.into_iter().map(|t| t.definition.name).collect())
}

/// Discover tools from a specific MCP server
///
/// # Arguments
///
/// * `client_manager` - Manager coordinating MCP server connections
/// * `server_name` - Name of the MCP server to query
///
/// # Returns
///
/// List of tool names from the specified server
pub async fn list_server_tools(
    client_manager: Arc<MCPClientManager>,
    server_name: &str,
) -> Result<Vec<String>> {
    let tools = client_manager.discover_tools().await?;

    Ok(tools
        .into_iter()
        .filter(|t| t.server_name == server_name)
        .map(|t| t.definition.name)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MCPConfig, ToolFilter, ToolPattern};

    #[tokio::test]
    async fn test_discover_tools_empty() {
        let config = Arc::new(MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));

        let tools = list_available_tools(manager).await.unwrap();
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_discover_and_register_with_empty_manager() {
        let config = Arc::new(MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));
        let mut registry = ToolRegistry::new();

        let agent_config = AgentMCPConfig {
            mcp_servers: vec![],
            tools: Default::default(),
            resources: Default::default(),
        };

        let count = discover_and_register_tools(manager, &mut registry, &agent_config)
            .await
            .unwrap();

        assert_eq!(count, 0);
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_tool_filtering_integration() {
        // Test that the filtering logic works correctly
        let agent_config = AgentMCPConfig {
            mcp_servers: vec!["test".to_string()],
            tools: ToolFilter {
                allow: ToolPattern::List(vec!["allowed_tool".to_string()]),
                deny: vec!["denied_tool".to_string()],
            },
            resources: Default::default(),
        };

        assert!(crate::config::should_include_tool(
            "allowed_tool",
            &agent_config
        ));
        assert!(!crate::config::should_include_tool(
            "denied_tool",
            &agent_config
        ));
        assert!(!crate::config::should_include_tool(
            "other_tool",
            &agent_config
        ));
    }
}
