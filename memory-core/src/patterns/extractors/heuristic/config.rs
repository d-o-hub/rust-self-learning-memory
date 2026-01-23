//! Configuration for heuristic extraction

/// Configuration for heuristic extraction
#[derive(Debug, Clone)]
pub struct HeuristicExtractorConfig {
    /// Minimum confidence score to keep a heuristic (0.0 to 1.0)
    pub min_confidence: f32,
    /// Minimum number of occurrences needed to extract a heuristic
    pub min_sample_size: usize,
}

impl Default for HeuristicExtractorConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            min_sample_size: 2,
        }
    }
}
