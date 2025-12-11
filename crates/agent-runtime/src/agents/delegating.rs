//! Delegating agent implementation (routes to sub-agents)

use agent_core::{Agent, Context, Error, Result};
use async_trait::async_trait;
use crate::runtime::AgentRuntime;
use std::collections::HashMap;
use std::sync::Arc;

/// An agent that delegates to sub-agents based on routing logic
///
/// DelegatingAgent provides a hierarchical agent structure where a manager
/// agent routes requests to specialized worker agents. This is useful for:
/// - Manager-worker patterns
/// - Specialized task routing
/// - Dynamic agent selection
///
/// # Example
///
/// ```no_run
/// use agent_runtime::{AgentRuntime, DelegatingAgent};
/// use agent_core::{Agent, Context};
/// use std::sync::Arc;
///
/// # async fn example() -> agent_core::Result<()> {
/// let runtime = Arc::new(AgentRuntime::builder()
///     .provider(provider)
///     .build()?);
///
/// let delegator = DelegatingAgent::builder(runtime.clone(), "manager")
///     .add_agent("coder", Arc::new(coder_agent))
///     .add_agent("reviewer", Arc::new(reviewer_agent))
///     .router(|input, _ctx| {
///         if input.contains("write") {
///             "coder".to_string()
///         } else {
///             "reviewer".to_string()
///         }
///     })
///     .build()?;
///
/// let mut context = Context::new();
/// let response = delegator.process("write a function".to_string(), &mut context).await?;
/// # Ok(())
/// # }
/// ```
pub struct DelegatingAgent {
    #[allow(dead_code)]
    runtime: Arc<AgentRuntime>,
    sub_agents: HashMap<String, Arc<dyn Agent>>,
    router: Box<dyn Fn(&str, &Context) -> String + Send + Sync>,
    name: String,
}

impl DelegatingAgent {
    /// Create a new builder for delegating agent
    ///
    /// # Arguments
    ///
    /// * `runtime` - The runtime to use
    /// * `name` - Name of the delegating agent
    pub fn builder(runtime: Arc<AgentRuntime>, name: impl Into<String>) -> DelegatingAgentBuilder {
        DelegatingAgentBuilder::new(runtime, name)
    }

    /// Get the number of sub-agents
    pub fn agent_count(&self) -> usize {
        self.sub_agents.len()
    }

    /// Get the list of available agent names
    pub fn agent_names(&self) -> Vec<&str> {
        self.sub_agents.keys().map(|s| s.as_str()).collect()
    }
}

#[async_trait]
impl Agent for DelegatingAgent {
    async fn process(&self, input: String, context: &mut Context) -> Result<String> {
        // Use router to determine which agent to delegate to
        let agent_name = (self.router)(&input, context);

        // Get the selected agent
        let agent = self
            .sub_agents
            .get(&agent_name)
            .ok_or_else(|| {
                Error::ProcessingFailed(format!(
                    "Agent '{}' not found. Available agents: {:?}",
                    agent_name,
                    self.agent_names()
                ))
            })?;

        // Delegate to the selected agent
        agent.process(input, context).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for DelegatingAgent
pub struct DelegatingAgentBuilder {
    runtime: Arc<AgentRuntime>,
    sub_agents: HashMap<String, Arc<dyn Agent>>,
    router: Option<Box<dyn Fn(&str, &Context) -> String + Send + Sync>>,
    name: String,
}

impl DelegatingAgentBuilder {
    /// Create a new builder
    pub fn new(runtime: Arc<AgentRuntime>, name: impl Into<String>) -> Self {
        Self {
            runtime,
            sub_agents: HashMap::new(),
            router: None,
            name: name.into(),
        }
    }

    /// Add a sub-agent
    ///
    /// # Arguments
    ///
    /// * `key` - The key to identify this agent in routing
    /// * `agent` - The agent to add
    pub fn add_agent(mut self, key: impl Into<String>, agent: Arc<dyn Agent>) -> Self {
        self.sub_agents.insert(key.into(), agent);
        self
    }

    /// Set the routing function
    ///
    /// The router receives the input string and context, and returns
    /// the key of the agent that should handle the request.
    ///
    /// # Arguments
    ///
    /// * `router` - Function that takes (input, context) and returns agent key
    pub fn router<F>(mut self, router: F) -> Self
    where
        F: Fn(&str, &Context) -> String + Send + Sync + 'static,
    {
        self.router = Some(Box::new(router));
        self
    }

    /// Build the delegating agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No router function is set
    /// - No sub-agents are added
    pub fn build(self) -> Result<DelegatingAgent> {
        let router = self.router.ok_or_else(|| {
            Error::InitializationFailed("Router function not set".to_string())
        })?;

        if self.sub_agents.is_empty() {
            return Err(Error::InitializationFailed(
                "No sub-agents added".to_string(),
            ));
        }

        Ok(DelegatingAgent {
            runtime: self.runtime,
            sub_agents: self.sub_agents,
            router,
            name: self.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegating_agent_builder() {
        // Test that builder validates router is set
        let runtime = Arc::new(AgentRuntime::builder());
        // Note: This would require mock agents to fully test
    }
}
