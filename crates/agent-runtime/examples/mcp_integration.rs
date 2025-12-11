//! Example demonstrating MCP integration with agent-runtime
//!
//! This example shows how to:
//! 1. Load MCP configuration from a file
//! 2. Create an AgentRuntime with MCP support
//! 3. Create tool agents that automatically discover MCP tools
//!
//! To run this example:
//! ```bash
//! cargo run --example mcp_integration
//! ```

use agent_core::Result;
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
            println!(
                "  (MCP tools have been automatically discovered and registered)\n"
            );

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
    use agent_llm::LLMProvider;

    // In a real application, you would create an actual LLM provider here
    // For this example, we'll just demonstrate the structure
    struct DemoProvider;

    #[async_trait::async_trait]
    impl LLMProvider for DemoProvider {
        async fn complete(
            &self,
            _request: agent_llm::CompletionRequest,
        ) -> agent_llm::Result<agent_llm::CompletionResponse> {
            unimplemented!("This is a demo provider for example purposes only")
        }

        fn name(&self) -> &str {
            "demo"
        }
    }

    let provider = Arc::new(DemoProvider);

    // Build runtime with MCP configuration
    let runtime = AgentRuntime::builder()
        .provider(provider)
        .mcp_config_from_file(config_path)?
        .build()?;

    Ok(Arc::new(runtime))
}

/// Create an AgentRuntime without MCP support
async fn create_runtime_without_mcp() -> Result<Arc<AgentRuntime>> {
    use agent_llm::LLMProvider;

    struct DemoProvider;

    #[async_trait::async_trait]
    impl LLMProvider for DemoProvider {
        async fn complete(
            &self,
            _request: agent_llm::CompletionRequest,
        ) -> agent_llm::Result<agent_llm::CompletionResponse> {
            unimplemented!("This is a demo provider for example purposes only")
        }

        fn name(&self) -> &str {
            "demo"
        }
    }

    let provider = Arc::new(DemoProvider);

    let runtime = AgentRuntime::builder().provider(provider).build()?;

    Ok(Arc::new(runtime))
}
