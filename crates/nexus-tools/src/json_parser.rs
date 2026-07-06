use serde::de::DeserializeOwned;
use serde_json;
use thiserror::Error;

/// Errors that can occur during JSON parsing.
#[derive(Error, Debug)]
pub enum JsonParseError {
    /// The input was empty (no bytes to parse).
    #[error("input is empty — expected non-empty JSON")]
    EmptyInput,

    /// The JSON is syntactically malformed.
    #[error("malformed JSON: {0}")]
    Malformed(#[from] serde_json::Error),

    /// A required field was missing or had the wrong type.
    #[error("validation error: {0}")]
    Validation(String),
}

/// Parse a JSON string into a strongly-typed struct.
///
/// Handles all error cases:
/// - Empty input (empty string or whitespace-only)
/// - Malformed JSON (syntax errors)
/// - Missing or invalid fields (via serde validation)
/// - Type mismatches (via serde's type checking)
///
/// # Example
///
/// ```rust
/// use serde::Deserialize;
/// use nexus_tools::json_parser::parse_json;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Config {
///     name: String,
///     version: u32,
/// }
///
/// let json = r#"{"name": "nexus", "version": 1}"#;
/// let config: Config = parse_json(json).unwrap();
/// assert_eq!(config.name, "nexus");
/// ```
pub fn parse_json<T: DeserializeOwned>(input: &str) -> Result<T, JsonParseError> {
    let trimmed = input.trim();

    // Reject empty input
    if trimmed.is_empty() {
        return Err(JsonParseError::EmptyInput);
    }

    // Parse JSON — serde_json::from_str handles:
    // - Malformed syntax (trailing commas, invalid tokens, etc.)
    // - Type mismatches (string vs number, etc.)
    // - Missing required fields (if struct uses #[serde(deny_unknown_fields)])
    // - Unexpected fields (if struct uses #[serde(deny_unknown_fields)])
    serde_json::from_str(trimmed).map_err(JsonParseError::Malformed)
}

/// Parse JSON bytes into a strongly-typed struct.
///
/// Same as [`parse_json`] but accepts `&[u8]` instead of `&str`.
pub fn parse_json_bytes<T: DeserializeOwned>(input: &[u8]) -> Result<T, JsonParseError> {
    if input.is_empty() {
        return Err(JsonParseError::EmptyInput);
    }

    serde_json::from_slice(input).map_err(JsonParseError::Malformed)
}

/// Parse a JSON value (serde_json::Value) into a struct, with additional
/// validation beyond what serde provides.
///
/// This is useful when you need to check semantic constraints (e.g., ranges,
/// string lengths) after structural parsing.
pub fn parse_json_with_validation<T, F>(input: &str, validator: F) -> Result<T, JsonParseError>
where
    T: DeserializeOwned,
    F: FnOnce(&T) -> Result<(), String>,
{
    let parsed: T = parse_json(input)?;
    validator(&parsed).map_err(JsonParseError::Validation)?;
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct TestStruct {
        name: String,
        version: u32,
        #[serde(default)]
        enabled: bool,
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"name": "nexus", "version": 1}"#;
        let result: TestStruct = parse_json(json).unwrap();
        assert_eq!(result.name, "nexus");
        assert_eq!(result.version, 1);
        assert!(!result.enabled); // default is false
    }

    #[test]
    fn test_parse_with_defaults() {
        let json = r#"{"name": "test", "version": 42, "enabled": true}"#;
        let result: TestStruct = parse_json(json).unwrap();
        assert_eq!(result.name, "test");
        assert_eq!(result.version, 42);
        assert!(result.enabled);
    }

    #[test]
    fn test_reject_empty_input() {
        let result: Result<TestStruct, _> = parse_json("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JsonParseError::EmptyInput));

        let result: Result<TestStruct, _> = parse_json("   ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JsonParseError::EmptyInput));
    }

    #[test]
    fn test_reject_malformed_json() {
        let result: Result<TestStruct, _> = parse_json("{invalid}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JsonParseError::Malformed(_)));

        let result: Result<TestStruct, _> = parse_json(r#"{"name": "nexus", version: 1}"#);
        assert!(result.is_err());

        let result: Result<TestStruct, _> = parse_json(r#"{"name": }"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_reject_type_mismatch() {
        let result: Result<TestStruct, _> = parse_json(r#"{"name": "nexus", "version": "not-a-number"}"#);
        assert!(result.is_err());
        // serde_json returns a type error
        let err = result.unwrap_err();
        assert!(matches!(err, JsonParseError::Malformed(_)));
        // Error message may contain "version" or "string" or "number" depending on serde version
        let msg = err.to_string();
        assert!(msg.contains("version") || msg.contains("string") || msg.contains("number") || msg.contains("type"),
            "Error should mention the type mismatch: {}", msg);
    }

    #[test]
    fn test_reject_unknown_fields() {
        let result: Result<TestStruct, _> =
            parse_json(r#"{"name": "nexus", "version": 1, "extra": "boom"}"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, JsonParseError::Malformed(_)));
        assert!(err.to_string().contains("extra"));
    }

    #[test]
    fn test_parse_json_bytes() {
        let json = b"{\"name\": \"nexus\", \"version\": 1}";
        let result: TestStruct = parse_json_bytes(json).unwrap();
        assert_eq!(result.name, "nexus");
    }

    #[test]
    fn test_parse_json_bytes_empty() {
        let result: Result<TestStruct, _> = parse_json_bytes(b"");
        assert!(matches!(result.unwrap_err(), JsonParseError::EmptyInput));
    }

    #[test]
    fn test_parse_with_validation() {
        let json = r#"{"name": "nexus", "version": 1}"#;
        let result: TestStruct = parse_json_with_validation(json, |s: &TestStruct| {
            if s.version > 0 {
                Ok(())
            } else {
                Err("version must be positive".to_string())
            }
        })
        .unwrap();
        assert_eq!(result.name, "nexus");

        // Failing validation
        let result: Result<TestStruct, _> = parse_json_with_validation(json, |s: &TestStruct| {
            if s.name.len() < 3 {
                Err("name too short".to_string())
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok()); // name is 5 chars, passes
    }

    #[test]
    fn test_validation_failure() {
        let json = r#"{"name": "a", "version": 1}"#;
        let result: Result<TestStruct, _> = parse_json_with_validation(json, |s: &TestStruct| {
            if s.name.len() > 2 {
                Ok(())
            } else {
                Err("name must be longer than 2 characters".to_string())
            }
        });
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JsonParseError::Validation(_)));
    }

    #[test]
    fn test_error_display() {
        let err = JsonParseError::EmptyInput;
        assert_eq!(format!("{}", err), "input is empty — expected non-empty JSON");

        let err = JsonParseError::Validation("test".to_string());
        assert_eq!(format!("{}", err), "validation error: test");

        let err = JsonParseError::Malformed(serde_json::from_str::<TestStruct>("").unwrap_err());
        assert!(format!("{}", err).contains("EOF"));
    }
}