//! MCP parameter validation utilities.

use crate::types::{ForgeError, ForgeResult};
use serde_json::Value;

pub struct McpValidator;

impl McpValidator {
    pub fn require_string(params: &Value, field: &str) -> ForgeResult<String> {
        params.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }

    pub fn optional_string(params: &Value, field: &str) -> Option<String> {
        params.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn require_u64(params: &Value, field: &str) -> ForgeResult<u64> {
        params.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }

    pub fn optional_u64(params: &Value, field: &str) -> Option<u64> {
        params.get(field).and_then(|v| v.as_u64())
    }

    pub fn require_f64(params: &Value, field: &str) -> ForgeResult<f64> {
        params.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }

    pub fn optional_f64(params: &Value, field: &str) -> Option<f64> {
        params.get(field).and_then(|v| v.as_f64())
    }

    pub fn require_bool(params: &Value, field: &str) -> ForgeResult<bool> {
        params.get(field)
            .and_then(|v| v.as_bool())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }

    pub fn optional_bool(params: &Value, field: &str) -> Option<bool> {
        params.get(field).and_then(|v| v.as_bool())
    }

    pub fn require_array<'a>(params: &'a Value, field: &str) -> ForgeResult<&'a Vec<Value>> {
        params.get(field)
            .and_then(|v| v.as_array())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }

    pub fn optional_array<'a>(params: &'a Value, field: &str) -> Option<&'a Vec<Value>> {
        params.get(field).and_then(|v| v.as_array())
    }

    pub fn require_object<'a>(params: &'a Value, field: &str) -> ForgeResult<&'a serde_json::Map<String, Value>> {
        params.get(field)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ForgeError::MissingField(field.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_require_string() {
        let params = json!({"name": "test"});
        assert_eq!(McpValidator::require_string(&params, "name").unwrap(), "test");
        assert!(McpValidator::require_string(&params, "missing").is_err());
    }

    #[test]
    fn test_optional_string() {
        let params = json!({"name": "test"});
        assert_eq!(McpValidator::optional_string(&params, "name"), Some("test".into()));
        assert_eq!(McpValidator::optional_string(&params, "missing"), None);
    }

    #[test]
    fn test_require_u64() {
        let params = json!({"count": 42});
        assert_eq!(McpValidator::require_u64(&params, "count").unwrap(), 42);
        assert!(McpValidator::require_u64(&params, "missing").is_err());
    }

    #[test]
    fn test_require_f64() {
        let params = json!({"score": 3.14});
        let val = McpValidator::require_f64(&params, "score").unwrap();
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_require_bool() {
        let params = json!({"active": true});
        assert!(McpValidator::require_bool(&params, "active").unwrap());
        assert!(McpValidator::require_bool(&params, "missing").is_err());
    }

    #[test]
    fn test_require_array() {
        let params = json!({"items": [1, 2, 3]});
        assert_eq!(McpValidator::require_array(&params, "items").unwrap().len(), 3);
    }

    #[test]
    fn test_require_object() {
        let params = json!({"meta": {"key": "value"}});
        let obj = McpValidator::require_object(&params, "meta").unwrap();
        assert!(obj.contains_key("key"));
    }

    #[test]
    fn test_optional_u64() {
        let params = json!({"count": 5});
        assert_eq!(McpValidator::optional_u64(&params, "count"), Some(5));
        assert_eq!(McpValidator::optional_u64(&params, "missing"), None);
    }

    #[test]
    fn test_optional_f64() {
        let params = json!({"score": 2.5});
        assert!(McpValidator::optional_f64(&params, "score").is_some());
        assert!(McpValidator::optional_f64(&params, "missing").is_none());
    }

    #[test]
    fn test_optional_bool() {
        let params = json!({"flag": false});
        assert_eq!(McpValidator::optional_bool(&params, "flag"), Some(false));
    }

    #[test]
    fn test_optional_array() {
        let params = json!({"tags": ["a", "b"]});
        assert!(McpValidator::optional_array(&params, "tags").is_some());
        assert!(McpValidator::optional_array(&params, "missing").is_none());
    }
}
