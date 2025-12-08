//! LLM provider abstraction layer for agent-rs
//!
//! This crate provides provider-agnostic abstractions for interacting with
//! Large Language Models (LLMs). It includes:
//!
//! - Message types for LLM communication
//! - Completion request/response types
//! - Tool definitions for function calling
//! - Provider trait for LLM implementations
//! - Concrete provider implementations (behind feature flags)

pub mod completion;
pub mod error;
pub mod messages;
pub mod provider;
pub mod tools;

// Re-export main types
pub use completion::{CompletionRequest, CompletionResponse, StopReason, TokenUsage};
pub use error::{LLMError, Result};
pub use messages::{ContentBlock, ImageSource, Message, MessageContent, Role};
pub use provider::LLMProvider;
pub use tools::ToolDefinition;

// Provider implementations (feature-gated)
#[cfg(feature = "anthropic")]
pub mod providers;
