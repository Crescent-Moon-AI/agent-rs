//! Resource management example for agent-mcp
//!
//! This example demonstrates:
//! - Working with MCP resources
//! - Resource caching
//! - Resource filtering
//! - Context integration
//!
//! Run with: cargo run --example resource_example

use agent_mcp::{MCPClientManager, MCPConfig, MCPContext, MCPContextExt, ResourceFilter};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("=== Agent MCP Resource Management Example ===\n");

    // 1. Setup
    println!("1. Setting up MCP context...");
    let config = create_config_with_resources();
    let manager = Arc::new(MCPClientManager::new(
        Arc::new(config),
        "resource-example".to_string(),
    ));

    manager.initialize().await?;

    if !manager.has_connections().await {
        println!("   ⚠ No MCP servers connected");
        println!("   This example requires MCP servers with resource support.\n");
        return Ok(());
    }

    // Create MCP context
    let context = MCPContext::new(manager.clone());
    println!("   ✓ Context created\n");

    // 2. Resource discovery
    println!("2. Discovering resources...");
    match context.discover_resources().await {
        Ok(resources) => {
            println!("   ✓ Found {} resource(s):", resources.len());
            for resource in resources.iter().take(3) {
                println!("     - {} ({})", resource.name, resource.uri);
                if let Some(mime) = &resource.mime_type {
                    println!("       Type: {}", mime);
                }
            }
        }
        Err(e) => println!("   ⚠ Discovery failed: {}", e),
    }
    println!();

    // 3. Resource filtering
    println!("3. Resource filtering examples...\n");

    // Allow all resources
    let filter_all = ResourceFilter::allow_all();
    println!("   Allow all filter created");

    // Allow specific patterns
    let filter_docs = ResourceFilter::new(
        vec!["file://docs/**".to_string(), "file://*.md".to_string()],
        vec![],
    );
    println!("   Docs filter created");

    // Deny sensitive files
    let filter_safe = ResourceFilter::new(
        vec!["file://**".to_string()],
        vec![
            "file://**/.env".to_string(),
            "file://**/secrets/**".to_string(),
        ],
    );
    println!("   Safe filter created\n");

    // Test filtering
    let test_uris = vec![
        "file://docs/README.md",
        "file://src/main.rs",
        "file://.env",
        "file://secrets/api_key.txt",
    ];

    for uri in test_uris {
        let allowed = filter_safe.should_include(uri);
        let icon = if allowed { "✓" } else { "✗" };
        println!(
            "   {} {}: {}",
            icon,
            uri,
            if allowed { "allowed" } else { "denied" }
        );
    }
    println!();

    // 4. Loading resources with caching
    println!("4. Loading resources (with caching)...");

    // First load - fetches from server
    let uri = "file://example.txt";
    match context.load_resource(uri).await {
        Ok(resource) => {
            println!(
                "   ✓ Loaded: {} ({} bytes)",
                resource.uri,
                resource.content.len()
            );
            println!("     MIME type: {:?}", resource.mime_type);
        }
        Err(e) => println!("   ⚠ Failed to load: {}", e),
    }

    // Second load - served from cache
    match context.load_resource(uri).await {
        Ok(resource) => {
            println!("   ✓ Loaded from cache: {}", resource.uri);
        }
        Err(e) => println!("   ⚠ Failed to load: {}", e),
    }
    println!();

    // 5. Batch loading
    println!("5. Batch loading multiple resources...");
    let uris = vec![
        "file://example1.txt".to_string(),
        "file://example2.txt".to_string(),
        "file://example3.txt".to_string(),
    ];

    match context.load_resources(&uris).await {
        Ok(resources) => {
            println!("   ✓ Loaded {} resource(s)", resources.len());
            for resource in resources {
                println!("     - {}", resource.uri);
            }
        }
        Err(e) => println!("   ⚠ Batch load failed: {}", e),
    }
    println!();

    // 6. Cache management
    println!("6. Cache management...");
    context.clear_resources().await?;
    println!("   ✓ Cache cleared\n");

    // 7. Cleanup
    println!("7. Cleaning up...");
    manager.shutdown().await?;
    println!("   ✓ Shutdown complete\n");

    println!("=== Example completed ===");
    Ok(())
}

fn create_config_with_resources() -> MCPConfig {
    use agent_mcp::{AgentMCPConfig, MCPServerConfig, ResourceFilterConfig};
    use std::collections::HashMap;

    let mut config = MCPConfig::default();

    config.mcp_servers.insert(
        "filesystem".to_string(),
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

    config.agent_configurations.insert(
        "resource-example".to_string(),
        AgentMCPConfig {
            mcp_servers: vec!["filesystem".to_string()],
            tools: Default::default(),
            resources: ResourceFilterConfig {
                allow: vec!["file://**".to_string()],
                deny: vec!["file://**/.env".to_string()],
            },
        },
    );

    config
}
