//! Gist extraction from episode descriptions.

use crate::episode::Episode;

use super::types::EpisodeGist;

/// Extracts gist summaries from episode descriptions.
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

        let scored = self.score_sentences(&sentences);
        let key_points = self.select_top_k(scored, self.max_key_points);

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
        text.split(['.', '!', '?'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() >= 5)
            .map(|s| {
                if !s.ends_with('.') && !s.ends_with('!') && !s.ends_with('?') {
                    format!("{s}.")
                } else {
                    s.to_string()
                }
            })
            .collect()
    }

    /// Score sentences by information density.
    pub fn score_sentences(&self, sentences: &[String]) -> Vec<(String, f32)> {
        let num_sentences = sentences.len();
        sentences
            .iter()
            .enumerate()
            .map(|(idx, s)| {
                let mut score = self.sentence_score(s);

                // Positional bias: favor intro and conclusion
                if idx == 0 || idx == num_sentences - 1 {
                    score *= 1.2;
                }

                (s.clone(), score)
            })
            .collect()
    }

    /// Compute information density score for a sentence.
    fn sentence_score(&self, sentence: &str) -> f32 {
        let len = sentence.len();

        let length_score = if len < 10 {
            0.3
        } else if len < 20 {
            0.5
        } else if len <= 50 {
            1.0
        } else if len <= 100 {
            0.7
        } else {
            0.4
        };

        let keyword_score = self.keyword_score(sentence);

        0.4 * length_score + 0.6 * keyword_score
    }

    /// Score based on keyword indicators.
    fn keyword_score(&self, sentence: &str) -> f32 {
        let lower = sentence.to_lowercase();

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

        // Cognitive markers (CogitoRAG inspired)
        let cognitive = ["learned", "insight", "decided", "realized", "discovered"];
        let outcome = ["success", "failed", "error", "bug", "issue", "feature"];

        let has_high_value = high_value.iter().any(|kw| lower.contains(kw));
        let has_cognitive = cognitive.iter().any(|kw| lower.contains(kw));
        let has_outcome = outcome.iter().any(|kw| lower.contains(kw));

        let mut score: f32 = 0.4;
        if has_high_value {
            score += 0.4;
        }
        if has_cognitive {
            score += 0.3;
        }
        if has_outcome {
            score += 0.2;
        }

        score.min(1.0)
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

        let compression = gist_len as f32 / original_len as f32;
        let coverage = num_points as f32 / self.max_key_points.max(1) as f32;

        let compression_score = if compression < 0.3 {
            1.0
        } else if compression < 0.5 {
            0.8
        } else if compression < 0.7 {
            0.5
        } else {
            0.3
        };

        0.5 * coverage + 0.5 * compression_score
    }
}
