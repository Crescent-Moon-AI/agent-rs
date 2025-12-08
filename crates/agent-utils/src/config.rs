//! Configuration management utilities

use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub app_name: String,
    /// Environment (dev, prod, etc.)
    pub environment: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "agent-rs".to_string(),
            environment: "development".to_string(),
        }
    }
}
