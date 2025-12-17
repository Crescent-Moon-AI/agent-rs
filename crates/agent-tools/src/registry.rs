//! Tool registry for managing available tools

use crate::Tool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing tools
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn Tool>>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a tool
    pub fn register(&self, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().unwrap();
        tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().unwrap();
        tools.get(name).cloned()
    }

    /// List all registered tools
    ///
    /// Returns a vector of all tools in the registry. This is useful for
    /// building tool definitions to send to the LLM.
    pub fn list_tools(&self) -> Vec<Arc<dyn Tool>> {
        let tools = self.tools.read().unwrap();
        tools.values().cloned().collect()
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        let tools = self.tools.read().unwrap();
        tools.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        let tools = self.tools.read().unwrap();
        tools.is_empty()
    }
}
