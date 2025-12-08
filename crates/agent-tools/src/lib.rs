//! Tool management and execution framework for agent-rs
//!
//! This crate provides a framework for defining and executing tools (functions)
//! that agents can use to perform actions.

pub mod registry;
pub mod tool;

pub use registry::ToolRegistry;
pub use tool::Tool;
