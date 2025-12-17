//! Runtime for executing agents with dependency injection
//!
//! The AgentRuntime manages shared resources like LLM providers and tool registries,
//! and provides factory methods for creating different types of agents.

use agent_core::Result;
use agent_llm::LLMProvider;
use agent_mcp::{MCPClientManager, MCPConfig};
use agent_tools::ToolRegistry;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

use crate::agents::{SimpleAgent, SimpleConfig, ToolAgent};
use crate::executor::{AgentExecutor, ExecutorConfig};

/// Configuration for the agent runtime
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Default maximum iterations for tool-using agents
    pub default_max_iterations: usize,

    /// Default model to use
    pub default_model: String,

    /// Path to MCP configuration file
    pub mcp_config_path: Option<PathBuf>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            default_max_iterations: 10,
            default_model: "claude-sonnet-4-5-20250929".to_string(),
            mcp_config_path: None,
        }
    }
}

/// Runtime for executing agents with dependency injection
///
/// The AgentRuntime manages shared resources (LLM provider, tool registry)
/// and provides factory methods for creating different types of agents.
///
/// # Example
///
/// ```no_run
/// use agent_runtime::{AgentRuntime, SimpleConfig, ExecutorConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> agent_core::Result<()> {
/// let runtime = AgentRuntime::builder()
///     .provider(provider)
///     .tool_registry(tools)
///     .build()?;
///
/// // Create a simple agent
/// let simple_agent = runtime.create_simple_agent(
///     SimpleConfig::default(),
///     "assistant"
/// );
///
/// // Create a tool-using agent
/// let tool_agent = runtime.create_tool_agent(
///     ExecutorConfig::default(),
///     "researcher"
/// );
/// # Ok(())
/// # }
/// ```
pub struct AgentRuntime {
    provider: Arc<dyn LLMProvider>,
    tool_registry: Arc<ToolRegistry>,
    config: RuntimeConfig,
    mcp_config: Option<Arc<MCPConfig>>,
}

impl AgentRuntime {
    /// Create a new agent runtime
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        tool_registry: Arc<ToolRegistry>,
        config: RuntimeConfig,
        mcp_config: Option<Arc<MCPConfig>>,
    ) -> Self {
        Self {
            provider,
            tool_registry,
            config,
            mcp_config,
        }
    }

    /// Create a new runtime builder
    pub fn builder() -> AgentRuntimeBuilder {
        AgentRuntimeBuilder::new()
    }

    /// Get a reference to the LLM provider
    pub fn provider(&self) -> &Arc<dyn LLMProvider> {
        &self.provider
    }

    /// Get a reference to the tool registry
    pub fn tools(&self) -> &Arc<ToolRegistry> {
        &self.tool_registry
    }

    /// Get a reference to the runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get a reference to the MCP configuration
    pub fn mcp_config(&self) -> Option<&Arc<MCPConfig>> {
        self.mcp_config.as_ref()
    }

    /// Create a simple agent (LLM only, no tools)
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the simple agent
    /// * `name` - Name of the agent
    ///
    /// # Returns
    ///
    /// A new SimpleAgent instance
    pub fn create_simple_agent(
        &self,
        config: SimpleConfig,
        name: impl Into<String>,
    ) -> SimpleAgent {
        SimpleAgent::new(self.provider.clone(), config, name.into())
    }

    /// Create a tool-using agent (with LLM loop and tool execution)
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the executor
    /// * `name` - Name of the agent
    ///
    /// # Returns
    ///
    /// A new ToolAgent instance
    pub fn create_tool_agent(&self, config: ExecutorConfig, name: impl Into<String>) -> ToolAgent {
        let executor =
            AgentExecutor::new(self.provider.clone(), self.tool_registry.clone(), config);
        ToolAgent::new(executor, name.into())
    }

    /// Create a tool-using agent with MCP support
    ///
    /// This method creates a tool agent and automatically discovers and registers
    /// MCP tools based on the agent's configuration. If MCP is not configured,
    /// it falls back to creating a regular tool agent.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the executor
    /// * `name` - Name of the agent (must match an agent configuration in MCP config)
    ///
    /// # Returns
    ///
    /// A Result containing the new ToolAgent instance with MCP tools registered
    ///
    /// # Errors
    ///
    /// Returns an error if MCP initialization or tool discovery fails
    pub async fn create_tool_agent_with_mcp(
        &self,
        config: ExecutorConfig,
        name: impl Into<String>,
    ) -> Result<ToolAgent> {
        let agent_name = name.into();

        // If no MCP config, create regular tool agent
        let Some(mcp_config) = &self.mcp_config else {
            info!("No MCP configuration found, creating regular tool agent");
            return Ok(self.create_tool_agent(config, agent_name));
        };

        // Check if this agent has MCP configuration
        let agent_config = mcp_config.get_agent_config(&agent_name);
        if agent_config.is_none() {
            info!(
                "No MCP configuration for agent '{}', creating regular tool agent",
                agent_name
            );
            return Ok(self.create_tool_agent(config, agent_name));
        }

        let agent_config = agent_config.unwrap();

        // Create MCP client manager for this agent
        let manager = Arc::new(MCPClientManager::new(
            mcp_config.clone(),
            agent_name.clone(),
        ));

        // Initialize MCP connections
        match manager.initialize().await {
            Ok(_) => info!("MCP client manager initialized for agent '{}'", agent_name),
            Err(e) => {
                warn!(
                    "Failed to initialize MCP clients for agent '{}': {}. Continuing with regular tools.",
                    agent_name, e
                );
                return Ok(self.create_tool_agent(config, agent_name));
            }
        }

        // Create a new registry with existing tools plus MCP tools
        let mut registry = ToolRegistry::new();

        // Copy existing tools to the new registry
        for tool in self.tool_registry.list_tools() {
            registry.register(tool);
        }

        // Discover and register MCP tools
        match agent_mcp::discovery::discover_and_register_tools(
            manager.clone(),
            &mut registry,
            agent_config,
        )
        .await
        {
            Ok(count) => {
                info!(
                    "Discovered and registered {} MCP tools for agent '{}'",
                    count, agent_name
                );
            }
            Err(e) => {
                warn!(
                    "Failed to discover MCP tools for agent '{}': {}. Continuing with existing tools.",
                    agent_name, e
                );
            }
        }

        // Create tool agent with the enhanced registry
        let registry = Arc::new(registry);
        let executor = AgentExecutor::new(self.provider.clone(), registry, config);
        Ok(ToolAgent::new(executor, agent_name))
    }
}

