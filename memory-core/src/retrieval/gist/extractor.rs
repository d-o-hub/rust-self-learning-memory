//! Gist extraction from episode descriptions.

use crate::episode::Episode;

use super::types::EpisodeGist;

/// Extracts gist summaries from episode descriptions.
///
/// Parses episode text to extract key points that capture the
/// essential information for downstream prompts.
///
/// # Algorithm
///
/// 1. Split description into sentences
/// 2. Score each sentence by information density
/// 3. Select top-k sentences as key points
/// 4. Compute density score based on compression ratio
///
/// # Examples
///
/// ```
/// use do_memory_core::retrieval::GistExtractor;
///
/// let extractor = GistExtractor::default();
///
/// // Extract key points from a description
/// let gist = extractor.extract("Fixed authentication bug. Added JWT validation. Improved error handling.");
/// assert!(gist.key_points.len() <= 3);
/// assert!(gist.density > 0.0);
///
/// // Empty description returns empty gist
/// let empty = extractor.extract("");
/// assert!(empty.key_points.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct GistExtractor {
    /// Maximum key points to extract
    max_key_points: usize,
}

impl Default for GistExtractor {
    fn default() -> Self {
        Self::new(3)
    }
}

impl GistExtractor {
    /// Create a new gist extractor.
    ///
    /// # Arguments
    ///
    /// * `max_key_points` - Maximum sentences to extract per episode
    #[must_use]
    pub fn new(max_key_points: usize) -> Self {
        Self {
            max_key_points: max_key_points.max(1),
        }
    }

    /// Get the maximum key points setting.
    #[must_use]
    pub fn max_key_points(&self) -> usize {
        self.max_key_points
    }

    /// Extract gist from an episode.
    ///
    /// # Arguments
    ///
    /// * `episode` - The episode to extract gist from
    ///
    /// # Returns
    ///
    /// An `EpisodeGist` with key points and density score
    #[must_use]
    pub fn extract_from_episode(&self, episode: &Episode) -> EpisodeGist {
        let gist = self.extract(&episode.task_description);
        EpisodeGist {
            episode_id: episode.episode_id.to_string(),
            key_points: gist.key_points,
            density: gist.density,
            original_length: gist.original_length,
            gist_length: gist.gist_length,
        }
    }

    /// Extract gist from a text description.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to extract gist from
    ///
    /// # Returns
    ///
    /// An `EpisodeGist` with key points and density score
    #[must_use]
    pub fn extract(&self, text: &str) -> EpisodeGist {
        let original_length = text.len();

        if text.is_empty() {
            return EpisodeGist {
                episode_id: String::new(),
                key_points: Vec::new(),
                density: 0.0,
                original_length: 0,
                gist_length: 0,
            };
        }

        // Split into sentences
        let sentences = self.split_sentences(text);

        if sentences.is_empty() {
            return EpisodeGist {
                episode_id: String::new(),
                key_points: Vec::new(),
                density: 0.0,
                original_length,
                gist_length: 0,
            };
        }

        // Score and select top sentences
        let scored = self.score_sentences(&sentences);
        let key_points = self.select_top_k(scored, self.max_key_points);

        // Compute density
        let gist_length = key_points.iter().map(|s| s.len()).sum();
        let density = self.compute_density(original_length, gist_length, key_points.len());

        EpisodeGist {
            episode_id: String::new(),
            key_points,
            density,
            original_length,
            gist_length,
        }
    }

    /// Split text into sentences.
    fn split_sentences(&self, text: &str) -> Vec<String> {
        // Simple sentence splitting on punctuation
        text.split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() >= 5)
            .map(|s| {
                // Ensure sentence ends with punctuation
                if !s.ends_with('.') && !s.ends_with('!') && !s.ends_with('?') {
                    format!("{s}.")
                } else {
                    s.to_string()
                }
            })
            .collect()
    }

    /// Score sentences by information density.
    fn score_sentences(&self, sentences: &[String]) -> Vec<(String, f32)> {
        sentences
            .iter()
            .map(|s| {
                let score = self.sentence_score(s);
                (s.clone(), score)
            })
            .collect()
    }

    /// Compute information density score for a sentence.
    fn sentence_score(&self, sentence: &str) -> f32 {
        // Score based on:
        // 1. Length (longer sentences may have more info, but penalize very long)
        // 2. Keyword indicators (action verbs, outcomes)
        // 3. Token density (words per character)

        let len = sentence.len();

        // Length score: optimal around 20-50 chars
        let length_score = if len < 10 {
            0.3 // Too short, likely incomplete
        } else if len < 20 {
            0.5 // Short but acceptable
        } else if len <= 50 {
            1.0 // Optimal length
        } else if len <= 100 {
            0.7 // Long but acceptable
        } else {
            0.4 // Very long, verbose
        };

        // Keyword score: action verbs indicate high-value info
        let keyword_score = self.keyword_score(sentence);

        // Combine scores
        0.4 * length_score + 0.6 * keyword_score
    }

    /// Score based on keyword indicators.
    fn keyword_score(&self, sentence: &str) -> f32 {
        let lower = sentence.to_lowercase();

        // High-value action keywords
        let high_value = [
            "fixed",
            "added",
            "implemented",
            "resolved",
            "completed",
            "solved",
            "created",
            "updated",
            "refactored",
            "optimized",
            "deployed",
            "tested",
            "validated",
            "confirmed",
        ];

        // Outcome indicators
        let outcome = ["success", "failed", "error", "bug", "issue", "feature"];

        let has_high_value = high_value.iter().any(|kw| lower.contains(kw));
        let has_outcome = outcome.iter().any(|kw| lower.contains(kw));

        if has_high_value && has_outcome {
            1.0
        } else if has_high_value {
            0.8
        } else if has_outcome {
            0.6
        } else {
            0.4
        }
    }

    /// Select top-k sentences by score.
    fn select_top_k(&self, scored: Vec<(String, f32)>, k: usize) -> Vec<String> {
        let mut sorted = scored;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        sorted.into_iter().take(k).map(|(s, _)| s).collect()
    }

    /// Compute gist density score.
    fn compute_density(&self, original_len: usize, gist_len: usize, num_points: usize) -> f32 {
        if original_len == 0 {
            return 0.0;
        }

        // Compression ratio (how much we condensed)
        let compression = gist_len as f32 / original_len as f32;

        // Information coverage (how many key points extracted)
        let coverage = num_points as f32 / self.max_key_points.max(1) as f32;

        // Density = high coverage + good compression
        // Penalize if gist is too long (bad compression)
        // Reward if gist captures key points (good coverage)
        let compression_score = if compression < 0.3 {
            1.0 // Excellent compression
        } else if compression < 0.5 {
            0.8 // Good compression
        } else if compression < 0.7 {
            0.5 // Moderate compression
        } else {
            0.3 // Poor compression
        };

        0.5 * coverage + 0.5 * compression_score
    }
}
