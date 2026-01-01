//! Execution context for agents
//!
//! The `Context` struct provides a flexible key-value store for passing
//! runtime configuration and state to agents during execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Well-known context keys for common configuration
pub mod keys {
    /// Language preference (e.g., "en", "zh")
    pub const LANGUAGE: &str = "language";
    /// Response format preference (e.g., "json", "text", "markdown")
    pub const RESPONSE_FORMAT: &str = "response_format";
    /// User ID for personalization
    pub const USER_ID: &str = "user_id";
    /// Session ID for tracking
    pub const SESSION_ID: &str = "session_id";
    /// Timezone for date/time formatting
    pub const TIMEZONE: &str = "timezone";
}

/// Context passed to agents during execution
///
/// Context provides a flexible way to pass configuration and state to agents.
/// It supports both untyped JSON values and typed accessors for common fields.
///
/// # Example
///
/// ```
/// use agent_core::Context;
///
/// let mut ctx = Context::new()
///     .with_language("en")
///     .with_session_id("session-123");
///
/// assert_eq!(ctx.language(), Some("en"));
/// assert_eq!(ctx.session_id(), Some("session-123"));
/// ```
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

    // =========== Builder Methods ===========

    /// Set the language preference
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.set_language(lang);
        self
    }

    /// Set the session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.insert(keys::SESSION_ID, serde_json::json!(session_id.into()));
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.insert(keys::USER_ID, serde_json::json!(user_id.into()));
        self
    }

    /// Set the timezone
    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.insert(keys::TIMEZONE, serde_json::json!(timezone.into()));
        self
    }

    // =========== Common Accessors ===========

    /// Get the language preference
    pub fn language(&self) -> Option<&str> {
        self.get(keys::LANGUAGE).and_then(|v| v.as_str())
    }

    /// Set the language preference
    pub fn set_language(&mut self, lang: impl Into<String>) {
        self.insert(keys::LANGUAGE, serde_json::json!(lang.into()));
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&str> {
        self.get(keys::SESSION_ID).and_then(|v| v.as_str())
    }

    /// Get the user ID
    pub fn user_id(&self) -> Option<&str> {
        self.get(keys::USER_ID).and_then(|v| v.as_str())
    }

    /// Get the timezone
    pub fn timezone(&self) -> Option<&str> {
        self.get(keys::TIMEZONE).and_then(|v| v.as_str())
    }

    // =========== Generic Key-Value Operations ===========

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
    /// Serializes the value to JSON before storing.
    pub fn insert_typed<T: Serialize>(
        &mut self,
        key: impl Into<String>,
        value: &T,
    ) -> crate::Result<()> {
        let json_value = serde_json::to_value(value).map_err(|e| {
            crate::Error::ProcessingFailed(format!("Failed to serialize context value: {e}"))
        })?;
        self.data.insert(key.into(), json_value);
        Ok(())
    }

    /// Get a typed value from the context
    ///
    /// Deserializes the JSON value into the specified type.
    pub fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> crate::Result<Option<T>> {
        match self.data.get(key) {
            None => Ok(None),
            Some(value) => {
                let typed = serde_json::from_value(value.clone()).map_err(|e| {
                    crate::Error::ProcessingFailed(format!(
                        "Failed to deserialize context value: {e}"
                    ))
                })?;
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

    /// Merge another context into this one (other values override)
    pub fn merge(&mut self, other: Context) {
        self.data.extend(other.data);
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
    fn test_language() {
        let ctx = Context::new().with_language("en");
        assert_eq!(ctx.language(), Some("en"));

        let mut ctx2 = Context::new();
        ctx2.set_language("zh");
        assert_eq!(ctx2.language(), Some("zh"));
    }

    #[test]
    fn test_session_id() {
        let ctx = Context::new().with_session_id("sess-123");
        assert_eq!(ctx.session_id(), Some("sess-123"));
    }

    #[test]
    fn test_builder_chain() {
        let ctx = Context::new()
            .with_language("en")
            .with_session_id("sess-123")
            .with_user_id("user-456")
            .with_timezone("Asia/Shanghai");

        assert_eq!(ctx.language(), Some("en"));
        assert_eq!(ctx.session_id(), Some("sess-123"));
        assert_eq!(ctx.user_id(), Some("user-456"));
        assert_eq!(ctx.timezone(), Some("Asia/Shanghai"));
    }

    #[test]
    fn test_merge() {
        let mut ctx1 = Context::new().with_language("en");
        let ctx2 = Context::new().with_language("zh").with_session_id("sess");

        ctx1.merge(ctx2);
        assert_eq!(ctx1.language(), Some("zh")); // overridden
        assert_eq!(ctx1.session_id(), Some("sess")); // merged
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
