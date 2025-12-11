# agent-mcp

Model Context Protocol (MCP) integration for agent-rs.

## Status: Phase 1-7 Complete ✅

**Phase 1 (Foundation)** - Completed:
- ✅ Crate structure created with proper dependencies
- ✅ Configuration system implemented with `.mcp.json` support
- ✅ Error types defined
- ✅ Environment variable resolution (${VAR} and $VAR syntax)
- ✅ Per-agent configuration with tool filtering
- ✅ Comprehensive unit tests (7 config tests passing)

**Phase 2 (Client Architecture)** - Completed:
- ✅ MCPClient trait abstraction defined
- ✅ Stdio client skeleton implemented
- ✅ HTTP/SSE client skeleton implemented
- ✅ MCPClientManager with multi-server coordination
- ✅ Graceful degradation and timeout handling
- ✅ All type definitions for MCP protocol (tools, resources, prompts)
- ✅ Manager tests (9/9 total tests passing)

**Phase 3 (Tool Integration)** - Completed:
- ✅ MCPTool wrapper implementing agent-rs Tool trait
- ✅ Tool discovery and filtering logic
- ✅ JSON Schema helpers for tool definitions
- ✅ Content type conversion (text, image, resource)
- ✅ Tool execution tests (24/24 tests passing)

**Phase 4 (Runtime Integration)** - Completed:
- ✅ MCP support integrated into AgentRuntime
- ✅ `create_tool_agent_with_mcp()` factory method
- ✅ Automatic MCP tool discovery on agent creation
- ✅ Per-agent tool registry with MCP tools
- ✅ Graceful fallback when MCP unavailable
- ✅ Builder methods for MCP configuration
- ✅ Usage examples and documentation

**Phase 5 (Resource System)** - Completed:
- ✅ MCPResource type for representing MCP resources
- ✅ ResourceCache for caching resources with automatic fetching
- ✅ Resource discovery across all configured servers
- ✅ ResourceFilter for URI pattern matching
- ✅ MCPContextExt trait for context integration
- ✅ MCPContext wrapper for standalone use
- ✅ Resource tests (32/32 tests passing)

**Phase 7 (Transport & Lifecycle)** - Completed:
- ✅ Stdio transport with full subprocess communication
- ✅ HTTP/SSE transport with JSON-RPC over HTTP
- ✅ Request/response handling with error checking
- ✅ Protocol handshake (initialize/initialized)
- ✅ **Retry logic with exponential backoff**
- ✅ Configurable retry policies (max attempts, backoff timing)
- ✅ Retryable vs non-retryable error classification
- ✅ **Client lifecycle management**
- ✅ Graceful connection/disconnection
- ✅ Reconnection support for transient failures
- ✅ Health check for connection status
- ✅ Graceful degradation when servers fail
- ✅ Transport tests (49/49 tests passing)

## Features Implemented

### Configuration System

The configuration system supports:
- **Project-level configuration**: `.mcp.json` (version controlled)
- **User-level configuration**: `~/.config/agent-rs/mcp.json` (personal)
- **Merged configurations**: Project settings override user settings
- **Multiple transports**: stdio (local), HTTP, and SSE (remote)
- **Per-agent configuration**: Different agents can use different MCP servers
- **Tool filtering**: Allow/deny lists for fine-grained control
- **Resource filtering**: Control resource access per agent
- **Environment variable expansion**: `${VAR}` and `$VAR` syntax

### Example Configuration

```json
{
  "mcpServers": {
    "filesystem": {
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "env": {"LOG_LEVEL": "info"}
    },
    "github": {
      "transport": "http",
      "url": "http://localhost:8080/mcp",
      "headers": {"Authorization": "Bearer ${GITHUB_TOKEN}"}
    }
  },
  "agentConfigurations": {
    "file-assistant": {
      "mcpServers": ["filesystem"],
      "tools": {
        "allow": ["read_file", "write_file", "list_directory"],
        "deny": ["delete_file"]
      }
    },
    "researcher": {
      "mcpServers": ["filesystem", "github"],
      "tools": {"allow": "*"}
    },
    "default": {
      "mcpServers": ["filesystem"],
      "tools": {"allow": "*"}
    }
  }
}
```

## Usage

### Basic Configuration Loading

```rust
use agent_mcp::config::MCPConfig;

// Load merged configuration (user + project)
let config = MCPConfig::load_merged()?;

// Get configuration for a specific agent
if let Some(agent_config) = config.get_agent_config("file-assistant") {
    println!("Agent uses {} MCP servers", agent_config.mcp_servers.len());
}

// Load from a specific file
let config = MCPConfig::from_file(".mcp.json")?;

// Save configuration
config.save_to_file(".mcp.json")?;
```

