//! Reconstructive retrieval windows (WG-120).
//!
//! Inspired by E-mem (arXiv:2601.21714): expand top-k hits into bounded
//! local windows to preserve useful context with fewer irrelevant tokens.
//!
//! Instead of returning isolated episodes/steps, return a window around
//! each hit, maintaining temporal context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for reconstructive window expansion.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Number of steps to expand before the hit (backward context).
    pub backward_steps: usize,
    /// Number of steps to expand after the hit (forward context).
    pub forward_steps: usize,
    /// Maximum window size (bounded to prevent excessive context).
    pub max_window_size: usize,
    /// Whether to include episode-level context in the window.
    pub include_episode_context: bool,
    /// Minimum score threshold for window expansion (skip low-relevance hits).
    pub min_score_threshold: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            backward_steps: 3,
            forward_steps: 2,
            max_window_size: 20,
            include_episode_context: true,
            min_score_threshold: 0.3,
        }
    }
}

impl WindowConfig {
    /// Create a compact window config (smaller context).
    #[must_use]
    pub fn compact() -> Self {
        Self {
            backward_steps: 1,
            forward_steps: 1,
            max_window_size: 10,
            include_episode_context: false,
            min_score_threshold: 0.5,
        }
    }

    /// Create a comprehensive window config (larger context).
    #[must_use]
    pub fn comprehensive() -> Self {
        Self {
            backward_steps: 5,
            forward_steps: 5,
            max_window_size: 50,
            include_episode_context: true,
            min_score_threshold: 0.2,
        }
    }
}

/// A single hit from retrieval that will be expanded into a window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    /// Episode ID of the hit.
    pub episode_id: Uuid,
    /// Step number within the episode (if applicable).
    pub step_number: Option<usize>,
    /// Relevance score of this hit.
    pub score: f32,
    /// Source tier that produced this hit (bm25, hdc, etc).
    pub source_tier: String,
}

impl RetrievalHit {
    /// Create a new retrieval hit.
    #[must_use]
    pub fn new(episode_id: Uuid, score: f32, source_tier: String) -> Self {
        Self {
            episode_id,
            step_number: None,
            score,
            source_tier,
        }
    }

    /// Create a retrieval hit with a specific step number.
    #[must_use]
    pub fn with_step(
        episode_id: Uuid,
        step_number: usize,
        score: f32,
        source_tier: String,
    ) -> Self {
        Self {
            episode_id,
            step_number: Some(step_number),
            score,
            source_tier,
        }
    }

    /// Check if this hit meets the minimum score threshold.
    #[must_use]
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.score >= threshold
    }
}

/// An expanded context window around a retrieval hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    /// The original hit that triggered this window.
    pub hit: RetrievalHit,
    /// Episode ID for this window.
    pub episode_id: Uuid,
    /// Start step number of the window.
    pub start_step: usize,
    /// End step number of the window.
    pub end_step: usize,
    /// Episode-level context (task description, outcome).
    pub episode_context: Option<String>,
    /// Window score (adjusted from hit score based on window size).
    pub window_score: f32,
    /// Number of steps in this window.
    pub step_count: usize,
}

impl ContextWindow {
    /// Get the window size (number of steps).
    #[must_use]
    pub fn size(&self) -> usize {
        self.step_count
    }

    /// Check if window is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.step_count == 0
    }

    /// Get the step range as a tuple.
    #[must_use]
    pub fn range(&self) -> (usize, usize) {
        (self.start_step, self.end_step)
    }
}

/// Result of window expansion containing multiple context windows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowExpansionResult {
    /// Expanded context windows.
    pub windows: Vec<ContextWindow>,
    /// Total steps across all windows.
    pub total_steps: usize,
    /// Original hit count before expansion.
    pub original_hit_count: usize,
    /// Whether any windows were truncated due to max size.
    pub truncated: bool,
}

