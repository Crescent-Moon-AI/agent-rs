//! Tool definition types for LLM tool use

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Tool definition for LLM provider
///
/// This describes a tool that the LLM can use, including its name,
/// description, and input schema in JSON Schema format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (must match the tool in ToolRegistry)
    pub name: String,

    /// Description of what the tool does
    pub description: String,

    /// JSON schema for the tool's input parameters
    pub input_schema: Value,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// Helper module to build JSON schemas for tools
pub mod schema {
    use serde_json::{json, Value};

    /// Create a JSON schema for an object with properties
    ///
    /// # Example
    ///
    /// ```
    /// use agent_llm::tools::schema;
    /// use serde_json::json;
    ///
    /// let schema = schema::object(
    ///     json!({
    ///         "query": schema::string("Search query"),
    ///         "limit": schema::number("Maximum results"),
    ///     }),
    ///     vec!["query"],
    /// );
    /// ```
    pub fn object(properties: Value, required: Vec<&str>) -> Value {
        json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }

    /// String property schema
    ///
    /// # Example
    ///
    /// ```
    /// use agent_llm::tools::schema;
    ///
    /// let schema = schema::string("A text description");
    /// ```
    pub fn string(description: &str) -> Value {
        json!({
            "type": "string",
            "description": description,
        })
    }

    /// Number property schema
    ///
    /// # Example
    ///
    /// ```
    /// use agent_llm::tools::schema;
    ///
    /// let schema = schema::number("A numeric value");
    /// ```
    pub fn number(description: &str) -> Value {
        json!({
            "type": "number",
            "description": description,
        })
    }

    /// Integer property schema
    pub fn integer(description: &str) -> Value {
        json!({
            "type": "integer",
            "description": description,
        })
    }

    /// Boolean property schema
    pub fn boolean(description: &str) -> Value {
        json!({
            "type": "boolean",
            "description": description,
        })
    }

    /// Array property schema
    pub fn array(description: &str, items: Value) -> Value {
        json!({
            "type": "array",
            "description": description,
            "items": items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definition_creation() {
        let schema = schema::object(
            json!({
                "query": schema::string("Search query"),
            }),
            vec!["query"],
        );

        let tool = ToolDefinition::new("search", "Search the web", schema.clone());
        assert_eq!(tool.name, "search");
        assert_eq!(tool.description, "Search the web");
        assert_eq!(tool.input_schema, schema);
    }

    #[test]
    fn test_schema_builders() {
        let str_schema = schema::string("test");
        assert_eq!(str_schema["type"], "string");

        let num_schema = schema::number("count");
        assert_eq!(num_schema["type"], "number");

        let bool_schema = schema::boolean("flag");
        assert_eq!(bool_schema["type"], "boolean");
    }
}
