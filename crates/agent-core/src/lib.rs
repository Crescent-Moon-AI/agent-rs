//! Core abstractions and runtime for agent-rs
//!
//! This crate defines the fundamental traits and types used throughout the agent-rs framework.

pub mod agent;
pub mod context;
pub mod error;

pub use agent::Agent;
pub use context::Context;
pub use error::{Error, Result};
