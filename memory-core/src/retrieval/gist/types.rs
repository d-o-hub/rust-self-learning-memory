//! Data types for gist extraction and reranking.

use serde::{Deserialize, Serialize};

/// Extracted gist from an episode.
///
/// Contains key points extracted from the episode description and
/// a computed density score for reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeGist {
    /// Episode ID
    pub episode_id: String,
    /// Extracted key points (1-3 sentences)
    pub key_points: Vec<String>,
    /// Gist density score (0.0-1.0)
    /// Higher = more information per token
    pub density: f32,
    /// Original episode description length
    pub original_length: usize,
    /// Gist summary length
    pub gist_length: usize,
}

impl EpisodeGist {
    /// Create a new episode gist.
    #[must_use]
    pub fn new(episode_id: String, key_points: Vec<String>, density: f32) -> Self {
        let gist_length = key_points.iter().map(|s| s.len()).sum();
        Self {
            episode_id,
            key_points,
            density,
            original_length: 0,
            gist_length,
        }
    }

    /// Get the compression ratio (gist/original).
    #[must_use]
    pub fn compression_ratio(&self) -> f32 {
        if self.original_length == 0 {
            return 1.0;
        }
        self.gist_length as f32 / self.original_length as f32
    }

    /// Get the gist summary as a single string.
    #[must_use]
    pub fn summary(&self) -> String {
        self.key_points.join(" | ")
    }
}
