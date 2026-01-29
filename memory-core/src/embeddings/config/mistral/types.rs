//! Mistral API request/response types

use serde::{Deserialize, Serialize};

/// Input for Mistral embedding request (single text or batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)] // Used by MistralEmbeddingProvider
pub enum MistralEmbeddingInput {
    /// Single text input
    Single(String),
    /// Batch of text inputs
    Batch(Vec<String>),
}

impl From<String> for MistralEmbeddingInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<Vec<String>> for MistralEmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        Self::Batch(v)
    }
}

impl From<&str> for MistralEmbeddingInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<&str>> for MistralEmbeddingInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Batch(v.into_iter().map(String::from).collect())
    }
}

/// Mistral API request structure
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)] // Used by MistralEmbeddingProvider
pub struct MistralEmbeddingRequest {
    /// Input text(s) to embed
    pub inputs: MistralEmbeddingInput,
    /// Model ID
    pub model: String,
    /// Output data type (codestral-embed only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dtype: Option<String>,
    /// Output dimension (codestral-embed only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimension: Option<usize>,
}

/// Individual embedding data from Mistral API response
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used by MistralEmbeddingProvider
pub struct MistralEmbeddingData {
    /// The embedding vector (format depends on `output_dtype`)
    pub embedding: Vec<f32>,
    /// Index in the input batch
    pub index: usize,
    /// Object type (always "embedding")
    pub object: String,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used by MistralEmbeddingProvider
pub struct MistralUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
}

/// Mistral API response structure
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Used by MistralEmbeddingProvider
pub struct MistralEmbeddingResponse {
    /// List of embedding data
    pub data: Vec<MistralEmbeddingData>,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: MistralUsage,
    /// Object type (always "list")
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_embedding_input_conversions() {
        let single: MistralEmbeddingInput = "test".into();
        assert!(matches!(single, MistralEmbeddingInput::Single(_)));

        let batch: MistralEmbeddingInput = vec!["a", "b", "c"].into();
        assert!(matches!(batch, MistralEmbeddingInput::Batch(_)));
    }

    #[test]
    fn test_mistral_request_serialization() {
        let request = MistralEmbeddingRequest {
            inputs: MistralEmbeddingInput::Batch(vec!["hello".to_string(), "world".to_string()]),
            model: "codestral-embed".to_string(),
            output_dtype: Some("int8".to_string()),
            output_dimension: Some(512),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("codestral-embed"));
        assert!(json.contains("int8"));
        assert!(json.contains("512"));
    }

    #[test]
    fn test_mistral_request_serialization_without_optional() {
        let request = MistralEmbeddingRequest {
            inputs: MistralEmbeddingInput::Single("hello".to_string()),
            model: "mistral-embed".to_string(),
            output_dtype: None,
            output_dimension: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("mistral-embed"));
        assert!(!json.contains("output_dtype"));
        assert!(!json.contains("output_dimension"));
    }
}
