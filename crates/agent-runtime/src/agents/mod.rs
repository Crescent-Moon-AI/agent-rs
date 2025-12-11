//! Concrete agent implementations
//!
//! This module provides concrete implementations of the Agent trait:
//! - SimpleAgent: LLM-only agent without tool execution
//! - ToolAgent: Agent with LLM loop and tool execution capabilities
//! - DelegatingAgent: Agent that routes to sub-agents based on custom logic

pub mod simple;
pub mod tool;
pub mod delegating;

pub use simple::{SimpleAgent, SimpleConfig};
pub use tool::ToolAgent;
pub use delegating::{DelegatingAgent, DelegatingAgentBuilder};
