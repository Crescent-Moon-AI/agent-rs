# Available MCP Servers

A curated list of Model Context Protocol (MCP) servers that can be used with agent-rs.

## Official MCP Servers

These servers are maintained by the MCP team and provide well-tested functionality.

### 1. Filesystem Server

Access local filesystem with safety controls.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-filesystem
```

**Configuration**:
```json
{
  "filesystem": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/allowed/directory"
    ],
    "env": {}
  }
}
```

**Features**:
- Read/write files
- List directories
- File search
- Safe path validation

**Tools**:
- `read_file` - Read file contents
- `write_file` - Write to file
- `list_directory` - List directory contents
- `search_files` - Search for files

**Resources**:
- `file://` URIs for file access

---

### 2. GitHub Server

Interact with GitHub repositories, issues, and PRs.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-github
```

**Configuration**:
```json
{
  "github": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-github"],
    "env": {
      "GITHUB_TOKEN": "${GITHUB_TOKEN}"
    }
  }
}
```

**Features**:
- Repository operations
- Issue management
- Pull request handling
- Code search

**Tools**:
- `create_issue` - Create GitHub issue
- `list_issues` - List repository issues
- `search_code` - Search code in repositories
- `create_pull_request` - Create PR

**Resources**:
- `github://` URIs for repository content

---

### 3. Google Drive Server

Access and manage Google Drive files.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-gdrive
```

**Configuration**:
```json
{
  "gdrive": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-gdrive"],
    "env": {
      "GOOGLE_API_KEY": "${GOOGLE_API_KEY}"
    }
  }
}
```

**Features**:
- List Drive files
- Read documents
- Upload files
- Share management

---

### 4. Slack Server

Interact with Slack workspaces.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-slack
```

**Configuration**:
```json
{
  "slack": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-slack"],
    "env": {
      "SLACK_BOT_TOKEN": "${SLACK_BOT_TOKEN}",
      "SLACK_TEAM_ID": "${SLACK_TEAM_ID}"
    }
  }
}
```

**Features**:
- Send messages
- Read channels
- User management
- File uploads

---

### 5. PostgreSQL Server

Query and manage PostgreSQL databases.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-postgres
```

**Configuration**:
```json
{
  "postgres": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-postgres"],
    "env": {
      "POSTGRES_CONNECTION_STRING": "${DATABASE_URL}"
    }
  }
}
```

**Features**:
- Execute SQL queries
- Schema inspection
- Table operations
- Safe query validation

---

### 6. Browser Automation Server

Control web browsers for automation.

**Installation**:
```bash
npm install -g @modelcontextprotocol/server-puppeteer
```

**Configuration**:
```json
{
  "browser": {
    "type": "stdio",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-puppeteer"],
    "env": {}
  }
}
```

**Features**:
- Navigate pages
- Click elements
- Extract content
- Take screenshots

---

## Community Servers

Third-party MCP servers built by the community.

### AWS Services Server

Interact with AWS services.

**Repository**: https://github.com/example/mcp-aws

**Configuration**:
```json
{
  "aws": {
    "type": "stdio",
    "command": "mcp-aws-server",
    "env": {
      "AWS_REGION": "us-east-1",
      "AWS_ACCESS_KEY_ID": "${AWS_ACCESS_KEY_ID}",
      "AWS_SECRET_ACCESS_KEY": "${AWS_SECRET_ACCESS_KEY}"
    }
  }
}
```

---

### Docker Server

Manage Docker containers and images.

**Repository**: https://github.com/example/mcp-docker

**Configuration**:
```json
{
  "docker": {
    "type": "stdio",
    "command": "mcp-docker-server",
    "env": {}
  }
}
```

---

## HTTP/SSE Servers

These servers run as standalone HTTP services.

### Custom API Server

Example HTTP MCP server for custom APIs.

**Configuration**:
```json
{
  "api": {
    "type": "http",
    "url": "http://localhost:3000/mcp",
    "headers": {
      "Authorization": "Bearer ${API_TOKEN}",
      "X-API-Version": "2024-11-05"
    },
    "timeoutSecs": 30
  }
}
```

---

## Creating Your Own MCP Server

### Using the MCP SDK

Create a custom MCP server in Node.js:

```javascript
import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

// Create server
const server = new Server(
  {
    name: "my-custom-server",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
      resources: {},
    },
  }
);

// Register tools
server.setRequestHandler("tools/list", async () => {
  return {
    tools: [
      {
        name: "my_tool",
        description: "My custom tool",
        inputSchema: {
          type: "object",
          properties: {
            input: { type: "string" },
          },
        },
      },
    ],
  };
});

server.setRequestHandler("tools/call", async (request) => {
  const { name, arguments: args } = request.params;

  if (name === "my_tool") {
    // Tool implementation
    return {
      content: [
        {
          type: "text",
          text: `Processed: ${args.input}`,
        },
      ],
    };
  }

  throw new Error(`Unknown tool: ${name}`);
});

