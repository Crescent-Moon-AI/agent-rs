//! Shared utilities for agent-rs
//!
//! This crate provides common functionality used across the agent-rs workspace,
//! including logging setup, configuration management, and utility functions.

pub mod config;
pub mod logging;

pub use config::Config;
pub use logging::init_tracing;
