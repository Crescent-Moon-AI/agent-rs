//! Workflow definition and execution

use agent_core::{Agent, Context, Result};
use std::sync::Arc;

/// A workflow that coordinates multiple agents
///
/// Workflows execute agents sequentially, passing the output of each agent
/// as input to the next agent.
pub struct Workflow {
    agents: Vec<Arc<dyn Agent>>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
        }
    }

    /// Add an agent to the workflow
    pub fn add_agent(&mut self, agent: Arc<dyn Agent>) {
        self.agents.push(agent);
    }

    /// Execute the workflow
    ///
    /// # Arguments
    ///
    /// * `initial_input` - The initial input string to process
    ///
    /// # Returns
    ///
    /// The final output after all agents have processed
    pub async fn execute(&self, initial_input: String) -> Result<String> {
        let mut context = Context::new();
        let mut current_output = initial_input;

        // Sequential execution through all agents
        for agent in &self.agents {
            current_output = agent.process(current_output, &mut context).await?;
        }

        Ok(current_output)
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new()
    }
}
