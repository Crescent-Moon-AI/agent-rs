//! Execution context for agents

use serde::{Deserialize, Serialize};
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

    /// Insert a typed value into the context
    ///
    /// Serializes the value to JSON before storing. This provides
    /// a more ergonomic API for storing structured data.
    ///
    /// # Example
    ///
    /// ```
    /// use agent_core::Context;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct StockData {
    ///     symbol: String,
    ///     price: f64,
    /// }
    ///
    /// let mut ctx = Context::new();
    /// let data = StockData {
    ///     symbol: "AAPL".to_string(),
    ///     price: 150.0,
    /// };
    ///
    /// ctx.insert_typed("stock", &data).unwrap();
    /// ```
    pub fn insert_typed<T: Serialize>(&mut self, key: impl Into<String>, value: &T) -> crate::Result<()> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| crate::Error::ProcessingFailed(format!("Failed to serialize context value: {}", e)))?;
        self.data.insert(key.into(), json_value);
        Ok(())
    }

    /// Get a typed value from the context
    ///
    /// Deserializes the JSON value into the specified type. Returns None
    /// if the key doesn't exist, or an error if deserialization fails.
    ///
    /// # Example
    ///
    /// ```
    /// use agent_core::Context;
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct StockData {
    ///     symbol: String,
    ///     price: f64,
    /// }
    ///
    /// let mut ctx = Context::new();
    /// let data = StockData {
    ///     symbol: "AAPL".to_string(),
    ///     price: 150.0,
    /// };
    ///
    /// ctx.insert_typed("stock", &data).unwrap();
    /// let retrieved: StockData = ctx.get_typed("stock").unwrap().unwrap();
    /// assert_eq!(retrieved.symbol, "AAPL");
    /// ```
    pub fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> crate::Result<Option<T>> {
        match self.data.get(key) {
            None => Ok(None),
            Some(value) => {
                let typed = serde_json::from_value(value.clone())
                    .map_err(|e| crate::Error::ProcessingFailed(format!("Failed to deserialize context value: {}", e)))?;
                Ok(Some(typed))
            }
        }
    }

    /// Check if a key exists in the context
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Remove a value from the context
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    /// Clear all values from the context
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the number of entries in the context
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the context is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: i32,
        text: String,
    }

    #[test]
    fn test_basic_operations() {
        let mut ctx = Context::new();
        assert!(ctx.is_empty());

        ctx.insert("key", serde_json::json!("value"));
        assert_eq!(ctx.len(), 1);
        assert!(ctx.contains_key("key"));
        assert_eq!(ctx.get("key"), Some(&serde_json::json!("value")));

        ctx.remove("key");
        assert!(ctx.is_empty());
    }

    #[test]
    fn test_typed_insert_get() {
        let mut ctx = Context::new();
        let data = TestData {
            value: 42,
            text: "hello".to_string(),
        };

        ctx.insert_typed("test", &data).unwrap();

        let retrieved: TestData = ctx.get_typed("test").unwrap().unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_get_typed_missing_key() {
        let ctx = Context::new();
        let result: crate::Result<Option<TestData>> = ctx.get_typed("missing");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_clear() {
        let mut ctx = Context::new();
        ctx.insert("key1", serde_json::json!(1));
        ctx.insert("key2", serde_json::json!(2));
        assert_eq!(ctx.len(), 2);

        ctx.clear();
        assert!(ctx.is_empty());
    }
}

