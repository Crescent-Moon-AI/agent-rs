//! Tool trait definition

use async_trait::async_trait;
use agent_core::Result;
use serde_json::Value;

/// Trait for tools that agents can execute
///
/// Tools are functions that LLM agents can call to interact with the world.
/// Each tool must provide a name, description, and JSON schema for its input.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Execute the tool with given parameters
    ///
    /// # Arguments
    ///
    /// * `params` - Tool input as JSON value (should match input_schema)
    ///
    /// # Returns
    ///
    /// Tool output as JSON value
    async fn execute(&self, params: Value) -> Result<Value>;

    /// Get the tool's name
    ///
    /// Must be unique within a ToolRegistry and match the name in ToolDefinition
    fn name(&self) -> &str;

    /// Get the tool's description
    ///
    /// This description helps the LLM understand when to use this tool
    fn description(&self) -> &str;

    /// Get the tool's input schema (JSON Schema format)
    ///
    /// Describes the parameters this tool expects. The LLM uses this schema
    /// to generate valid tool calls.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_json::json;
    ///
    /// // Example schema for a calculator tool:
    /// let schema = json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "operation": {
    ///             "type": "string",
    ///             "enum": ["add", "subtract", "multiply", "divide"]
    ///         },
    ///         "a": { "type": "number" },
    ///         "b": { "type": "number" }
    ///     },
    ///     "required": ["operation", "a", "b"]
    /// });
    /// ```
    fn input_schema(&self) -> Value;
}
