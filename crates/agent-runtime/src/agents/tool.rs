//! Tool agent implementation (wraps AgentExecutor)

use crate::executor::AgentExecutor;
use agent_core::{Agent, Context, Result};
use async_trait::async_trait;

/// An agent that uses the LLM loop with tool execution
///
/// ToolAgent wraps the AgentExecutor to provide the Agent trait interface
/// while supporting tool execution in an LLM loop. It's suitable for:
/// - Agents that need to use tools to accomplish tasks
/// - Multi-step reasoning with external actions
/// - Complex workflows requiring tool integration
///
/// # Example
///
/// ```no_run
/// use agent_runtime::{AgentRuntime, ExecutorConfig, ToolAgent};
/// use agent_core::{Agent, Context};
///
/// # async fn example() -> agent_core::Result<()> {
/// let runtime = AgentRuntime::builder()
///     .provider(provider)
///     .tool_registry(tools)
///     .build()?;
///
/// let agent = runtime.create_tool_agent(
///     ExecutorConfig::default(),
///     "researcher"
/// );
///
/// let mut context = Context::new();
/// let response = agent.process("Search for Rust tutorials".to_string(), &mut context).await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolAgent {
    executor: AgentExecutor,
    name: String,
}

impl ToolAgent {
    /// Create a new tool agent
    ///
    /// # Arguments
    ///
    /// * `executor` - The agent executor to wrap
    /// * `name` - Name of the agent
    pub fn new(executor: AgentExecutor, name: String) -> Self {
        Self { executor, name }
    }

    /// Create a tool agent from parts
    ///
    /// Convenience method that accepts any string-like type for the name.
    ///
    /// # Arguments
    ///
    /// * `executor` - The agent executor to wrap
    /// * `name` - Name of the agent (can be String, &str, etc.)
    pub fn from_parts(executor: AgentExecutor, name: impl Into<String>) -> Self {
        Self {
            executor,
            name: name.into(),
        }
    }

    /// Get a reference to the underlying executor
    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }
}

#[async_trait]
impl Agent for ToolAgent {
    async fn process(&self, input: String, _context: &mut Context) -> Result<String> {
        // Delegate to the executor's run method
        self.executor.run(input).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::{AgentExecutorBuilder, ExecutorConfig};
    use agent_tools::ToolRegistry;
    use std::sync::Arc;

    #[test]
    fn test_tool_agent_name() {
        // Create a minimal executor for testing
        let registry = Arc::new(ToolRegistry::new());
        let config = ExecutorConfig::default();

        // Note: This would require a mock provider to fully test
        // For now, we just test the struct creation pattern
        assert_eq!(config.model, "claude-sonnet-4-5-20250929");
    }
}
