//! Field projection and selection for query result optimization
//!
//! This module provides field-level filtering for query results, allowing clients
//! to request only the fields they need, significantly reducing output token usage.
//!
//! ## Features
//!
//! - **Generic Field Selection**: Works with any serializable type
//! - **Nested Field Support**: Select nested fields (e.g., "episode.id", "episode.task_description")
//! - **Whitelist-Based**: Only returns explicitly requested fields
//! - **Backward Compatible**: No `fields` parameter = return all fields
//!
//! ## Usage
//!
//! ```rust
//! # use anyhow::Result;
//! # fn main() -> Result<()> {
//! use memory_mcp::server::tools::field_projection::FieldSelector;
//! use serde_json::json;
//!
//! // Create selector from request
//! let args = json!({"fields": ["episode.id", "episode.task_description"]});
//! let selector = FieldSelector::from_request(&args);
//!
//! // Apply to result
//! let result = json!({"episode": {"id": "123", "task_description": "test"}});
//! let filtered = selector.apply(&result)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Token Savings
//!
//! - **Before**: Full object returned (~500 tokens)
//! - **After**: Selected fields only (~200 tokens, 60% reduction)

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;
use tracing::{debug, trace};

/// Field selector for filtering query results
#[derive(Debug, Clone)]
pub struct FieldSelector {
    /// Set of field paths to select (e.g., "episode.id", "episode.task_description")
    allowed_fields: Option<HashSet<String>>,
    /// Whether to return all fields (backward compatibility mode)
    return_all: bool,
}

impl FieldSelector {
    /// Create a new field selector with specific allowed fields
    ///
    /// # Arguments
    ///
    /// * `fields` - Set of field paths to allow (e.g., "episode.id", "episode.task_description")
    pub fn new(fields: HashSet<String>) -> Self {
        if fields.is_empty() {
            Self {
                allowed_fields: None,
                return_all: true,
            }
        } else {
            Self {
                allowed_fields: Some(fields),
                return_all: false,
            }
        }
    }

    /// Create a field selector from request arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Request arguments that may contain a `fields` parameter
    ///
    /// # Returns
    ///
    /// Returns a FieldSelector. If no `fields` parameter is present,
    /// returns a selector that allows all fields (backward compatible).
    pub fn from_request(args: &Value) -> Self {
        match args.get("fields") {
            Some(Value::Array(fields)) => {
                let field_set: HashSet<String> = fields
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if field_set.is_empty() {
                    debug!("Empty fields array, returning all fields");
                    Self {
                        allowed_fields: None,
                        return_all: true,
                    }
                } else {
                    debug!(
                        "Field selector created with {} fields: {:?}",
                        field_set.len(),
                        field_set
                    );
                    Self {
                        allowed_fields: Some(field_set),
                        return_all: false,
                    }
                }
            }
            Some(_) => {
                debug!("Invalid fields parameter type, returning all fields");
                Self {
                    allowed_fields: None,
                    return_all: true,
                }
            }
            None => {
                trace!("No fields parameter, returning all fields (backward compatible)");
                Self {
                    allowed_fields: None,
                    return_all: true,
                }
            }
        }
    }

    /// Apply field selection to a serializable value
    ///
    /// # Arguments
    ///
    /// * `value` - Value to filter (must implement Serialize)
    ///
    /// # Returns
    ///
    /// Returns filtered JSON value
    pub fn apply<T: Serialize>(&self, value: &T) -> Result<Value> {
        let full = serde_json::to_value(value)?;

        if self.return_all {
            return Ok(full);
        }

        let allowed = match self.allowed_fields.as_ref() {
            Some(fields) => fields,
            None => return Ok(full),
        };
        Ok(self.filter_value(&full, allowed, ""))
    }

    /// Filter a JSON value based on allowed field paths
    ///
    /// # Arguments
    ///
    /// * `value` - JSON value to filter
    /// * `allowed` - Set of allowed field paths
    /// * `prefix` - Current path prefix for nested fields
    fn filter_value(&self, value: &Value, allowed: &HashSet<String>, prefix: &str) -> Value {
        match value {
            Value::Object(map) => self.filter_object(map, allowed, prefix),
            Value::Array(arr) => self.filter_array(arr, allowed, prefix),
            _ => value.clone(),
        }
    }

    /// Filter an object based on allowed field paths
    fn filter_object(
        &self,
        map: &serde_json::Map<String, Value>,
        allowed: &HashSet<String>,
        prefix: &str,
    ) -> Value {
        let mut result = serde_json::Map::new();

        for (key, value) in map.iter() {
            let full_path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            // Check if this field or any of its children are allowed
            let field_allowed = allowed.contains(&full_path);
            let child_allowed = allowed
                .iter()
                .any(|f| f.starts_with(&format!("{}.", full_path)));

            if field_allowed || child_allowed {
                if field_allowed {
                    // Field is explicitly allowed, include it as-is
                    result.insert(key.clone(), value.clone());
                } else if child_allowed {
                    // Child fields are allowed, recurse
                    let filtered = self.filter_value(value, allowed, &full_path);
                    if !filtered.is_null() {
                        result.insert(key.clone(), filtered);
                    }
                }
            }
        }

        Value::Object(result)
    }

