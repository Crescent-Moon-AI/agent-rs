//! Example demonstrating MCP integration with agent-runtime
//!
//! This example shows how to:
//! 1. Load MCP configuration from a file
//! 2. Create an AgentRuntime with MCP support
//! 3. Create tool agents that automatically discover MCP tools
//!
//! ## Requirements
//!
//! - ANTHROPIC_API_KEY environment variable must be set
//! - Optional: .mcp.json file for MCP server configuration
//!
//! ## To run this example:
//! ```bash
//! export ANTHROPIC_API_KEY=your_key_here
//! cargo run --example mcp_integration
//! ```

use agent_core::Result;
use agent_llm::providers::AnthropicProvider;
use agent_runtime::{AgentRuntime, ExecutorConfig};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== MCP Integration Example ===\n");

    // Example 1: Creating runtime with MCP configuration from file
    println!("1. Loading MCP configuration from file...");
    let mcp_config_path = PathBuf::from(".mcp.json");

    // Note: This will fail if .mcp.json doesn't exist, which is expected
    let runtime_result = create_runtime_with_mcp(mcp_config_path.clone()).await;

    match runtime_result {
        Ok(runtime) => {
            println!("✓ Successfully loaded MCP configuration\n");

            // Example 2: Create a tool agent with MCP support
            println!("2. Creating tool agent with MCP tools...");
            let agent = runtime
                .create_tool_agent_with_mcp(ExecutorConfig::default(), "research-agent")
                .await?;

            println!("✓ Agent '{}' created successfully", agent.name());
            println!("  (MCP tools have been automatically discovered and registered)\n");

            // Example 3: Using the agent would look like this:
            // let result = agent.execute("Search for Rust MCP documentation").await?;
        }
        Err(e) => {
            println!("✗ Failed to load MCP configuration: {}", e);
            println!("  This is expected if .mcp.json doesn't exist yet.\n");

            // Example 4: Create runtime without MCP (fallback)
            println!("3. Creating runtime without MCP (fallback mode)...");
            let runtime = create_runtime_without_mcp().await?;
            println!("✓ Runtime created without MCP support\n");

            // Even without MCP config, we can still create agents
            let agent = runtime.create_tool_agent(ExecutorConfig::default(), "simple-agent");
            println!("✓ Simple agent '{}' created successfully", agent.name());
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Create an AgentRuntime with MCP configuration loaded from file
async fn create_runtime_with_mcp(config_path: PathBuf) -> Result<Arc<AgentRuntime>> {
    // Get Anthropic API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| agent_core::Error::ConfigError(
            "ANTHROPIC_API_KEY environment variable not set. Set it with: export ANTHROPIC_API_KEY=your_key".to_string()
        ))?;

    let provider = Arc::new(AnthropicProvider::new(api_key)?);

    // Build runtime with MCP configuration
    let runtime = AgentRuntime::builder()
        .provider(provider)
        .mcp_config_from_file(config_path)?
        .build()?;

    Ok(Arc::new(runtime))
}

/// Create an AgentRuntime without MCP support
async fn create_runtime_without_mcp() -> Result<Arc<AgentRuntime>> {
    // Get Anthropic API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| agent_core::Error::ConfigError(
            "ANTHROPIC_API_KEY environment variable not set. Set it with: export ANTHROPIC_API_KEY=your_key".to_string()
        ))?;

    let provider = Arc::new(AnthropicProvider::new(api_key)?);

    let runtime = AgentRuntime::builder().provider(provider).build()?;

    Ok(Arc::new(runtime))
}
