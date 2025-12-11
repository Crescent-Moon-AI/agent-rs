//! Workflow definition and execution

use agent_core::{Agent, Context, Result};
use agent_runtime::AgentRuntime;
use std::sync::Arc;

/// A step in a workflow
#[derive(Clone)]
pub enum WorkflowStep {
    /// Execute a single agent
    Agent(Arc<dyn Agent>),
    /// Execute a sub-workflow
    SubWorkflow(Arc<Workflow>),
}

/// A workflow that coordinates multiple agents
///
/// Workflows execute agents sequentially, passing the output of each agent
/// as input to the next agent. Workflows support nesting, allowing complex
/// hierarchical structures.
///
/// # Example
///
/// ```no_run
/// use agent_workflow::Workflow;
/// use agent_runtime::AgentRuntime;
/// use std::sync::Arc;
///
/// # async fn example() -> agent_core::Result<()> {
/// let runtime = Arc::new(AgentRuntime::builder()
///     .provider(provider)
///     .tool_registry(tools)
///     .build()?);
///
/// let workflow = Workflow::builder(runtime.clone())
///     .add_agent(Arc::new(agent1))
///     .add_agent(Arc::new(agent2))
///     .build()?;
///
/// let result = workflow.execute("Input".to_string()).await?;
/// # Ok(())
/// # }
/// ```
pub struct Workflow {
    steps: Vec<WorkflowStep>,
    runtime: Arc<AgentRuntime>,
}

impl Workflow {
    /// Create a new workflow builder
    ///
    /// # Arguments
    ///
    /// * `runtime` - The runtime to use for agent execution
    pub fn builder(runtime: Arc<AgentRuntime>) -> WorkflowBuilder {
        WorkflowBuilder::new(runtime)
    }

    /// Execute the workflow
    ///
    /// # Arguments
    ///
    /// * `initial_input` - The initial input string to process
    ///
    /// # Returns
    ///
    /// The final output after all steps have been executed
    pub fn execute(&self, initial_input: String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + '_>> {
        Box::pin(async move {
            let mut context = Context::new();
            let mut current_output = initial_input;

            // Sequential execution through all steps
            for step in &self.steps {
                current_output = match step {
                    WorkflowStep::Agent(agent) => {
                        agent.process(current_output, &mut context).await?
                    }
                    WorkflowStep::SubWorkflow(workflow) => {
                        workflow.execute(current_output).await?
                    }
                };
            }

            Ok(current_output)
        })
    }

    /// Get a reference to the runtime
    pub fn runtime(&self) -> &Arc<AgentRuntime> {
        &self.runtime
    }
}

/// Builder for constructing workflows
pub struct WorkflowBuilder {
    steps: Vec<WorkflowStep>,
    runtime: Arc<AgentRuntime>,
}

impl WorkflowBuilder {
    /// Create a new workflow builder
    pub fn new(runtime: Arc<AgentRuntime>) -> Self {
        Self {
            steps: Vec::new(),
            runtime,
        }
    }

    /// Add an agent to the workflow
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to add
    pub fn add_agent(mut self, agent: Arc<dyn Agent>) -> Self {
        self.steps.push(WorkflowStep::Agent(agent));
        self
    }

    /// Add a sub-workflow to the workflow
    ///
    /// This allows for nested workflow structures.
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow to nest
    pub fn add_workflow(mut self, workflow: Workflow) -> Self {
        self.steps.push(WorkflowStep::SubWorkflow(Arc::new(workflow)));
        self
    }

    /// Build the workflow
    pub fn build(self) -> Result<Workflow> {
        Ok(Workflow {
            steps: self.steps,
            runtime: self.runtime,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder() {
        // Test that builder can be created
        // Note: This would require a mock runtime to fully test
    }
}
