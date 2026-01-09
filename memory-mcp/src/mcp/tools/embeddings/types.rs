//! Embedding tool types and input/output structures.

use serde::{Deserialize, Serialize};

/// Input parameters for configuring embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureEmbeddingsInput {
    /// Embedding provider to use
    pub provider: String,
    /// Model name (e.g., text-embedding-3-small, all-MiniLM-L6-v2)
    pub model: Option<String>,
    /// Environment variable name for API key (if cloud provider)
    pub api_key_env: Option<String>,
    /// Similarity threshold for searches (0.0-1.0)
    pub similarity_threshold: Option<f32>,
    /// Batch size for embedding generation
    pub batch_size: Option<usize>,
    /// Custom base URL for API endpoint
    pub base_url: Option<String>,
    /// API version (for Azure)
    pub api_version: Option<String>,
    /// Azure resource name
    pub resource_name: Option<String>,
    /// Azure deployment name
    pub deployment_name: Option<String>,
}

/// Output from configuring embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureEmbeddingsOutput {
    /// Whether configuration was successful
    pub success: bool,
    /// Provider that was configured
    pub provider: String,
    /// Model that was configured
    pub model: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Configuration message
    pub message: String,
    /// Warnings (if any)
    pub warnings: Vec<String>,
}

/// Input parameters for semantic memory query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySemanticMemoryInput {
    /// Natural language query
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<usize>,
    /// Minimum similarity score (0.0-1.0)
    pub similarity_threshold: Option<f32>,
    /// Domain filter (optional)
    pub domain: Option<String>,
    /// Task type filter (optional)
    pub task_type: Option<String>,
}

/// Output from semantic memory query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySemanticMemoryOutput {
    /// Number of results found
    pub results_found: usize,
    /// Results with similarity scores
    pub results: Vec<SemanticResult>,
    /// Query embedding dimension
    pub embedding_dimension: usize,
    /// Query processing time in milliseconds
    pub query_time_ms: f64,
    /// Provider used for embeddings
    pub provider: String,
}

/// A semantic search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticResult {
    /// Episode ID
    pub episode_id: String,
    /// Similarity score (0.0-1.0)
    pub similarity_score: f32,
    /// Task description
    pub task_description: String,
    /// Domain
    pub domain: String,
    /// Task type
    pub task_type: String,
    /// Outcome summary
    pub outcome: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

/// Output from embedding provider test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEmbeddingsOutput {
    /// Whether provider is available
    pub available: bool,
    /// Provider name
    pub provider: String,
    /// Model name
    pub model: String,
    /// Embedding dimension
    pub dimension: usize,
    /// Test query time in milliseconds
    pub test_time_ms: u64,
    /// Sample embedding (first 5 values)
    pub sample_embedding: Vec<f32>,
    /// Test status message
    pub message: String,
    /// Errors (if any)
    pub errors: Vec<String>,
}
