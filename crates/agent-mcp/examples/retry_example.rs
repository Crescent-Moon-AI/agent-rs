//! Retry logic example for agent-mcp
//!
//! This example demonstrates:
//! - Configuring custom retry policies
//! - Using retry logic with MCP clients
//! - Handling transient failures
//!
//! Run with: cargo run --example retry_example

use agent_mcp::{MCPConfig, MCPClientManager, RetryPolicy};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see retry attempts
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    println!("=== Agent MCP Retry Logic Example ===\n");

    // 1. Create custom retry policies
    println!("1. Creating custom retry policies...\n");

    // Fast retry policy (for testing or low-latency scenarios)
    let fast_policy = RetryPolicy::new(
        3,                              // 3 attempts
        Duration::from_millis(10),      // 10ms initial backoff
        Duration::from_millis(100),     // 100ms max backoff
        2.0,                            // exponential multiplier
    );
    println!("   Fast policy: 3 attempts, 10-100ms backoff");

    // Default policy (balanced)
    let default_policy = RetryPolicy::default();
    println!("   Default policy: 3 attempts, 100ms-10s backoff");

    // Aggressive retry policy (for unreliable networks)
    let aggressive_policy = RetryPolicy::new(
        5,                              // 5 attempts
        Duration::from_millis(200),     // 200ms initial backoff
        Duration::from_secs(30),        // 30s max backoff
        2.0,                            // exponential multiplier
    );
    println!("   Aggressive policy: 5 attempts, 200ms-30s backoff\n");

    // 2. Demonstrate retry behavior
    println!("2. Demonstrating retry behavior...\n");

    // Example: Retry a failing operation
    let result = default_policy
        .execute("example_operation", || async {
            // This would be a real MCP operation
            println!("   Attempting operation...");
            Err(agent_mcp::MCPError::ConnectionFailed(
                "Simulated transient failure".to_string(),
            ))
        })
        .await;

    match result {
        Ok(_) => println!("\n   ✓ Operation succeeded"),
        Err(e) => println!("\n   ✗ Operation failed after all retries: {}", e),
    }
    println!();

    // 3. Using retry policies with MCP clients
    println!("3. Using retry policies with MCP clients...\n");

    let config = create_example_config();
    let manager = MCPClientManager::new(Arc::new(config), "retry-example".to_string());

    // The manager will automatically use retry logic during initialization
    println!("   Initializing with automatic retry...");
    match manager.initialize().await {
        Ok(_) => println!("   ✓ Initialization succeeded"),
        Err(e) => println!("   ⚠ Initialization completed with warnings: {}", e),
    }
    println!();

    // 4. Reconnection example
    if manager.has_connections().await {
        println!("4. Testing reconnection...\n");

        let servers = manager.connected_servers().await;
        if let Some(server) = servers.first() {
            println!("   Attempting to reconnect to '{}'...", server);
            match manager.reconnect(server).await {
                Ok(_) => println!("   ✓ Reconnection succeeded"),
                Err(e) => println!("   ✗ Reconnection failed: {}", e),
            }
        }
    }

    println!("\n=== Example completed ===");
    Ok(())
}

fn create_example_config() -> MCPConfig {
    use agent_mcp::{AgentMCPConfig, MCPServerConfig};
    use std::collections::HashMap;

    let mut config = MCPConfig::default();

    // Configure an HTTP MCP server with custom timeout
    config.mcp_servers.insert(
        "example-http-server".to_string(),
        MCPServerConfig::Http {
            url: "http://localhost:3000/mcp".to_string(),
            headers: HashMap::new(),
            timeout_secs: 30,
        },
    );

    config.agent_configurations.insert(
        "retry-example".to_string(),
        AgentMCPConfig {
            mcp_servers: vec!["example-http-server".to_string()],
            tools: Default::default(),
            resources: Default::default(),
        },
    );

    config
}