/// Builder for AgentRuntime
pub struct AgentRuntimeBuilder {
    provider: Option<Arc<dyn LLMProvider>>,
    tool_registry: Option<Arc<ToolRegistry>>,
    config: RuntimeConfig,
    mcp_config: Option<Arc<MCPConfig>>,
}

impl AgentRuntimeBuilder {
    /// Create a new runtime builder
    pub fn new() -> Self {
        Self {
            provider: None,
            tool_registry: None,
            config: RuntimeConfig::default(),
            mcp_config: None,
        }
    }

    /// Set the LLM provider
    pub fn provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set the tool registry
    pub fn tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Set the runtime configuration
    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the MCP configuration
    pub fn mcp_config(mut self, config: Arc<MCPConfig>) -> Self {
        self.mcp_config = Some(config);
        self
    }

    /// Load MCP configuration from file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed
    pub fn mcp_config_from_file(mut self, path: PathBuf) -> Result<Self> {
        let config = MCPConfig::from_file(&path)?;
        self.config.mcp_config_path = Some(path);
        self.mcp_config = Some(Arc::new(config));
        Ok(self)
    }

    /// Set the default max iterations
    pub fn default_max_iterations(mut self, max: usize) -> Self {
        self.config.default_max_iterations = max;
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = model.into();
        self
    }

    /// Set the MCP configuration path
    pub fn mcp_config_path(mut self, path: PathBuf) -> Self {
        self.config.mcp_config_path = Some(path);
        self
    }

    /// Build the runtime
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not set
    pub fn build(self) -> Result<AgentRuntime> {
        let provider = self.provider.ok_or_else(|| {
            agent_core::Error::InitializationFailed("Provider not set".to_string())
        })?;

        let tool_registry = self
            .tool_registry
            .unwrap_or_else(|| Arc::new(ToolRegistry::new()));

        Ok(AgentRuntime::new(
            provider,
            tool_registry,
            self.config,
            self.mcp_config,
        ))
    }
}

impl Default for AgentRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.default_max_iterations, 10);
        assert_eq!(config.default_model, "claude-sonnet-4-5-20250929");
        assert!(config.mcp_config_path.is_none());
    }

    #[test]
    fn test_runtime_builder() {
        let builder = AgentRuntimeBuilder::new()
            .default_max_iterations(5)
            .default_model("test-model");

        assert_eq!(builder.config.default_max_iterations, 5);
        assert_eq!(builder.config.default_model, "test-model");
    }

    #[test]
    fn test_runtime_builder_with_mcp_path() {
        let path = PathBuf::from("/path/to/mcp.json");
        let builder = AgentRuntimeBuilder::new().mcp_config_path(path.clone());

        assert_eq!(builder.config.mcp_config_path, Some(path));
    }

    #[test]
    fn test_runtime_has_mcp_config_accessor() {
        // Test that runtime exposes mcp_config accessor
        use agent_llm::{CompletionRequest, CompletionResponse, LLMProvider};
        use std::collections::HashMap;

        // Create minimal MCP config
        let mcp_config = Arc::new(MCPConfig {
            mcp_servers: HashMap::new(),
            agent_configurations: HashMap::new(),
        });

        // Mock provider for testing
        struct MockProvider;
        #[async_trait::async_trait]
        impl LLMProvider for MockProvider {
            async fn complete(
                &self,
                _request: CompletionRequest,
            ) -> agent_llm::Result<CompletionResponse> {
                unimplemented!()
            }
            fn name(&self) -> &str {
                "mock"
            }
        }

        let runtime = AgentRuntime::new(
            Arc::new(MockProvider),
            Arc::new(ToolRegistry::new()),
            RuntimeConfig::default(),
            Some(mcp_config.clone()),
        );

        assert!(runtime.mcp_config().is_some());
        assert_eq!(
            Arc::as_ptr(runtime.mcp_config().unwrap()),
            Arc::as_ptr(&mcp_config)
        );
    }
}
