//! Embedding provider trait and common functionality

use anyhow::Result;
use async_trait::async_trait;

/// Result from embedding generation
#[derive(Debug, Clone)]
pub struct EmbeddingResult {
    /// The generated embedding vector
    pub embedding: Vec<f32>,
    /// Number of tokens processed
    pub token_count: Option<usize>,
    /// Model used for generation
    pub model: String,
    /// Generation time in milliseconds
    pub generation_time_ms: Option<u64>,
}

impl EmbeddingResult {
    /// Create a simple embedding result
    pub fn new(embedding: Vec<f32>, model: String) -> Self {
        Self {
            embedding,
            token_count: None,
            model,
            generation_time_ms: None,
        }
    }

    /// Create a detailed embedding result
    pub fn detailed(
        embedding: Vec<f32>,
        model: String,
        token_count: usize,
        generation_time_ms: u64,
    ) -> Self {
        Self {
            embedding,
            token_count: Some(token_count),
            model,
            generation_time_ms: Some(generation_time_ms),
        }
    }
}

/// Trait for embedding providers that convert text to vectors
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// Vector representation of the text
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts in batch
    ///
    /// More efficient than calling embed_text multiple times.
    /// Default implementation calls embed_text for each text.
    ///
    /// # Arguments
    /// * `texts` - Batch of texts to embed
    ///
    /// # Returns
    /// Vector of embeddings in the same order as input texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.embed_text(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    /// Calculate semantic similarity between two texts
    ///
    /// # Arguments
    /// * `text1` - First text
    /// * `text2` - Second text
    ///
    /// # Returns
    /// Similarity score between 0.0 and 1.0 (higher = more similar)
    async fn similarity(&self, text1: &str, text2: &str) -> Result<f32> {
        let embedding1 = self.embed_text(text1).await?;
        let embedding2 = self.embed_text(text2).await?;
        Ok(crate::embeddings::similarity::cosine_similarity(
            &embedding1,
            &embedding2,
        ))
    }

    /// Get the embedding dimension for this provider
    fn embedding_dimension(&self) -> usize;

    /// Get the model name/identifier
    fn model_name(&self) -> &str;

    /// Check if the provider is available/configured
    async fn is_available(&self) -> bool {
        // Default implementation tries to embed a simple test
        self.embed_text("test").await.is_ok()
    }

    /// Warm up the provider (load models, test connections, etc.)
    async fn warmup(&self) -> Result<()> {
        // Default implementation does a simple test embedding
        self.embed_text("warmup test").await?;
        Ok(())
    }

    /// Get provider-specific metadata
    fn metadata(&self) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name(),
            "dimension": self.embedding_dimension()
        })
    }
}

/// Utility functions for embedding providers
pub mod utils {
    use super::*;

    /// Normalize a vector to unit length
    pub fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
        let magnitude = (vector.iter().map(|x| x * x).sum::<f32>()).sqrt();
        if magnitude > 0.0 {
            for x in &mut vector {
                *x /= magnitude;
            }
        }
        vector
    }

    /// Validate embedding dimension matches expected
    pub fn validate_dimension(embedding: &[f32], expected: usize) -> Result<()> {
        if embedding.len() != expected {
            anyhow::bail!(
                "Embedding dimension mismatch: got {}, expected {}",
                embedding.len(),
                expected
            );
        }
        Ok(())
    }

    /// Chunk text into smaller pieces for embedding
    /// Useful for long texts that exceed model token limits
    pub fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
        if text.len() <= max_chars {
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_chunk = String::new();

        for word in words {
            if current_chunk.len() + word.len() + 1 > max_chars && !current_chunk.is_empty() {
                chunks.push(current_chunk.trim().to_string());
                current_chunk = word.to_string();
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(word);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Average multiple embeddings into a single embedding
    /// Useful for combining embeddings from chunked text
    pub fn average_embeddings(embeddings: &[Vec<f32>]) -> Result<Vec<f32>> {
        if embeddings.is_empty() {
            anyhow::bail!("Cannot average empty embeddings list");
        }

        let dimension = embeddings[0].len();
        let mut result = vec![0.0; dimension];

        for embedding in embeddings {
            if embedding.len() != dimension {
                anyhow::bail!("Inconsistent embedding dimensions");
            }
            for (i, &value) in embedding.iter().enumerate() {
                result[i] += value;
            }
        }

        let count = embeddings.len() as f32;
        for value in &mut result {
            *value /= count;
        }

        Ok(normalize_vector(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_vector() {
        let vector = vec![3.0, 4.0]; // Magnitude = 5.0
        let normalized = utils::normalize_vector(vector);

        // Should be [0.6, 0.8]
        assert!((normalized[0] - 0.6).abs() < 0.001);
        assert!((normalized[1] - 0.8).abs() < 0.001);

        // Check unit magnitude
        let magnitude = (normalized.iter().map(|x| x * x).sum::<f32>()).sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_chunk_text() {
        let text =
            "This is a long text that needs to be chunked into smaller pieces for processing";
        let chunks = utils::chunk_text(text, 25);

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 25);
        }

        // All chunks should contain parts of the original text
        let rejoined = chunks.join(" ");
        let original_words: Vec<&str> = text.split_whitespace().collect();
        let rejoined_words: Vec<&str> = rejoined.split_whitespace().collect();
        assert_eq!(original_words, rejoined_words);
    }

    #[test]
    fn test_average_embeddings() {
        let embeddings = vec![
            vec![1.0, 2.0, 3.0],
            vec![2.0, 4.0, 6.0],
            vec![3.0, 6.0, 9.0],
        ];

        let averaged = utils::average_embeddings(&embeddings).unwrap();

        // Average before normalization would be [2.0, 4.0, 6.0]
        // After normalization, it should be a unit vector in that direction
        let expected_magnitude = (4.0 + 16.0 + 36.0_f32).sqrt(); // sqrt(56) ≈ 7.48
        let expected = vec![
            2.0 / expected_magnitude,
            4.0 / expected_magnitude,
            6.0 / expected_magnitude,
        ];

        for (actual, expected) in averaged.iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 0.001);
        }
    }

    #[test]
    fn test_validate_dimension() {
        let embedding = vec![1.0, 2.0, 3.0];

        assert!(utils::validate_dimension(&embedding, 3).is_ok());
        assert!(utils::validate_dimension(&embedding, 4).is_err());
    }
}
