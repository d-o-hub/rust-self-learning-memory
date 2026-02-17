//! OpenAI API request/response types

use serde::{Deserialize, Serialize};

/// Input for embedding request (single text or batch)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIEmbeddingInput {
    /// Single text input
    Single(String),
    /// Batch of text inputs
    Batch(Vec<String>),
}

impl From<String> for OpenAIEmbeddingInput {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<Vec<String>> for OpenAIEmbeddingInput {
    fn from(v: Vec<String>) -> Self {
        Self::Batch(v)
    }
}

impl From<&str> for OpenAIEmbeddingInput {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<&str>> for OpenAIEmbeddingInput {
    fn from(v: Vec<&str>) -> Self {
        Self::Batch(v.into_iter().map(String::from).collect())
    }
}

/// OpenAI API request structure
#[derive(Debug, Clone, Serialize)]
pub struct OpenAIEmbeddingRequest {
    /// Input text(s) to embed
    pub input: OpenAIEmbeddingInput,
    /// Model ID
    pub model: String,
    /// Encoding format (float or base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    /// Number of dimensions (for text-embedding-3.x models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
}

/// Individual embedding data from API response
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIEmbeddingData {
    /// The embedding vector
    pub embedding: Vec<f32>,
    /// Index in the input batch
    pub index: usize,
    /// Object type (always "embedding")
    pub object: String,
}

/// Token usage information
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIUsage {
    /// Tokens in the prompt
    pub prompt_tokens: usize,
    /// Total tokens used
    pub total_tokens: usize,
}

/// OpenAI API response structure
#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIEmbeddingResponse {
    /// List of embedding data
    pub data: Vec<OpenAIEmbeddingData>,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: OpenAIUsage,
    /// Object type (always "list")
    pub object: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_input_conversions() {
        let single: OpenAIEmbeddingInput = "test".into();
        assert!(matches!(single, OpenAIEmbeddingInput::Single(_)));

        let batch: OpenAIEmbeddingInput = vec!["a", "b", "c"].into();
        assert!(matches!(batch, OpenAIEmbeddingInput::Batch(_)));
    }

    #[test]
    fn test_request_serialization() {
        let request = OpenAIEmbeddingRequest {
            input: OpenAIEmbeddingInput::Single("hello".to_string()),
            model: "text-embedding-3-small".to_string(),
            encoding_format: Some("float".to_string()),
            dimensions: Some(512),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("text-embedding-3-small"));
        assert!(json.contains("512"));
    }
}