    /// Filter an array based on allowed field paths
    fn filter_array(&self, arr: &[Value], allowed: &HashSet<String>, prefix: &str) -> Value {
        let filtered: Vec<Value> = arr
            .iter()
            .map(|item| self.filter_value(item, allowed, prefix))
            .collect();

        Value::Array(filtered)
    }

    /// Check if a specific field path is allowed
    pub fn is_field_allowed(&self, path: &str) -> bool {
        if self.return_all {
            return true;
        }

        match &self.allowed_fields {
            None => true,
            Some(allowed) => {
                // Exact match or parent path match
                allowed.contains(path)
                    || allowed.iter().any(|f| path.starts_with(&format!("{}.", f)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_no_fields_returns_all() {
        let args = json!({});
        let selector = FieldSelector::from_request(&args);

        let result = json!({"id": "123", "name": "test", "nested": {"value": 42}});
        let filtered = selector.apply(&result).unwrap();

        assert_eq!(filtered, result);
    }

    #[test]
    fn test_simple_field_selection() {
        let args = json!({"fields": ["id", "name"]});
        let selector = FieldSelector::from_request(&args);

        let result = json!({"id": "123", "name": "test", "extra": "ignored"});
        let filtered = selector.apply(&result).unwrap();

        assert_eq!(filtered["id"], "123");
        assert_eq!(filtered["name"], "test");
        assert!(!filtered.as_object().unwrap().contains_key("extra"));
    }

    #[test]
    fn test_nested_field_selection() {
        let args = json!({"fields": ["episode.id", "episode.task_description"]});
        let selector = FieldSelector::from_request(&args);

        let result = json!({
            "episode": {
                "id": "123",
                "task_description": "test task",
                "steps": ["step1", "step2"],
                "outcome": {"type": "success"}
            }
        });
        let filtered = selector.apply(&result).unwrap();

        assert_eq!(filtered["episode"]["id"], "123");
        assert_eq!(filtered["episode"]["task_description"], "test task");
        assert!(
            !filtered["episode"]
                .as_object()
                .unwrap()
                .contains_key("steps")
        );
    }

    #[test]
    fn test_array_field_selection() {
        let args = json!({"fields": ["episodes.id", "episodes.task_description"]});
        let selector = FieldSelector::from_request(&args);

        let result = json!({
            "episodes": [
                {"id": "1", "task_description": "task1", "extra": "data1"},
                {"id": "2", "task_description": "task2", "extra": "data2"}
            ]
        });
        let filtered = selector.apply(&result).unwrap();

        assert_eq!(filtered["episodes"].as_array().unwrap().len(), 2);
        assert_eq!(filtered["episodes"][0]["id"], "1");
        assert_eq!(filtered["episodes"][0]["task_description"], "task1");
        assert!(
            !filtered["episodes"][0]
                .as_object()
                .unwrap()
                .contains_key("extra")
        );
    }

    #[test]
    fn test_empty_fields_array_returns_all() {
        let args = json!({"fields": []});
        let selector = FieldSelector::from_request(&args);

        assert!(selector.return_all);
    }

    #[test]
    fn test_is_field_allowed() {
        let selector = FieldSelector::new(
            vec![
                "episode.id".to_string(),
                "episode.task_description".to_string(),
            ]
            .into_iter()
            .collect(),
        );

        assert!(selector.is_field_allowed("episode.id"));
        assert!(selector.is_field_allowed("episode.task_description"));
        assert!(!selector.is_field_allowed("episode.steps"));
    }

    #[test]
    fn test_complex_nested_structure() {
        let args = json!({"fields": [
            "episodes.id",
            "episodes.task_description",
            "episodes.outcome.type",
            "patterns.success_rate"
        ]});
        let selector = FieldSelector::from_request(&args);

        let result = json!({
            "episodes": [
                {
                    "id": "1",
                    "task_description": "task1",
                    "steps": ["s1", "s2"],
                    "outcome": {"type": "success", "verdict": "good"}
                }
            ],
            "patterns": [
                {"success_rate": 0.9, "description": "pattern1"}
            ],
            "insights": {"total": 10}
        });
        let filtered = selector.apply(&result).unwrap();

        // Verify episodes filtered correctly
        assert_eq!(filtered["episodes"][0]["id"], "1");
        assert_eq!(filtered["episodes"][0]["task_description"], "task1");
        assert_eq!(filtered["episodes"][0]["outcome"]["type"], "success");
        assert!(
            !filtered["episodes"][0]
                .as_object()
                .unwrap()
                .contains_key("steps")
        );
        assert!(
            !filtered["episodes"][0]["outcome"]
                .as_object()
                .unwrap()
                .contains_key("verdict")
        );

        // Verify patterns filtered correctly
        assert_eq!(filtered["patterns"][0]["success_rate"], 0.9);
        assert!(
            !filtered["patterns"][0]
                .as_object()
                .unwrap()
                .contains_key("description")
        );

        // Verify insights not included
        assert!(!filtered.as_object().unwrap().contains_key("insights"));
    }
}