impl WindowExpansionResult {
    /// Create an empty result.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            windows: Vec::new(),
            total_steps: 0,
            original_hit_count: 0,
            truncated: false,
        }
    }

    /// Get all episode IDs in the result.
    #[must_use]
    pub fn episode_ids(&self) -> Vec<Uuid> {
        self.windows.iter().map(|w| w.episode_id).collect()
    }

    /// Check if result is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty()
    }

    /// Get the number of windows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.windows.len()
    }
}

/// Expands retrieval hits into bounded context windows.
///
/// Given a set of top-k hits from retrieval, expands each hit into
/// a window of surrounding steps to preserve temporal context.
pub struct WindowExpander {
    config: WindowConfig,
}

impl WindowExpander {
    /// Create a new window expander with given configuration.
    pub fn new(config: WindowConfig) -> Self {
        Self { config }
    }

    /// Create a window expander with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(WindowConfig::default())
    }

    /// Expand retrieval hits into context windows.
    ///
    /// For each hit, compute a window of steps around it based on
    /// the configuration. Returns bounded windows that preserve context.
    #[must_use]
    pub fn expand(
        &self,
        hits: &[RetrievalHit],
        episode_step_counts: &[(Uuid, usize)],
    ) -> WindowExpansionResult {
        if hits.is_empty() {
            return WindowExpansionResult::empty();
        }

        let mut windows: Vec<ContextWindow> = Vec::new();
        let mut truncated = false;
        let original_hit_count = hits.len();

        for hit in hits {
            // Skip hits below threshold
            if !hit.meets_threshold(self.config.min_score_threshold) {
                continue;
            }

            // Find the step count for this episode
            let step_count = episode_step_counts
                .iter()
                .find(|(id, _)| *id == hit.episode_id)
                .map(|(_, count)| *count)
                .unwrap_or(0);

            // Determine window bounds
            let center_step = hit.step_number.unwrap_or(1);
            let start_step = center_step.saturating_sub(self.config.backward_steps);
            let end_step = std::cmp::min(center_step + self.config.forward_steps, step_count);

            // Apply max_window_size bound
            let window_size = end_step - start_step + 1;
            if window_size > self.config.max_window_size {
                truncated = true;
                // Trim from both ends to stay within bound
                let excess = window_size - self.config.max_window_size;
                let trim_back = excess / 2;
                let trim_forward = excess - trim_back;
                let new_start = start_step + trim_back;
                let new_end = end_step - trim_forward;
                let window = ContextWindow {
                    hit: hit.clone(),
                    episode_id: hit.episode_id,
                    start_step: new_start,
                    end_step: new_end,
                    episode_context: None, // Would be populated by storage lookup
                    window_score: hit.score
                        * (self.config.max_window_size as f32 / window_size as f32),
                    step_count: new_end - new_start + 1,
                };
                windows.push(window);
            } else {
                let window = ContextWindow {
                    hit: hit.clone(),
                    episode_id: hit.episode_id,
                    start_step,
                    end_step,
                    episode_context: None, // Would be populated by storage lookup
                    window_score: hit.score,
                    step_count: window_size,
                };
                windows.push(window);
            }
        }

        let total_steps = windows.iter().map(|w| w.step_count).sum();

        WindowExpansionResult {
            windows,
            total_steps,
            original_hit_count,
            truncated,
        }
    }

    /// Expand hits with episode context included.
    ///
    /// Requires episode descriptions to be provided for context.
    #[must_use]
    pub fn expand_with_context(
        &self,
        hits: &[RetrievalHit],
        episode_data: &[(Uuid, usize, String)], // (id, step_count, description)
    ) -> WindowExpansionResult {
        if hits.is_empty() {
            return WindowExpansionResult::empty();
        }

        let mut windows: Vec<ContextWindow> = Vec::new();
        let mut truncated = false;
        let original_hit_count = hits.len();

        for hit in hits {
            if !hit.meets_threshold(self.config.min_score_threshold) {
                continue;
            }

            // Find episode data
            let episode_info = episode_data.iter().find(|(id, _, _)| *id == hit.episode_id);

            if let Some((_, step_count, description)) = episode_info {
                let center_step = hit.step_number.unwrap_or(1);
                let start_step = center_step.saturating_sub(self.config.backward_steps);
                let end_step = std::cmp::min(center_step + self.config.forward_steps, *step_count);

                let window_size = end_step - start_step + 1;
                let (final_start, final_end, adjusted_score) =
                    if window_size > self.config.max_window_size {
                        truncated = true;
                        let excess = window_size - self.config.max_window_size;
                        let trim_back = excess / 2;
                        let trim_forward = excess - trim_back;
                        (
                            start_step + trim_back,
                            end_step - trim_forward,
                            hit.score * (self.config.max_window_size as f32 / window_size as f32),
                        )
                    } else {
                        (start_step, end_step, hit.score)
                    };

                let episode_context = if self.config.include_episode_context {
                    Some(description.clone())
                } else {
                    None
                };

                windows.push(ContextWindow {
                    hit: hit.clone(),
                    episode_id: hit.episode_id,
                    start_step: final_start,
                    end_step: final_end,
                    episode_context,
                    window_score: adjusted_score,
                    step_count: final_end - final_start + 1,
                });
            }
        }

        let total_steps = windows.iter().map(|w| w.step_count).sum();

        WindowExpansionResult {
            windows,
            total_steps,
            original_hit_count,
            truncated,
        }
    }

    /// Get the configuration for this expander.
    #[must_use]
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// Estimate the token savings from window-based retrieval.
    ///
    /// Returns the estimated reduction in tokens compared to
    /// returning full episodes.
    #[must_use]
    pub fn estimate_token_savings(&self, full_episode_steps: usize, window_size: usize) -> f32 {
        if full_episode_steps == 0 || window_size == 0 {
            return 0.0;
        }
        let reduction_ratio = 1.0 - (window_size as f32 / full_episode_steps as f32);
        reduction_ratio * 100.0 // Return as percentage
    }
}

