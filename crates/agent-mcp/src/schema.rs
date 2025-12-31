//! Schema conversion helpers
//!
//! Utilities for converting between MCP JSON schemas and agent-rs formats.

use serde_json::{Value, json};

/// Create a JSON Schema object type
///
/// # Arguments
///
/// * `properties` - Map of property names to their schemas
/// * `required` - List of required property names
///
/// # Example
///
/// ```
/// use agent_mcp::schema::{object, string, number};
/// use serde_json::json;
///
/// let schema = object(
///     json!({
///         "name": string("User's name"),
///         "age": number("User's age"),
///     }),
///     vec!["name"],
/// );
/// ```
pub fn object(properties: Value, required: Vec<&str>) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

/// Create a JSON Schema string type
///
/// # Arguments
///
/// * `description` - Optional description of the string field
pub fn string(description: Option<&str>) -> Value {
    if let Some(d) = description {
        json!({
            "type": "string",
            "description": d,
        })
    } else {
        json!({"type": "string"})
    }
}

/// Create a JSON Schema number type
///
/// # Arguments
///
/// * `description` - Optional description of the number field
pub fn number(description: Option<&str>) -> Value {
    if let Some(d) = description {
        json!({
            "type": "number",
            "description": d,
        })
    } else {
        json!({"type": "number"})
    }
}

/// Create a JSON Schema integer type
///
/// # Arguments
///
/// * `description` - Optional description of the integer field
pub fn integer(description: Option<&str>) -> Value {
    if let Some(d) = description {
        json!({
            "type": "integer",
            "description": d,
        })
    } else {
        json!({"type": "integer"})
    }
}

/// Create a JSON Schema boolean type
///
/// # Arguments
///
/// * `description` - Optional description of the boolean field
pub fn boolean(description: Option<&str>) -> Value {
    if let Some(d) = description {
        json!({
            "type": "boolean",
            "description": d,
        })
    } else {
        json!({"type": "boolean"})
    }
}

/// Create a JSON Schema array type
///
/// # Arguments
///
/// * `items` - Schema for array items
/// * `description` - Optional description of the array field
pub fn array(items: Value, description: Option<&str>) -> Value {
    if let Some(d) = description {
        json!({
            "type": "array",
            "items": items,
            "description": d,
        })
    } else {
        json!({
            "type": "array",
            "items": items,
        })
    }
}

/// Create an enum schema (string with allowed values)
///
/// # Arguments
///
/// * `values` - List of allowed string values
/// * `description` - Optional description
pub fn enum_string(values: Vec<&str>, description: Option<&str>) -> Value {
    let values: Vec<String> = values.into_iter().map(std::string::ToString::to_string).collect();

    if let Some(d) = description {
        json!({
            "type": "string",
            "enum": values,
            "description": d,
        })
    } else {
        json!({
            "type": "string",
            "enum": values,
        })
    }
}

/// Validate that a value matches a JSON schema (basic validation)
///
/// This is a simple validator - for production use, consider using
/// a full JSON Schema validation library.
///
/// # Arguments
///
/// * `value` - The value to validate
/// * `schema` - The JSON schema to validate against
///
/// # Returns
///
/// `true` if the value matches the schema, `false` otherwise
pub fn validate_basic(value: &Value, schema: &Value) -> bool {
    // Get the type from schema
    let schema_type = match schema.get("type") {
        Some(Value::String(t)) => t.as_str(),
        _ => return true, // No type constraint, accept anything
    };

    // Check type match
    match schema_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => true, // Unknown type, accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        let schema = string(Some("A test string"));
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "A test string");

        let schema_no_desc = string(None);
        assert_eq!(schema_no_desc["type"], "string");
        assert!(schema_no_desc.get("description").is_none());
    }

    #[test]
    fn test_number_schema() {
        let schema = number(Some("A test number"));
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["description"], "A test number");
    }

    #[test]
    fn test_object_schema() {
        let schema = object(
            json!({
                "name": string(Some("Name")),
                "age": integer(Some("Age")),
            }),
            vec!["name"],
        );

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].is_object());
        assert!(schema["required"].is_array());
        assert_eq!(schema["required"][0], "name");
    }

    #[test]
    fn test_array_schema() {
        let schema = array(string(None), Some("List of names"));

        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
        assert_eq!(schema["description"], "List of names");
    }

    #[test]
    fn test_enum_schema() {
        let schema = enum_string(vec!["red", "green", "blue"], Some("Color choice"));

        assert_eq!(schema["type"], "string");
        assert!(schema["enum"].is_array());
        assert_eq!(schema["enum"].as_array().unwrap().len(), 3);
        assert_eq!(schema["description"], "Color choice");
    }

    #[test]
    fn test_validate_basic() {
        // String validation
        assert!(validate_basic(&json!("hello"), &string(None)));
        assert!(!validate_basic(&json!(42), &string(None)));

        // Number validation
        assert!(validate_basic(&json!(42), &number(None)));
        assert!(validate_basic(&json!(3.14), &number(None)));
        assert!(!validate_basic(&json!("hello"), &number(None)));

        // Boolean validation
        assert!(validate_basic(&json!(true), &boolean(None)));
        assert!(!validate_basic(&json!(1), &boolean(None)));

        // Array validation
        assert!(validate_basic(
            &json!([1, 2, 3]),
            &array(number(None), None)
        ));
        assert!(!validate_basic(
            &json!({"key": "value"}),
            &array(number(None), None)
        ));

        // Object validation
        let obj_schema = object(json!({"name": string(None)}), vec![]);
        assert!(validate_basic(&json!({"name": "test"}), &obj_schema));
        assert!(!validate_basic(&json!([1, 2]), &obj_schema));
    }

    #[test]
    fn test_complex_schema() {
        // Test a complex nested schema
        let schema = object(
            json!({
                "user": object(
                    json!({
                        "name": string(Some("Username")),
                        "age": integer(Some("User age")),
                        "active": boolean(Some("Is active")),
                    }),
                    vec!["name"],
                ),
                "tags": array(string(None), Some("User tags")),
                "role": enum_string(vec!["admin", "user", "guest"], Some("User role")),
            }),
            vec!["user", "role"],
        );

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["user"].is_object());
        assert!(schema["properties"]["tags"].is_object());
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&json!("user"))
        );
    }
}
