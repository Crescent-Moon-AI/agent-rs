//! Agent runtime for executing agents with dependency injection
//!
//! This crate provides the runtime infrastructure for executing agents,
//! including the AgentExecutor for LLM loops, AgentRuntime for dependency
//! management, and concrete agent implementations.

pub mod executor;
pub mod runtime;
pub mod agents;

// Re-export key types
pub use executor::{AgentExecutor, AgentExecutorBuilder, ExecutorConfig};
pub use runtime::{AgentRuntime, AgentRuntimeBuilder, RuntimeConfig};
pub use agents::{
    DelegatingAgent, DelegatingAgentBuilder, SimpleAgent, SimpleConfig, ToolAgent,
};
