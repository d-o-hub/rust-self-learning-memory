//! OpenAI API request/response types.

use serde::{Deserialize, Serialize};

/// Input for embedding request (single text or batch)
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

/// OpenAI API request structure
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct EmbeddingRequest {
    pub input: EmbeddingInput,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<usize>,
}

/// OpenAI API response structure
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EmbeddingResponse {
    pub data: Vec<EmbeddingData>,
    #[allow(dead_code)]
    pub model: String,
    pub usage: Usage,
}

/// Individual embedding data from API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub index: usize,
    #[allow(dead_code)]
    pub object: String,
}

/// Token usage from API response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Usage {
    #[allow(dead_code)]
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}