/// Merge overlapping windows to avoid duplicate content.
///
/// When multiple hits come from the same episode with overlapping
/// step ranges, merge them into a single larger window.
#[must_use]
pub fn merge_overlapping_windows(windows: &[ContextWindow]) -> Vec<ContextWindow> {
    if windows.is_empty() {
        return Vec::new();
    }

    // Group windows by episode ID
    let mut episode_windows: std::collections::HashMap<Uuid, Vec<&ContextWindow>> =
        std::collections::HashMap::new();

    for window in windows {
        episode_windows
            .entry(window.episode_id)
            .or_default()
            .push(window);
    }

    // Merge windows within each episode
    let mut merged: Vec<ContextWindow> = Vec::new();

    for (episode_id, ep_windows) in episode_windows {
        if ep_windows.len() == 1 {
            // No merging needed
            merged.push(ep_windows[0].clone());
            continue;
        }

        // Find the union of all step ranges
        let min_start = ep_windows.iter().map(|w| w.start_step).min().unwrap_or(1);
        let max_end = ep_windows.iter().map(|w| w.end_step).max().unwrap_or(1);
        let max_score = ep_windows
            .iter()
            .map(|w| w.window_score)
            .fold(0.0f32, |a, b| a.max(b));

        // Combine episode context from best-scoring window
        let best_window = ep_windows
            .iter()
            .max_by(|a, b| {
                a.window_score
                    .partial_cmp(&b.window_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        merged.push(ContextWindow {
            hit: best_window.hit.clone(),
            episode_id,
            start_step: min_start,
            end_step: max_end,
            episode_context: best_window.episode_context.clone(),
            window_score: max_score,
            step_count: max_end - min_start + 1,
        });
    }

    // Sort by score descending
    merged.sort_by(|a, b| {
        b.window_score
            .partial_cmp(&a.window_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    merged
}

#[cfg(test)]
mod tests;
