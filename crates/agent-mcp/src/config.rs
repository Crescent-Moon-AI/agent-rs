//! Configuration types for MCP integration
//!
//! Supports project-level (`.mcp.json`) and user-level (`~/.config/agent-rs/mcp.json`)
//! configuration files with merge support.

use crate::error::MCPError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root MCP configuration
///
/// This is the top-level configuration structure loaded from `.mcp.json` files.
///
/// # Example
///
/// ```json
/// {
///   "mcpServers": {
///     "filesystem": {
///       "transport": "stdio",
///       "command": "npx",
///       "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
///     }
///   },
///   "agentConfigurations": {
///     "file-assistant": {
///       "mcpServers": ["filesystem"],
///       "tools": {"allow": "*"}
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MCPConfig {
    /// MCP server definitions
    #[serde(default)]
    pub mcp_servers: HashMap<String, MCPServerConfig>,

    /// Per-agent configurations
    #[serde(default)]
    pub agent_configurations: HashMap<String, AgentMCPConfig>,
}

/// MCP server configuration
///
/// Supports multiple transport types: stdio, HTTP, and SSE.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "lowercase")]
pub enum MCPServerConfig {
    /// Stdio transport (for local subprocess MCP servers)
    Stdio {
        /// Command to execute
        command: String,

        /// Command arguments
        #[serde(default)]
        args: Vec<String>,

        /// Environment variables
        #[serde(default)]
        env: HashMap<String, String>,

        /// Working directory (optional)
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<PathBuf>,
    },

    /// HTTP transport
    #[serde(rename = "http")]
    Http {
        /// Server URL
        url: String,

        /// HTTP headers
        #[serde(default)]
        headers: HashMap<String, String>,

        /// Timeout in seconds
        #[serde(default = "default_timeout")]
        timeout_secs: u64,
    },

    /// SSE (Server-Sent Events) transport
    #[serde(rename = "sse")]
    Sse {
        /// Server URL
        url: String,

        /// HTTP headers
        #[serde(default)]
        headers: HashMap<String, String>,

        /// Timeout in seconds
        #[serde(default = "default_timeout")]
        timeout_secs: u64,
    },
}

/// Per-agent MCP configuration
///
/// Specifies which MCP servers an agent can use and which tools/resources are allowed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentMCPConfig {
    /// MCP servers this agent can use (references keys in mcp_servers)
    pub mcp_servers: Vec<String>,

    /// Tool filtering configuration
    #[serde(default)]
    pub tools: ToolFilter,

    /// Resource filtering configuration
    #[serde(default)]
    pub resources: ResourceFilter,
}

/// Tool filtering configuration
///
/// Supports allow-listing and deny-listing of tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFilter {
    /// Allowed tools ("*" for all, or list of tool names)
    #[serde(default = "default_allow_all")]
    pub allow: ToolPattern,

    /// Denied tools (overrides allow list)
    #[serde(default)]
    pub deny: Vec<String>,
}

impl Default for ToolFilter {
    fn default() -> Self {
        Self {
            allow: default_allow_all(),
            deny: Vec::new(),
        }
    }
}

/// Tool pattern specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolPattern {
    /// Allow all tools
    All(String), // Must be "*"

    /// Allow specific tools by name
    List(Vec<String>),
}

/// Resource filtering configuration
///
/// Controls which resources an agent can access.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceFilter {
    /// Allowed resource URI patterns
    #[serde(default = "default_allow_all_resources")]
    pub allow: Vec<String>,

    /// Denied resource URI patterns
    #[serde(default)]
    pub deny: Vec<String>,
}

// Default functions for serde
fn default_timeout() -> u64 {
    30
}

fn default_allow_all() -> ToolPattern {
    ToolPattern::All("*".to_string())
}

fn default_allow_all_resources() -> Vec<String> {
    vec!["*".to_string()]
}

