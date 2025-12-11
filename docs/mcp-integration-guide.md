# MCP Integration Guide

Complete guide to using Model Context Protocol (MCP) integration in agent-rs.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Configuration](#configuration)
4. [Client Management](#client-management)
5. [Working with Tools](#working-with-tools)
6. [Working with Resources](#working-with-resources)
7. [Retry Logic](#retry-logic)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)

## Overview

The `agent-mcp` crate provides complete MCP integration for agent-rs, allowing agents to:

- **Connect to MCP servers** via stdio (local) or HTTP/SSE (remote)
- **Discover and execute tools** from multiple MCP servers
- **Access resources** with caching and filtering
- **Handle failures gracefully** with automatic retry and degradation

### Architecture

```
┌─────────────┐
│   Agent     │
└──────┬──────┘
       │
       ├─────► MCPClientManager ◄──── Configuration
       │              │
       │              ├─────► StdioMCPClient (Server 1)
       │              ├─────► HttpMCPClient (Server 2)
       │              └─────► HttpMCPClient (Server 3)
       │
       └─────► MCPContext (Resource Cache)
```

## Quick Start

### 1. Add Dependency

```toml
[dependencies]
agent-mcp = { path = "../agent-mcp" }
```

### 2. Create Configuration

Create `.mcp.json` in your project root:

```json
{
  "mcpServers": {
    "filesystem": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
      "env": {}
    },
    "github": {
      "type": "http",
      "url": "http://localhost:3000/mcp",
      "headers": {
        "Authorization": "Bearer ${GITHUB_TOKEN}"
      },
      "timeoutSecs": 30
    }
  },
  "agentConfigurations": {
    "my-agent": {
      "mcpServers": ["filesystem", "github"],
      "tools": {
        "allow": ["read_file", "list_files", "search_code"],
        "deny": ["delete_file"]
      },
      "resources": {
        "allow": ["file://**", "github://**"],
        "deny": ["file://**/.env", "file://**/secrets/**"]
      }
    }
  }
}
```

### 3. Initialize and Use

```rust
use agent_mcp::{MCPClientManager, MCPConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = MCPConfig::load_merged().await?;

    // Create manager
    let manager = MCPClientManager::new(
        Arc::new(config),
        "my-agent".to_string()
    );

    // Initialize connections
    manager.initialize().await?;

    // Discover tools
    let tools = manager.discover_tools().await?;
    println!("Available tools: {}", tools.len());

    // Call a tool
    let args = serde_json::json!({
        "path": "/tmp/example.txt"
    });
    let result = manager.call_tool("filesystem", "read_file", args).await?;

    // Cleanup
    manager.shutdown().await?;

    Ok(())
}
```

## Configuration

### Configuration File Structure

```json
{
  "mcpServers": {
    "server-name": {
      // Server configuration
    }
  },
  "agentConfigurations": {
    "agent-name": {
      "mcpServers": ["server-name"],
      "tools": { /* tool filters */ },
      "resources": { /* resource filters */ }
    }
  }
}
```

### Server Types

#### Stdio (Local Process)

```json
{
  "type": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-example"],
  "env": {
    "API_KEY": "${MY_API_KEY}"
  },
  "cwd": "/path/to/working/dir"
}
```

#### HTTP/SSE (Remote)

```json
{
  "type": "http",
  "url": "http://localhost:3000/mcp",
  "headers": {
    "Authorization": "Bearer ${TOKEN}",
    "X-Custom-Header": "value"
  },
  "timeoutSecs": 30
}
```

### Environment Variables

Use `${VAR}` or `$VAR` syntax for environment variable substitution:

```json
{
  "env": {
    "API_KEY": "${OPENAI_API_KEY}",
    "DEBUG": "$DEBUG_MODE"
  }
}
```

### Tool Filtering

Control which tools agents can access:

```json
{
  "tools": {
    "allow": ["read_*", "list_*", "search"],  // Wildcards supported
    "deny": ["delete_*", "admin_*"]           // Deny takes precedence
  }
}
```

### Resource Filtering

Control which resources agents can access:

```json
{
  "resources": {
    "allow": ["file://**", "github://myorg/**"],
    "deny": ["file://**/.env", "file://**/secrets/**"]
  }
}
```

## Client Management

### Creating a Manager

```rust
use agent_mcp::{MCPClientManager, MCPConfig};
use std::sync::Arc;

let config = MCPConfig::load_merged().await?;
let manager = MCPClientManager::new(
    Arc::new(config),
    "agent-name".to_string()
);
```

### Initialization

```rust
// Initialize with automatic retry
manager.initialize().await?;

// Check connection status
if manager.has_connections().await {
    let servers = manager.connected_servers().await;
    println!("Connected to: {:?}", servers);
}
```

### Health Monitoring

```rust
// Check health of all servers
let health = manager.health_check().await;
for (server, is_healthy) in health {
    println!("{}: {}", server, if is_healthy { "✓" } else { "✗" });
}
```

### Reconnection

```rust
// Reconnect to a specific server
match manager.reconnect("server-name").await {
    Ok(_) => println!("Reconnected successfully"),
    Err(e) => println!("Reconnection failed: {}", e),
}
```

### Shutdown

```rust
// Gracefully disconnect from all servers
manager.shutdown().await?;
```

## Working with Tools

### Discovering Tools

```rust
// Discover all tools from all connected servers
let tools = manager.discover_tools().await?;

for tool_info in tools {
    println!("Tool: {} (from {})",
        tool_info.definition.name,
        tool_info.server_name
    );

    if let Some(desc) = &tool_info.definition.description {
        println!("  Description: {}", desc);
    }

    println!("  Schema: {}", tool_info.definition.input_schema);
}
```

### Calling Tools

```rust
use serde_json::json;

// Prepare arguments
let args = json!({
    "path": "/tmp/example.txt",
    "encoding": "utf-8"
});

// Call the tool
let result = manager.call_tool(
    "filesystem",      // server name
    "read_file",       // tool name
    args
).await?;

// Process result
for content in result.content {
    match content {
        MCPContent::Text { text } => println!("Text: {}", text),
        MCPContent::Image { data, mime_type } => {
            println!("Image: {} bytes ({})", data.len(), mime_type);
        }
        MCPContent::Resource { uri, .. } => {
            println!("Resource: {}", uri);
        }
    }
}

// Check for errors
if result.is_error.unwrap_or(false) {
    eprintln!("Tool returned an error");
}
```

### Integrating with agent-rs Tools

```rust
use agent_mcp::MCPTool;
use agent_tools::ToolRegistry;

// Wrap MCP tool for use with agent-rs
let mcp_tool = MCPTool::new(
    tool_info.definition,
    tool_info.server_name,
    manager.clone()
);

// Register with tool registry
let mut registry = ToolRegistry::new();
registry.register(Box::new(mcp_tool));
```

## Working with Resources

### Creating Context

```rust
use agent_mcp::{MCPContext, MCPContextExt};

let context = MCPContext::new(manager.clone());
```

### Discovering Resources

```rust
// Discover all available resources
let resources = context.discover_resources().await?;

for resource in resources {
    println!("Resource: {} ({})", resource.name, resource.uri);
    if let Some(mime) = &resource.mime_type {
        println!("  Type: {}", mime);
    }
}
```

### Loading Resources

```rust
// Load a single resource (with caching)
let resource = context.load_resource("file:///tmp/example.txt").await?;

println!("URI: {}", resource.uri);
println!("Content: {} bytes", resource.content.len());
println!("Type: {:?}", resource.content_type);

// Access content as text
if let Some(text) = resource.text() {
    println!("Text content:\n{}", text);
}
```

### Batch Loading

```rust
// Load multiple resources efficiently
let uris = vec![
    "file:///tmp/file1.txt".to_string(),
    "file:///tmp/file2.txt".to_string(),
    "file:///tmp/file3.txt".to_string(),
];

let resources = context.load_resources(&uris).await?;
println!("Loaded {} resources", resources.len());
```

### Resource Filtering

```rust
use agent_mcp::ResourceFilter;

// Create filter
let filter = ResourceFilter::new(
    vec!["file://**".to_string()],           // allow
    vec!["file://**/.env".to_string()]       // deny
);

// Test URIs
if filter.should_include("file:///tmp/data.txt") {
    println!("URI is allowed");
}

if !filter.should_include("file:///tmp/.env") {
    println!("URI is denied");
}
```

### Cache Management

```rust
// Clear the resource cache
context.clear_resources().await?;
```

## Retry Logic

### Default Retry Policy

By default, MCP clients use this retry policy:
- **Max attempts**: 3
- **Initial backoff**: 100ms
- **Max backoff**: 10s
- **Multiplier**: 2.0 (exponential)

### Custom Retry Policy

```rust
use agent_mcp::RetryPolicy;
use std::time::Duration;

// Create custom policy
let policy = RetryPolicy::new(
    5,                              // max attempts
    Duration::from_millis(200),     // initial backoff
    Duration::from_secs(30),        // max backoff
    2.0                             // multiplier
);

// Apply to client
let client = StdioMCPClient::new(/* ... */)
    .with_retry_policy(policy);
```

### Retry Policies for Different Scenarios

```rust
// Fast retry (for testing)
let fast = RetryPolicy::fast();  // 3 attempts, 10-100ms

// No retry (fail immediately)
let no_retry = RetryPolicy::no_retry();  // 1 attempt

// Aggressive retry (unreliable networks)
let aggressive = RetryPolicy::new(
    10,                             // 10 attempts
    Duration::from_secs(1),         // 1s initial
    Duration::from_secs(60),        // 60s max
    2.0
);
```

### Manual Retry Execution

```rust
let policy = RetryPolicy::default();

let result = policy.execute("my_operation", || async {
    // Your operation here
    call_mcp_server().await
}).await?;
```

### Retryable vs Non-Retryable Errors

Automatically retried:
- `MCPError::ConnectionFailed`
- `MCPError::RequestFailed`
- `MCPError::NotConnected`

Not retried (fail immediately):
- `MCPError::ConfigError`
- `MCPError::InvalidUri`
- `MCPError::ServerNotFound`
- `MCPError::ToolCallFailed`

## Best Practices

### 1. Configuration Management

```rust
// Load merged configuration (project + user)
let config = MCPConfig::load_merged().await?;

// Or load from specific file
let config = MCPConfig::from_file(".mcp.json").await?;

// Always use Arc for shared access
let config = Arc::new(config);
```

### 2. Error Handling

```rust
// Graceful degradation
match manager.initialize().await {
    Ok(_) => println!("All servers connected"),
    Err(e) => {
        eprintln!("Some servers failed: {}", e);
        // Continue anyway - manager uses graceful degradation
    }
}

// Check connection before operations
if !manager.has_connections().await {
    return Err("No MCP servers available".into());
}
```

### 3. Resource Management

```rust
// Always shutdown gracefully
struct MyAgent {
    mcp_manager: MCPClientManager,
}

impl Drop for MyAgent {
    fn drop(&mut self) {
        // Note: Drop can't be async, so use blocking
        // Better to call shutdown() explicitly before drop
    }
}

// Explicit shutdown
async fn cleanup(agent: &MyAgent) {
    if let Err(e) = agent.mcp_manager.shutdown().await {
        eprintln!("Shutdown error: {}", e);
    }
}
```

### 4. Tool Discovery and Caching

```rust
// Discover tools once at startup
let tools = manager.discover_tools().await?;

// Cache tool definitions
let tool_map: HashMap<String, MCPToolInfo> = tools
    .into_iter()
    .map(|t| (t.definition.name.clone(), t))
    .collect();

// Reuse cached definitions
if let Some(tool) = tool_map.get("read_file") {
    // Use tool
}
```

### 5. Resource Caching

```rust
// MCPContext automatically caches resources
let context = MCPContext::new(manager);

// First load - fetches from server
let resource1 = context.load_resource("file:///data.txt").await?;

// Second load - served from cache (fast)
let resource2 = context.load_resource("file:///data.txt").await?;

// Clear cache when needed
context.clear_resources().await?;
```

### 6. Health Monitoring

```rust
use tokio::time::{interval, Duration};

// Periodic health check
let mut health_check = interval(Duration::from_secs(60));

loop {
    tokio::select! {
        _ = health_check.tick() => {
            let health = manager.health_check().await;
            for (server, is_healthy) in health {
                if !is_healthy {
                    eprintln!("Server {} is unhealthy", server);
                    // Attempt reconnection
                    let _ = manager.reconnect(&server).await;
                }
            }
        }
    }
}
```

## Troubleshooting

### Connection Failures

**Problem**: `MCPError::ConnectionFailed`

**Solutions**:
1. Check server is running: `ps aux | grep mcp-server`
2. Verify command/URL in configuration
3. Check environment variables are set
4. Review logs with `RUST_LOG=debug`
5. Increase timeout in configuration

### Tool Not Found

**Problem**: Tool doesn't appear in discovery

**Solutions**:
1. Check tool filtering in agent configuration
2. Verify server supports tools capability
3. List tools directly: `manager.get_client("server")?.list_tools()`

### Resource Access Denied

**Problem**: Resource cannot be loaded

**Solutions**:
1. Check resource filtering rules
2. Verify URI format (e.g., `file://` prefix)
3. Check server permissions
4. Review deny patterns in configuration

### High Memory Usage

**Problem**: Resource cache growing too large

**Solutions**:
1. Clear cache periodically: `context.clear_resources()`
2. Use resource filtering to limit cached items
3. Load resources on-demand instead of batch loading

### Slow Performance

**Problem**: Operations taking too long

**Solutions**:
1. Use fast retry policy for local servers
2. Increase timeout for slow servers
3. Enable HTTP/2 for HTTP servers
4. Use connection pooling (planned for Phase 8)

### Debug Logging

Enable detailed logging:

```bash
RUST_LOG=debug cargo run
RUST_LOG=agent_mcp=trace cargo run  # MCP-specific logs only
```

In code:

```rust
tracing_subscriber::fmt()
    .with_env_filter("debug")
    .init();
```

## Additional Resources

- [MCP Specification](https://modelcontextprotocol.io/specification/2024-11-05)
- [MCP Server Registry](https://github.com/modelcontextprotocol/servers)
- [agent-mcp README](../crates/agent-mcp/README.md)
- [Examples](../crates/agent-mcp/examples/)

## Support

For issues and questions:
- GitHub Issues: https://github.com/your-org/agent-rs/issues
- Documentation: https://docs.rs/agent-mcp
