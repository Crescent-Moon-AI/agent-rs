//! Model Context Protocol (MCP) integration for agent-rs
//!
//! This crate provides MCP client support for agent-rs, enabling agents to:
//! - Connect to external MCP servers via stdio or HTTP/SSE transports
//! - Discover and execute tools from MCP servers
//! - Access resources through the MCP protocol
//! - Configure different MCP servers per agent
//!
//! # Example
//!
//! ```no_run
//! use agent_mcp::config::MCPConfig;
//! use agent_mcp::client::manager::MCPClientManager;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = MCPConfig::load_merged()?;
//!
//! // Create client manager for an agent
//! let manager = MCPClientManager::new(Arc::new(config), "my-agent".to_string());
//!
//! // Initialize connections to configured MCP servers
//! manager.initialize().await?;
//!
//! // Discover tools
//! let tools = manager.discover_tools().await?;
//!
//! println!("Discovered {} tools", tools.len());
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod config;
pub mod context;
pub mod discovery;
pub mod error;
pub mod resource;
pub mod retry;
pub mod schema;
pub mod tool;

// Re-export commonly used types
pub use client::manager::MCPClientManager;
pub use config::{AgentMCPConfig, MCPConfig, MCPServerConfig};
pub use context::{MCPContext, MCPContextExt};
pub use error::MCPError;
pub use resource::{MCPResource, ResourceCache, ResourceFilter};
pub use retry::RetryPolicy;
pub use tool::MCPTool;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, MCPError>;