impl MCPConfig {
    /// Load configuration from a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use agent_mcp::config::MCPConfig;
    /// let config = MCPConfig::from_file(".mcp.json")?;
    /// # Ok::<(), agent_mcp::error::MCPError>(())
    /// ```
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, MCPError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| MCPError::ConfigError(format!("Failed to read config file: {}", e)))?;

        let mut config: MCPConfig = serde_json::from_str(&content)
            .map_err(|e| MCPError::ConfigError(format!("Failed to parse config file: {}", e)))?;

        // Resolve environment variables
        config.resolve_env_vars()?;

        Ok(config)
    }

    /// Load merged configuration (user + project)
    ///
    /// Loads the user-level config from `~/.config/agent-rs/mcp.json` and merges it
    /// with the project-level config from `.mcp.json`. Project-level settings
    /// take precedence.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use agent_mcp::config::MCPConfig;
    /// let config = MCPConfig::load_merged()?;
    /// # Ok::<(), agent_mcp::error::MCPError>(())
    /// ```
    pub fn load_merged() -> Result<Self, MCPError> {
        let mut config = Self::load_user_config().unwrap_or_default();

        if let Ok(project_config) = Self::load_project_config() {
            config.merge(project_config);
        }

        Ok(config)
    }

    /// Load user-level config from `~/.config/agent-rs/mcp.json`
    pub fn load_user_config() -> Result<Self, MCPError> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| MCPError::ConfigError("HOME or USERPROFILE not set".to_string()))?;

        let path = PathBuf::from(home)
            .join(".config")
            .join("agent-rs")
            .join("mcp.json");

        Self::from_file(path)
    }

    /// Load project-level config from `.mcp.json`
    pub fn load_project_config() -> Result<Self, MCPError> {
        Self::from_file(".mcp.json")
    }

    /// Merge another config into this one
    ///
    /// The `other` config's values take precedence over this config's values.
    pub fn merge(&mut self, other: MCPConfig) {
        self.mcp_servers.extend(other.mcp_servers);
        self.agent_configurations.extend(other.agent_configurations);
    }

    /// Get configuration for a specific agent
    ///
    /// Returns the agent-specific configuration, or the "default" configuration
    /// if no specific configuration exists.
    ///
    /// # Arguments
    ///
    /// * `agent_name` - Name of the agent
    pub fn get_agent_config(&self, agent_name: &str) -> Option<&AgentMCPConfig> {
        self.agent_configurations
            .get(agent_name)
            .or_else(|| self.agent_configurations.get("default"))
    }

    /// Resolve environment variables in configuration
    ///
    /// Supports `${VAR}` and `$VAR` syntax for environment variable expansion.
    pub fn resolve_env_vars(&mut self) -> Result<(), MCPError> {
        for server_config in self.mcp_servers.values_mut() {
            match server_config {
                MCPServerConfig::Stdio {
                    command,
                    args,
                    env,
                    cwd,
                } => {
                    *command = resolve_env_string(command)?;

                    for arg in args.iter_mut() {
                        *arg = resolve_env_string(arg)?;
                    }

                    for value in env.values_mut() {
                        *value = resolve_env_string(value)?;
                    }

                    if let Some(path) = cwd {
                        let path_str = path.to_string_lossy().to_string();
                        let resolved = resolve_env_string(&path_str)?;
                        *path = PathBuf::from(resolved);
                    }
                }
                MCPServerConfig::Http {
                    url,
                    headers,
                    timeout_secs: _,
                }
                | MCPServerConfig::Sse {
                    url,
                    headers,
                    timeout_secs: _,
                } => {
                    *url = resolve_env_string(url)?;

                    for value in headers.values_mut() {
                        *value = resolve_env_string(value)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Save configuration to a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the configuration file
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), MCPError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| MCPError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path.as_ref(), json)
            .map_err(|e| MCPError::ConfigError(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

/// Resolve environment variable references in strings
///
/// Supports `${VAR}` and `$VAR` syntax.
///
/// # Example
///
/// ```
/// # use agent_mcp::config::resolve_env_string;
/// std::env::set_var("TEST_VAR", "test_value");
/// let result = resolve_env_string("prefix_${TEST_VAR}_suffix")?;
/// assert_eq!(result, "prefix_test_value_suffix");
/// # Ok::<(), agent_mcp::error::MCPError>(())
/// ```
pub fn resolve_env_string(s: &str) -> Result<String, MCPError> {
    let mut result = s.to_string();

    // Pattern for ${VAR} syntax
    let re_braces = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
        .map_err(|e| MCPError::InvalidPattern(e.to_string()))?;

    for cap in re_braces.captures_iter(s) {
        let var_name = &cap[1];
        let value =
            std::env::var(var_name).map_err(|_| MCPError::EnvVarNotFound(var_name.to_string()))?;
        result = result.replace(&cap[0], &value);
    }

    // Pattern for $VAR syntax (without braces)
    let re_simple = regex::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)")
        .map_err(|e| MCPError::InvalidPattern(e.to_string()))?;

    for cap in re_simple.captures_iter(&result.clone()) {
        let var_name = &cap[1];
        let value =
            std::env::var(var_name).map_err(|_| MCPError::EnvVarNotFound(var_name.to_string()))?;
        result = result.replace(&cap[0], &value);
    }

    Ok(result)
}

/// Check if a tool should be included based on allow/deny rules
///
/// Deny list takes precedence over allow list.
pub fn should_include_tool(tool_name: &str, config: &AgentMCPConfig) -> bool {
    // Check deny list first (highest priority)
    if config.tools.deny.contains(&tool_name.to_string()) {
        return false;
    }

    // Check allow list
    match &config.tools.allow {
        ToolPattern::All(pattern) => pattern == "*",
        ToolPattern::List(allowed) => allowed.contains(&tool_name.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing_stdio() {
        let json = r#"{
            "mcpServers": {
                "test": {
                    "transport": "stdio",
                    "command": "test-server",
                    "args": ["--verbose"]
                }
            }
        }"#;

        let config: MCPConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);

        match config.mcp_servers.get("test").unwrap() {
            MCPServerConfig::Stdio {
                command,
                args,
                env,
                cwd,
            } => {
                assert_eq!(command, "test-server");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], "--verbose");
                assert!(env.is_empty());
                assert!(cwd.is_none());
            }
            _ => panic!("Expected Stdio transport"),
        }
    }

    #[test]
    fn test_config_parsing_http() {
        let json = r#"{
            "mcpServers": {
                "test": {
                    "transport": "http",
                    "url": "http://localhost:8080/mcp",
                    "headers": {"Authorization": "Bearer token"}
                }
            }
        }"#;

        let config: MCPConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.mcp_servers.len(), 1);

        match config.mcp_servers.get("test").unwrap() {
            MCPServerConfig::Http {
                url,
                headers,
                timeout_secs,
            } => {
                assert_eq!(url, "http://localhost:8080/mcp");
                assert_eq!(headers.get("Authorization").unwrap(), "Bearer token");
                assert_eq!(*timeout_secs, 30); // default
            }
            _ => panic!("Expected Http transport"),
        }
    }

    #[test]
    fn test_env_var_resolution() {
        unsafe {
            std::env::set_var("TEST_VAR", "test_value");
            std::env::set_var("ANOTHER_VAR", "another_value");
        }

        let result = resolve_env_string("${TEST_VAR}").unwrap();
        assert_eq!(result, "test_value");

        let result = resolve_env_string("prefix_${TEST_VAR}_suffix").unwrap();
        assert_eq!(result, "prefix_test_value_suffix");

        let result = resolve_env_string("$TEST_VAR").unwrap();
        assert_eq!(result, "test_value");

        let result = resolve_env_string("${TEST_VAR}_${ANOTHER_VAR}").unwrap();
        assert_eq!(result, "test_value_another_value");
    }

    #[test]
    fn test_tool_filtering() {
        let config = AgentMCPConfig {
            mcp_servers: vec!["test".to_string()],
            tools: ToolFilter {
                allow: ToolPattern::List(vec!["read_file".to_string(), "write_file".to_string()]),
                deny: vec!["delete_file".to_string()],
            },
            resources: Default::default(),
        };

        assert!(should_include_tool("read_file", &config));
        assert!(should_include_tool("write_file", &config));
        assert!(!should_include_tool("delete_file", &config));
        assert!(!should_include_tool("unknown_tool", &config));
    }

    #[test]
    fn test_tool_filtering_wildcard() {
        let config = AgentMCPConfig {
            mcp_servers: vec!["test".to_string()],
            tools: ToolFilter {
                allow: ToolPattern::All("*".to_string()),
                deny: vec!["dangerous_tool".to_string()],
            },
            resources: Default::default(),
        };

        assert!(should_include_tool("read_file", &config));
        assert!(should_include_tool("any_tool", &config));
        assert!(!should_include_tool("dangerous_tool", &config));
    }

    #[test]
    fn test_config_merge() {
        let mut config1 = MCPConfig::default();
        config1.mcp_servers.insert(
            "server1".to_string(),
            MCPServerConfig::Stdio {
                command: "test1".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
            },
        );

        let mut config2 = MCPConfig::default();
        config2.mcp_servers.insert(
            "server2".to_string(),
            MCPServerConfig::Stdio {
                command: "test2".to_string(),
                args: vec![],
                env: HashMap::new(),
                cwd: None,
            },
        );

        config1.merge(config2);
        assert_eq!(config1.mcp_servers.len(), 2);
        assert!(config1.mcp_servers.contains_key("server1"));
        assert!(config1.mcp_servers.contains_key("server2"));
    }

    #[test]
    fn test_get_agent_config() {
        let mut config = MCPConfig::default();

        config.agent_configurations.insert(
            "agent1".to_string(),
            AgentMCPConfig {
                mcp_servers: vec!["server1".to_string()],
                tools: Default::default(),
                resources: Default::default(),
            },
        );

        config.agent_configurations.insert(
            "default".to_string(),
            AgentMCPConfig {
                mcp_servers: vec!["default_server".to_string()],
                tools: Default::default(),
                resources: Default::default(),
            },
        );

        // Should get specific config
        let agent1_config = config.get_agent_config("agent1").unwrap();
        assert_eq!(agent1_config.mcp_servers[0], "server1");

        // Should fall back to default
        let unknown_config = config.get_agent_config("unknown").unwrap();
        assert_eq!(unknown_config.mcp_servers[0], "default_server");
    }
}
