//! Basic usage example for agent-mcp
//!
//! This example demonstrates:
//! - Loading MCP configuration
//! - Creating an MCP client manager
//! - Discovering tools from MCP servers
//! - Calling MCP tools
//! - Working with MCP resources
//!
//! Run with: cargo run --example basic_usage --features example

use agent_mcp::{MCPClientManager, MCPConfig, MCPContent};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("=== Agent MCP Basic Usage Example ===\n");

    // 1. Load MCP configuration
    println!("1. Loading MCP configuration...");
    let config = load_example_config();
    println!("   ✓ Configuration loaded\n");

    // 2. Create MCP client manager
    println!("2. Creating MCP client manager...");
    let manager = MCPClientManager::new(Arc::new(config), "example-agent".to_string());
    println!("   ✓ Manager created\n");

    // 3. Initialize connections to MCP servers
    println!("3. Initializing connections to MCP servers...");
    match manager.initialize().await {
        Ok(_) => println!("   ✓ Initialization completed"),
        Err(e) => println!("   ⚠ Initialization completed with warnings: {}", e),
    }
    println!();

    // 4. Check connection status
    println!("4. Checking connection status...");
    if !manager.has_connections().await {
        println!("   ⚠ No MCP servers connected");
        println!("   This example requires actual MCP servers to be running.");
        println!("   Configure MCP servers in .mcp.json to use this example.\n");
        return Ok(());
    }

    let connected = manager.connected_servers().await;
    println!(
        "   ✓ Connected to {} server(s): {:?}\n",
        connected.len(),
        connected
    );

    // 5. Health check
    println!("5. Performing health check...");
    let health = manager.health_check().await;
    for (server, status) in &health {
        let icon = if *status { "✓" } else { "✗" };
        println!(
            "   {} {}: {}",
            icon,
            server,
            if *status { "healthy" } else { "unhealthy" }
        );
    }
    println!();

    // 6. Discover available tools
    println!("6. Discovering available tools...");
    match manager.discover_tools().await {
        Ok(tools) => {
            println!("   ✓ Found {} tool(s):", tools.len());
            for tool in tools.iter().take(5) {
                println!(
                    "     - {} (from {})",
                    tool.definition.name, tool.server_name
                );
                if let Some(desc) = &tool.definition.description {
                    println!("       {}", desc);
                }
            }
            if tools.len() > 5 {
                println!("     ... and {} more", tools.len() - 5);
            }
        }
        Err(e) => println!("   ⚠ Failed to discover tools: {}", e),
    }
    println!();

    // 7. Discover available resources
    println!("7. Discovering available resources...");
    match manager.discover_resources().await {
        Ok(resources) => {
            println!("   ✓ Found {} resource(s):", resources.len());
            for resource in resources.iter().take(5) {
                println!("     - {} (from {})", resource.name, resource.server_name);
                println!("       URI: {}", resource.uri);
                if let Some(desc) = &resource.description {
                    println!("       {}", desc);
                }
            }
            if resources.len() > 5 {
                println!("     ... and {} more", resources.len() - 5);
            }
        }
        Err(e) => println!("   ⚠ Failed to discover resources: {}", e),
    }
    println!();

    // 8. Shutdown gracefully
    println!("8. Shutting down...");
    manager.shutdown().await?;
    println!("   ✓ All connections closed\n");

    println!("=== Example completed successfully ===");
    Ok(())
}

/// Load example configuration
///
/// In a real application, you would load this from .mcp.json
fn load_example_config() -> MCPConfig {
    use agent_mcp::{AgentMCPConfig, MCPServerConfig};
    use std::collections::HashMap;

    let mut config = MCPConfig::default();

    // Example: Configure a stdio MCP server
    // This is just for demonstration - you would need an actual MCP server
    config.mcp_servers.insert(
        "example-server".to_string(),
        MCPServerConfig::Stdio {
            command: "npx".to_string(),
            args: vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-filesystem".to_string(),
                "/tmp".to_string(),
            ],
            env: HashMap::new(),
            cwd: None,
        },
    );

    // Configure agent to use the server
    config.agent_configurations.insert(
        "example-agent".to_string(),
        AgentMCPConfig {
            mcp_servers: vec!["example-server".to_string()],
            tools: Default::default(),
            resources: Default::default(),
        },
    );

    config
}
