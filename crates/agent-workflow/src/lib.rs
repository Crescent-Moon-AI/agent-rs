//! Multi-agent orchestration and workflow execution for agent-rs
//!
//! This crate provides capabilities for coordinating multiple agents and
//! executing complex workflows.

pub mod workflow;
pub mod workflow_agent;

// Re-export for convenience
pub use workflow::{Workflow, WorkflowBuilder, WorkflowStep};
pub use workflow_agent::WorkflowAgent;
