//! Concrete LLM provider implementations
//!
//! This module contains implementations of the LLMProvider trait for
//! various LLM services.

#[cfg(feature = "anthropic")]
pub mod anthropic;

#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicProvider;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "openai")]
pub use openai::{OpenAIConfig, OpenAIProvider};
