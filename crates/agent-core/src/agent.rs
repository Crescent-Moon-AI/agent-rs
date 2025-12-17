//! Core Agent trait definition

use crate::{Context, Result};
use async_trait::async_trait;

/// Core trait that all agents must implement
///
/// Note: The Agent trait no longer uses Message directly. The Message type
/// has moved to agent-llm crate as it's LLM-specific. Concrete agent
/// implementations should use agent_llm::Message when interacting with LLMs.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Process input and return output
    ///
    /// The input/output types are intentionally kept as String for maximum
    /// flexibility. Concrete implementations can parse/format as needed.
    async fn process(&self, input: String, context: &mut Context) -> Result<String>;

    /// Get the agent's name
    fn name(&self) -> &str;

    /// Initialize the agent (optional)
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the agent (optional)
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
