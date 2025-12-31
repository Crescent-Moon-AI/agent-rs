//! Agent runtime for executing agents with dependency injection
//!
//! This crate provides the runtime infrastructure for executing agents,
//! including the AgentExecutor for LLM loops, AgentRuntime for dependency
//! management, and concrete agent implementations.

pub mod agents;
pub mod executor;
pub mod runtime;

// Re-export key types
pub use agents::{DelegatingAgent, DelegatingAgentBuilder, SimpleAgent, SimpleConfig, ToolAgent};
pub use executor::{
    AgentExecutor, AgentExecutorBuilder, ExecutorConfig, ExecutorEventHandler, NoOpEventHandler,
};
pub use runtime::{AgentRuntime, AgentRuntimeBuilder, RuntimeConfig};
