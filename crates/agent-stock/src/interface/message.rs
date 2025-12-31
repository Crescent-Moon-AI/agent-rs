//! Message types for bot communication

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub reply_to: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Text,
    Command,
    Response,
    System,
}

impl Message {
    pub fn text(user_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            content: content.into(),
            message_type: MessageType::Text,
            timestamp: Utc::now(),
            reply_to: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    pub fn command(user_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            content: content.into(),
            message_type: MessageType::Command,
            timestamp: Utc::now(),
            reply_to: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    pub fn replying_to(mut self, message_id: impl Into<String>) -> Self {
        self.reply_to = Some(message_id.into());
        self
    }
    
    pub fn is_command(&self) -> bool {
        self.message_type == MessageType::Command || self.content.starts_with('/')
    }
    
    pub fn parse_command(&self) -> Option<(String, Vec<String>)> {
        if !self.is_command() {
            return None;
        }
        
        let content = self.content.trim_start_matches('/');
        let parts: Vec<&str> = content.split_whitespace().collect();
        
        if parts.is_empty() {
            return None;
        }
        
        let command = parts[0].to_string();
        let args = parts[1..].iter().map(std::string::ToString::to_string).collect();
        
        Some((command, args))
    }
}
