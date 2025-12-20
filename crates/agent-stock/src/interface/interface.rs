//! Bot interface trait and core abstractions
//!
//! Defines platform-agnostic interface for all bot implementations

use crate::engine::AnalysisContext;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Platform identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BotPlatform {
    /// Command-line interface
    CLI,
    
    /// Telegram bot
    Telegram,
    
    /// DingTalk bot
    DingTalk,
    
    /// Feishu (Lark) bot
    Feishu,
    
    /// Web interface
    Web,
    
    /// Custom platform
    Custom,
}

/// Bot response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResponse {
    /// Response content
    pub content: String,
    
    /// Response type
    pub response_type: ResponseType,
    
    /// Attachments (images, files, etc.)
    pub attachments: Vec<Attachment>,
    
    /// Suggested actions
    pub actions: Vec<SuggestedAction>,
    
    /// Metadata for the platform
    pub metadata: serde_json::Value,
}

/// Type of bot response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseType {
    /// Plain text
    Text,
    
    /// Formatted text (Markdown, HTML, etc.)
    Formatted,
    
    /// Interactive card/rich message
    Interactive,
    
    /// Error message
    Error,
}

/// Attachment in a bot response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment type
    pub attachment_type: AttachmentType,
    
    /// Content or URL
    pub content: Vec<u8>,
    
    /// File name
    pub filename: Option<String>,
    
    /// MIME type
    pub mime_type: String,
}

/// Type of attachment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentType {
    /// Image file
    Image,
    
    /// Document
    Document,
    
    /// Chart/graph
    Chart,
}

/// Suggested action for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action label
    pub label: String,
    
    /// Action command or callback data
    pub action: String,
    
    /// Action type
    pub action_type: ActionType,
}

/// Type of action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    /// Execute a command
    Command,
    
    /// Follow-up query
    Query,
    
    /// External link
    Link,
}

impl BotResponse {
    /// Create a simple text response
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            response_type: ResponseType::Text,
            attachments: Vec::new(),
            actions: Vec::new(),
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Create a formatted response
    pub fn formatted(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            response_type: ResponseType::Formatted,
            attachments: Vec::new(),
            actions: Vec::new(),
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Create an error response
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            response_type: ResponseType::Error,
            attachments: Vec::new(),
            actions: Vec::new(),
            metadata: serde_json::Value::Null,
        }
    }
    
    /// Add an attachment
    pub fn with_attachment(mut self, attachment: Attachment) -> Self {
        self.attachments.push(attachment);
        self
    }
    
    /// Add a suggested action
    pub fn with_action(mut self, label: impl Into<String>, action: impl Into<String>) -> Self {
        self.actions.push(SuggestedAction {
            label: label.into(),
            action: action.into(),
            action_type: ActionType::Command,
        });
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Main bot interface trait
///
/// All platform implementations must implement this trait
#[async_trait]
pub trait BotInterface: Send + Sync {
    /// Get the platform identifier
    fn platform(&self) -> BotPlatform;
    
    /// Handle an incoming message
    async fn on_message(
        &mut self,
        user_id: &str,
        message: &str,
        context: &mut AnalysisContext,
    ) -> Result<BotResponse>;
    
    /// Handle a command
    async fn on_command(
        &mut self,
        user_id: &str,
        command: &str,
        args: &[String],
        context: &mut AnalysisContext,
    ) -> Result<BotResponse>;
    
    /// Format an analysis result for the platform
    fn format_response(&self, content: &str, context: &AnalysisContext) -> BotResponse;
    
    /// Handle user joining (optional)
    async fn on_user_join(&mut self, _user_id: &str) -> Result<()> {
        Ok(())
    }
    
    /// Handle user leaving (optional)
    async fn on_user_leave(&mut self, _user_id: &str) -> Result<()> {
        Ok(())
    }
    
    /// Handle platform-specific events (optional)
    async fn on_event(&mut self, _event: serde_json::Value) -> Result<()> {
        Ok(())
    }
}

impl std::fmt::Display for BotPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotPlatform::CLI => write!(f, "CLI"),
            BotPlatform::Telegram => write!(f, "Telegram"),
            BotPlatform::DingTalk => write!(f, "DingTalk"),
            BotPlatform::Feishu => write!(f, "Feishu"),
            BotPlatform::Web => write!(f, "Web"),
            BotPlatform::Custom => write!(f, "Custom"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bot_response_creation() {
        let response = BotResponse::text("Hello, world!");
        assert_eq!(response.response_type, ResponseType::Text);
        assert_eq!(response.content, "Hello, world!");
    }
    
    #[test]
    fn test_bot_response_builder() {
        let response = BotResponse::formatted("**Analysis**")
            .with_action("Refresh", "/refresh")
            .with_action("Compare", "/compare");
        
        assert_eq!(response.actions.len(), 2);
        assert_eq!(response.actions[0].label, "Refresh");
    }
}
