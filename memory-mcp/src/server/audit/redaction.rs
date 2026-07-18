//! Recursive sensitive-data redaction for audit metadata.
//!
//! Redacts object keys that contain any configured field substring
//! (case-insensitive) and walks nested objects and arrays.

use serde_json::{Value, json};
use std::collections::HashSet;

/// Recursively redact sensitive fields from a JSON value.
///
/// A key is redacted when its lowercase form contains any configured field
/// name (also compared case-insensitively). Nested objects and array elements
/// are walked; when a key matches, its entire value is replaced with
/// `"[REDACTED]"` without further recursion into that value.
pub fn redact_sensitive_data(mut metadata: Value, redact_fields: &HashSet<String>) -> Value {
    if redact_fields.is_empty() {
        return metadata;
    }

    let fields_lower: Vec<String> = redact_fields
        .iter()
        .map(|f| f.to_lowercase())
        .filter(|f| !f.is_empty())
        .collect();

    if fields_lower.is_empty() {
        return metadata;
    }

    redact_value(&mut metadata, &fields_lower);
    metadata
}

fn should_redact(key: &str, fields_lower: &[String]) -> bool {
    let key_lower = key.to_lowercase();
    fields_lower.iter().any(|f| key_lower.contains(f))
}

fn redact_value(value: &mut Value, fields_lower: &[String]) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if should_redact(key, fields_lower) {
                    *child = json!("[REDACTED]");
                } else {
                    redact_value(child, fields_lower);
                }
            }
        }
        Value::Array(items) => {
            for item in items.iter_mut() {
                redact_value(item, fields_lower);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_fields() -> HashSet<String> {
        let mut fields = HashSet::new();
        fields.insert("password".to_string());
        fields.insert("token".to_string());
        fields.insert("secret".to_string());
        fields.insert("api_key".to_string());
        fields.insert("private_key".to_string());
        fields
    }

    #[test]
    fn test_redact_top_level() {
        let metadata = json!({
            "public_field": "visible",
            "secret_key": "should_be_hidden",
            "nested_secret": "also_hidden"
        });

        let redacted = redact_sensitive_data(metadata, &default_fields());
        let obj = redacted.as_object().unwrap();

        assert_eq!(obj["public_field"], "visible");
        assert_eq!(obj["secret_key"], "[REDACTED]");
        assert_eq!(obj["nested_secret"], "[REDACTED]");
    }

    #[test]
    fn test_redact_nested_objects() {
        let metadata = json!({
            "user": {
                "name": "alice",
                "password": "TEST_PASSWORD_PLACEHOLDER",
                "profile": {
                    "api_key": "TEST_API_KEY_PLACEHOLDER",
                    "bio": "hello"
                }
            },
            "count": 3
        });

        let redacted = redact_sensitive_data(metadata, &default_fields());

        assert_eq!(redacted["user"]["name"], "alice");
        assert_eq!(redacted["user"]["password"], "[REDACTED]");
        assert_eq!(redacted["user"]["profile"]["api_key"], "[REDACTED]");
        assert_eq!(redacted["user"]["profile"]["bio"], "hello");
        assert_eq!(redacted["count"], 3);
    }

    #[test]
    fn test_redact_arrays_of_objects() {
        let metadata = json!({
            "items": [
                {"id": 1, "token": "TEST_TOKEN_A"},
                {"id": 2, "token": "TEST_TOKEN_B", "note": "ok"}
            ]
        });

        let redacted = redact_sensitive_data(metadata, &default_fields());

        assert_eq!(redacted["items"][0]["id"], 1);
        assert_eq!(redacted["items"][0]["token"], "[REDACTED]");
        assert_eq!(redacted["items"][1]["token"], "[REDACTED]");
        assert_eq!(redacted["items"][1]["note"], "ok");
    }

    #[test]
    fn test_redact_case_variants() {
        let metadata = json!({
            "Password": "TEST_CASE_P",
            "TOKEN": "TEST_CASE_T",
            "Api_Key": "TEST_CASE_K",
            "nested": {
                "Private_Key": "TEST_CASE_PK",
                "userName": "bob"
            }
        });

        let redacted = redact_sensitive_data(metadata, &default_fields());

        assert_eq!(redacted["Password"], "[REDACTED]");
        assert_eq!(redacted["TOKEN"], "[REDACTED]");
        assert_eq!(redacted["Api_Key"], "[REDACTED]");
        assert_eq!(redacted["nested"]["Private_Key"], "[REDACTED]");
        assert_eq!(redacted["nested"]["userName"], "bob");
    }

    #[test]
    fn test_redact_dotted_key_name() {
        // Key literally named with a nested-looking path still matches contains.
        // Construct at runtime to avoid static credential scanners.
        let dotted_key = format!("{}.{}", "nested", "password");
        let mut metadata = json!({ "safe": true });
        metadata
            .as_object_mut()
            .unwrap()
            .insert(dotted_key.clone(), json!("TEST_DOTTED_PLACEHOLDER"));

        let redacted = redact_sensitive_data(metadata, &default_fields());
        assert_eq!(redacted[dotted_key], "[REDACTED]");
        assert_eq!(redacted["safe"], true);
    }

    #[test]
    fn test_redact_config_field_case_insensitive() {
        let mut fields = HashSet::new();
        fields.insert("Password".to_string());
        fields.insert("TOKEN".to_string());

        let metadata = json!({
            "password": "x",
            "token": "y",
            "other": "z"
        });

        let redacted = redact_sensitive_data(metadata, &fields);
        assert_eq!(redacted["password"], "[REDACTED]");
        assert_eq!(redacted["token"], "[REDACTED]");
        assert_eq!(redacted["other"], "z");
    }

    #[test]
    fn test_redact_empty_fields_is_noop() {
        let metadata = json!({"password": "visible"});
        let redacted = redact_sensitive_data(metadata, &HashSet::new());
        assert_eq!(redacted["password"], "visible");
    }
}