// Start server
const transport = new StdioServerTransport();
await server.connect(transport);
```

### Using the Rust MCP SDK

Create a custom MCP server in Rust:

```rust
use rust_mcp_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server
    let server = MCPServer::new(
        "my-custom-server",
        "1.0.0",
        MCPServerCapabilities {
            tools: Some(ToolsCapability { list_changed: false }),
            resources: None,
            prompts: None,
        },
    );

    // Register tool handlers
    server.register_tool("my_tool", |args| async move {
        // Tool implementation
        Ok(MCPToolResult {
            content: vec![MCPContent::Text {
                text: format!("Processed: {:?}", args),
            }],
            is_error: None,
        })
    });

    // Start stdio transport
    server.run_stdio().await?;

    Ok(())
}
```

## Configuration Examples

### Multi-Server Setup

Configure multiple servers for different capabilities:

```json
{
  "mcpServers": {
    "filesystem": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    },
    "github": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    },
    "api": {
      "type": "http",
      "url": "http://localhost:3000/mcp",
      "headers": {
        "Authorization": "Bearer ${API_TOKEN}"
      },
      "timeoutSecs": 30
    }
  },
  "agentConfigurations": {
    "code-assistant": {
      "mcpServers": ["filesystem", "github"],
      "tools": {
        "allow": ["read_*", "list_*", "search_*"],
        "deny": ["write_*", "delete_*"]
      }
    },
    "api-agent": {
      "mcpServers": ["api"],
      "tools": {
        "allow": ["*"]
      }
    }
  }
}
```

### Development vs Production

Use different configurations for development and production:

**`.mcp.json`** (project, version controlled):
```json
{
  "mcpServers": {
    "dev-server": {
      "type": "stdio",
      "command": "npm",
      "args": ["run", "dev:mcp"],
      "cwd": "./mcp-server"
    }
  }
}
```

**`~/.config/agent-rs/mcp.json`** (user, personal):
```json
{
  "mcpServers": {
    "prod-api": {
      "type": "http",
      "url": "https://api.production.com/mcp",
      "headers": {
        "Authorization": "Bearer ${PROD_API_TOKEN}"
      }
    }
  },
  "agentConfigurations": {
    "production-agent": {
      "mcpServers": ["prod-api"]
    }
  }
}
```

## Security Considerations

### 1. Environment Variables

Always use environment variables for sensitive data:

```json
{
  "env": {
    "API_KEY": "${MY_API_KEY}",  // ✓ Good
    "API_KEY": "sk-1234567890"    // ✗ Bad - hardcoded secret
  }
}
```

### 2. Filesystem Access

Limit filesystem server to specific directories:

```json
{
  "filesystem": {
    "type": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/workspace",  // Only allow access to /workspace
      "/data"        // and /data directories
    ]
  }
}
```

### 3. Tool Filtering

Use deny lists to prevent dangerous operations:

```json
{
  "tools": {
    "allow": ["*"],
    "deny": [
      "delete_*",
      "drop_*",
      "truncate_*",
      "*_admin",
      "execute_shell"
    ]
  }
}
```

### 4. Resource Filtering

Protect sensitive files and directories:

```json
{
  "resources": {
    "allow": ["file://**"],
    "deny": [
      "file://**/.env",
      "file://**/.env.*",
      "file://**/secrets/**",
      "file://**/private/**",
      "file://**/*.key",
      "file://**/*.pem"
    ]
  }
}
```

## Testing MCP Servers

### Manual Testing

Test an MCP server manually:

```bash
# Start the server
npx -y @modelcontextprotocol/server-filesystem /tmp

# In another terminal, use the MCP inspector
npx @modelcontextprotocol/inspector npx -y @modelcontextprotocol/server-filesystem /tmp
```

### Automated Testing with agent-rs

```rust
use agent_mcp::{MCPClientManager, MCPConfig};

#[tokio::test]
async fn test_mcp_server() {
    let mut config = MCPConfig::default();

    // Configure test server
    config.mcp_servers.insert(
        "test-server".to_string(),
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

    let manager = MCPClientManager::new(Arc::new(config), "test-agent".to_string());
    manager.initialize().await.expect("Failed to initialize");

    // Test tool discovery
    let tools = manager.discover_tools().await.expect("Failed to discover tools");
    assert!(!tools.is_empty());

    // Test tool call
    let args = serde_json::json!({ "path": "/tmp/test.txt" });
    let result = manager.call_tool("test-server", "read_file", args).await;
    assert!(result.is_ok() || matches!(result, Err(MCPError::ToolCallFailed(_))));
}
```

## Resources

- **MCP Specification**: https://modelcontextprotocol.io
- **Official Servers**: https://github.com/modelcontextprotocol/servers
- **Server Template**: https://github.com/modelcontextprotocol/create-server
- **MCP Inspector**: https://github.com/modelcontextprotocol/inspector
- **Community Servers**: https://github.com/topics/mcp-server

## Contributing

To add your MCP server to this list:

1. Fork this repository
2. Add your server information to this document
3. Include:
   - Server name and description
   - Installation instructions
   - Configuration example
   - Features and tools list
   - Repository link
4. Submit a pull request

Please ensure your server:
- Implements MCP protocol version 2024-11-05 or later
- Includes comprehensive documentation
- Has example usage
- Handles errors gracefully
- Includes tests
