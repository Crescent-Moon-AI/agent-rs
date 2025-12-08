//! Execution context for agents

use std::collections::HashMap;

/// Context passed to agents during execution
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Key-value storage for context data
    data: HashMap<String, serde_json::Value>,
}

impl Context {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value into the context
    pub fn insert(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.data.insert(key.into(), value);
    }

    /// Get a value from the context
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}
