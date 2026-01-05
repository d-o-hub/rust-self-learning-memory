//! Types for salient feature extraction.

use serde::{Deserialize, Serialize};

/// Salient features extracted from an episode.
///
/// Contains the most important and reusable information from an episode,
/// enabling more effective retrieval and pattern learning.
///
/// # Fields
///
/// * `critical_decisions` - Key decision points and branching logic
/// * `tool_combinations` - Effective sequences of tools used together
/// * `error_recovery_patterns` - How errors were detected and resolved
/// * `key_insights` - Important discoveries and learnings
///
/// # Examples
///
/// ```
/// use memory_core::pre_storage::SalientFeatures;
///
/// let features = SalientFeatures {
///     critical_decisions: vec![
///         "Chose async implementation for better performance".to_string(),
///     ],
///     tool_combinations: vec![
///         vec!["parser".to_string(), "validator".to_string(), "generator".to_string()],
///     ],
///     error_recovery_patterns: vec![
///         "Connection timeout -> retry with exponential backoff".to_string(),
///     ],
///     key_insights: vec![
///         "Builder pattern simplifies configuration".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SalientFeatures {
    /// Critical decision points identified in the episode
    pub critical_decisions: Vec<String>,
    /// Effective tool sequences (2+ tools used together)
    pub tool_combinations: Vec<Vec<String>>,
    /// Error recovery patterns (error -> recovery steps)
    pub error_recovery_patterns: Vec<String>,
    /// Key insights from reflections and outcomes
    pub key_insights: Vec<String>,
}

impl SalientFeatures {
    /// Create a new empty `SalientFeatures`.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let features = SalientFeatures::new();
    /// assert!(features.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            critical_decisions: Vec::new(),
            tool_combinations: Vec::new(),
            error_recovery_patterns: Vec::new(),
            key_insights: Vec::new(),
        }
    }

    /// Check if the features are empty (no salient information extracted).
    ///
    /// # Returns
    ///
    /// `true` if all feature vectors are empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let empty = SalientFeatures::new();
    /// assert!(empty.is_empty());
    ///
    /// let mut features = SalientFeatures::new();
    /// features.key_insights.push("Important insight".to_string());
    /// assert!(!features.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.critical_decisions.is_empty()
            && self.tool_combinations.is_empty()
            && self.error_recovery_patterns.is_empty()
            && self.key_insights.is_empty()
    }

    /// Count total number of extracted features.
    ///
    /// # Returns
    ///
    /// Total count across all feature categories.
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::pre_storage::SalientFeatures;
    ///
    /// let mut features = SalientFeatures::new();
    /// features.critical_decisions.push("Decision 1".to_string());
    /// features.key_insights.push("Insight 1".to_string());
    /// features.key_insights.push("Insight 2".to_string());
    ///
    /// assert_eq!(features.count(), 3);
    /// ```
    #[must_use]
    pub fn count(&self) -> usize {
        self.critical_decisions.len()
            + self.tool_combinations.len()
            + self.error_recovery_patterns.len()
            + self.key_insights.len()
    }
}

impl Default for SalientFeatures {
    fn default() -> Self {
        Self::new()
    }
}
