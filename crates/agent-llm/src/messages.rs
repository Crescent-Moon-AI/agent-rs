//! Message types for LLM communication
//!
//! This module defines the core message types used for LLM interactions,
//! based on Anthropic's Claude API design with support for multi-modal content
//! and tool use.

use serde::{Deserialize, Serialize};

/// Message role in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// System message (handled separately in some providers)
    System,
}

/// Image source for multi-modal content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Image from URL
    Url {
        /// Image URL
        url: String,
    },
    /// Base64-encoded image
    Base64 {
        /// Media type (e.g., "image/png")
        media_type: String,
        /// Base64-encoded image data
        data: String,
    },
}

/// Content block in a message (supports multi-modal content)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text {
        /// Text content
        text: String,
    },

    /// Image content (base64 or URL)
    Image {
        /// Image source
        source: ImageSource,
    },

    /// Tool use request from assistant
    ToolUse {
        /// Unique ID for this tool use
        id: String,
        /// Tool name
        name: String,
        /// Tool input parameters (JSON)
        input: serde_json::Value,
    },

    /// Tool result from user
    ToolResult {
        /// ID of the tool use this is responding to
        tool_use_id: String,
        /// Result content
        content: String,
        /// Whether this is an error result
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

/// Message content: either simple text or structured blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Structured content blocks
    Blocks(Vec<ContentBlock>),
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: Role,

    /// Message content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
}

impl Message {
    /// Create a user message with text
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: Some(MessageContent::Text(text.into())),
        }
    }

    /// Create an assistant message with text
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: Some(MessageContent::Text(text.into())),
        }
    }

    /// Create a system message with text
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: Some(MessageContent::Text(text.into())),
        }
    }

    /// Create a user message with tool result
    pub fn tool_result(tool_use_id: String, result: String) -> Self {
        Self {
            role: Role::User,
            content: Some(MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id,
                content: result,
                is_error: None,
            }])),
        }
    }

    /// Create a user message with error tool result
    pub fn tool_error(tool_use_id: String, error: String) -> Self {
        Self {
            role: Role::User,
            content: Some(MessageContent::Blocks(vec![ContentBlock::ToolResult {
                tool_use_id,
                content: error,
                is_error: Some(true),
            }])),
        }
    }

    /// Extract text content from the message (convenience method)
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            Some(MessageContent::Text(s)) => Some(s),
            Some(MessageContent::Blocks(blocks)) => blocks.iter().find_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            }),
            None => None,
        }
    }

    /// Extract tool use requests from assistant messages
    pub fn tool_uses(&self) -> Vec<&ContentBlock> {
        match &self.content {
            Some(MessageContent::Blocks(blocks)) => blocks
                .iter()
                .filter(|b| matches!(b, ContentBlock::ToolUse { .. }))
                .collect(),
            _ => vec![],
        }
    }

    /// Check if this message contains any tool uses
    pub fn has_tool_uses(&self) -> bool {
        !self.tool_uses().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.text(), Some("Hello"));
    }

    #[test]
    fn test_assistant_message() {
        let msg = Message::assistant("Hi there");
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.text(), Some("Hi there"));
    }

    #[test]
    fn test_tool_result() {
        let msg = Message::tool_result("tool_123".to_string(), "result".to_string());
        assert_eq!(msg.role, Role::User);
        assert!(!msg.has_tool_uses());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("Test");
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.text(), Some("Test"));
    }
}