### Runtime Integration (Recommended)

```rust
use agent_runtime::{AgentRuntime, ExecutorConfig};
use std::path::PathBuf;
use std::sync::Arc;

// Create runtime with MCP support
let runtime = AgentRuntime::builder()
    .provider(llm_provider)
    .mcp_config_from_file(PathBuf::from(".mcp.json"))?
    .build()?;

// Create agent with automatic MCP tool discovery
let agent = runtime
    .create_tool_agent_with_mcp(
        ExecutorConfig::default(),
        "research-agent" // Must match agent name in .mcp.json
    )
    .await?;

// The agent now has access to all MCP tools configured for it!
// MCP tools are automatically filtered based on agent configuration
```

### Manual Tool Discovery

```rust
use agent_mcp::{MCPClientManager, discovery};
use agent_tools::ToolRegistry;
use std::sync::Arc;

// Create client manager
let config = MCPConfig::from_file(".mcp.json")?;
let manager = Arc::new(MCPClientManager::new(
    Arc::new(config.clone()),
    "research-agent".to_string()
));

// Initialize connections
manager.initialize().await?;

// Discover and register tools
let mut registry = ToolRegistry::new();
let agent_config = config.get_agent_config("research-agent").unwrap();
let tool_count = discovery::discover_and_register_tools(
    manager,
    &mut registry,
    agent_config
).await?;

println!("Registered {} MCP tools", tool_count);
```

## Dependencies

- **rust-mcp-sdk 0.2**: Official Rust SDK for Model Context Protocol
- **rust-mcp-schema 0.2**: Type-safe MCP schema implementation
- **serde/serde_json**: Configuration serialization
- **regex**: Environment variable expansion
- **thiserror**: Error handling

## Architecture

### Client Abstraction Layer

The `MCPClient` trait provides a unified interface for MCP communication:
- `connect()` / `disconnect()` - Lifecycle management
- `list_tools()` / `call_tool()` - Tool operations
- `list_resources()` / `read_resource()` - Resource access
- `list_prompts()` / `get_prompt()` - Prompt templates
- `server_info()` - Server capabilities

### Transport Implementations

- **StdioMCPClient**: Full subprocess communication with MCP servers
  - Spawns child processes with configurable command/args/env
  - JSON-RPC 2.0 over newline-delimited JSON via stdin/stdout
  - Automatic process cleanup on disconnect/drop
  - MCP protocol handshake (initialize → initialized)

- **HttpMCPClient**: HTTP-based communication with remote MCP servers
  - JSON-RPC 2.0 over HTTP POST requests
  - Configurable headers (e.g., Authorization tokens)
  - Request timeout handling
  - HTTP status code error checking
  - SSE transport support (same implementation)

### Multi-Server Management

`MCPClientManager` coordinates multiple MCP servers:
- Initializes connections to all configured servers for an agent
- Discovers tools across all connected servers
- Routes tool calls to the appropriate server
- Implements graceful degradation when servers fail

## Next Steps (Phase 6, 8+)

**Phase 6 (CLI Commands)**: Add command-line tools
- `mcp add` - Add MCP server to configuration
- `mcp list` - List configured servers and tools
- `mcp test` - Test MCP server connections
- `mcp discover` - Discover available tools

**Phase 8 (Polish)**: Production readiness
- Connection pooling for HTTP clients
- Comprehensive integration tests with real MCP servers
- Performance optimization and benchmarks
- Advanced error handling and recovery
- Documentation improvements and examples
- CLI commands for managing MCP configuration

## Testing

Run tests:
```bash
cargo test --package agent-mcp
```

All 49 unit tests are passing:
- Configuration parsing (stdio, HTTP, SSE)
- Environment variable resolution
- Tool filtering (list and wildcard)
- Configuration merging
- Agent configuration lookup
- Client manager creation and initialization
- Multi-server coordination
- **Stdio transport** (client creation, config parsing, subprocess management)
- **HTTP transport** (client creation, config parsing, header building)
- **Retry logic** (backoff calculation, retryable errors, execution with retries)
- Tool wrapper and conversion
- Tool discovery and registration
- Schema validation
- Resource caching and filtering
- Resource content extraction
- Context extension traits

Run runtime tests:
```bash
cargo test --package agent-runtime
```

All 10 runtime tests passing including MCP integration tests.

**Total**: 59 tests passing (49 agent-mcp + 10 agent-runtime)

## References

- [MCP Specification (2024-11-05)](https://modelcontextprotocol.io/specification/2024-11-05)
- [Official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [rust-mcp-sdk on crates.io](https://crates.io/crates/rust-mcp-sdk)
