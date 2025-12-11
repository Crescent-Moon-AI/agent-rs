//! Workflow agent implementation (wraps a workflow as an agent)

use agent_core::{Agent, Context, Result};
use async_trait::async_trait;
use crate::workflow::Workflow;

/// Wraps a Workflow as an Agent
///
/// WorkflowAgent allows workflows to be used as agents within other workflows,
/// enabling hierarchical workflow composition.
///
/// # Example
///
/// ```no_run
/// use agent_workflow::{Workflow, WorkflowAgent};
/// use agent_runtime::AgentRuntime;
/// use std::sync::Arc;
///
/// # async fn example() -> agent_core::Result<()> {
/// let runtime = Arc::new(AgentRuntime::builder()
///     .provider(provider)
///     .build()?);
///
/// // Create a sub-workflow
/// let sub_workflow = Workflow::builder(runtime.clone())
///     .add_agent(Arc::new(agent1))
///     .add_agent(Arc::new(agent2))
///     .build()?;
///
/// // Wrap it as an agent
/// let workflow_agent = WorkflowAgent::new("sub_workflow", sub_workflow);
///
/// // Use in a parent workflow
/// let main_workflow = Workflow::builder(runtime.clone())
///     .add_agent(Arc::new(workflow_agent))
///     .add_agent(Arc::new(agent3))
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct WorkflowAgent {
    workflow: Workflow,
    name: String,
}

impl WorkflowAgent {
    /// Create a new workflow agent
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the agent
    /// * `workflow` - The workflow to wrap
    pub fn new(name: impl Into<String>, workflow: Workflow) -> Self {
        Self {
            workflow,
            name: name.into(),
        }
    }

    /// Get a reference to the underlying workflow
    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }
}

#[async_trait]
impl Agent for WorkflowAgent {
    async fn process(&self, input: String, _context: &mut Context) -> Result<String> {
        // Execute the workflow
        self.workflow.execute(input).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_agent_name() {
        // Test that workflow agent can be created
        // Note: This would require a mock workflow to fully test
    }
}
