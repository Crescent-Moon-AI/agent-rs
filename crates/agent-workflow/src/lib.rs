//! Multi-agent orchestration and workflow execution for agent-rs
//!
//! This crate provides capabilities for coordinating multiple agents and
//! executing complex workflows, including:
//!
//! - AgentExecutor: Core agent loop (LLM → tools → repeat)
//! - Workflow: Sequential multi-agent execution

pub mod executor;
pub mod workflow;

pub use executor::{AgentExecutor, AgentExecutorBuilder, ExecutorConfig};
pub use workflow::Workflow;
