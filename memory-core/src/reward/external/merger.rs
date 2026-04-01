//! Signal merging and combination logic

use super::{ExternalSignalConfig, ExternalSignalSet};
use crate::types::RewardScore;

/// Merges internal reward with external signals
pub struct SignalMerger {
    /// Weight for internal calculation (default: 0.7)
    pub internal_weight: f32,
    /// Weight for external signals (default: 0.3)
    pub external_weight: f32,
    /// Minimum confidence threshold
    pub min_confidence: f32,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
}

/// Result of merging internal and external signals
#[derive(Debug, Clone)]
pub struct MergedReward {
    /// Merged base score
    pub base: f32,
    /// Merged efficiency score
    pub efficiency: f32,
    /// Original internal score
    pub internal_score: f32,
    /// External signal score
    pub external_score: f32,
    /// Confidence in the merge (0.0-1.0)
    pub confidence: f32,
}

impl MergedReward {
    /// Calculate total reward score
    pub fn calculate_total(&self) -> f32 {
        // Simple weighted combination
        self.base * self.efficiency
    }

    /// Get the influence of external signals (0.0-1.0)
    pub fn external_influence(&self) -> f32 {
        if self.internal_score == 0.0 {
            1.0 // Only external
        } else {
            let diff = (self.external_score - self.internal_score).abs();
            diff / self.internal_score
        }
    }
}

/// Strategy for resolving conflicts between internal and external signals
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolution {
    /// Prefer external signals (ground truth)
    PreferExternal,
    /// Prefer internal calculation
    PreferInternal,
    /// Average both signals
    Average,
    /// Weight by confidence
    WeightByConfidence,
}

impl SignalMerger {
    /// Create a merger with default weights (70% internal, 30% external)
    pub fn new() -> Self {
        Self {
            internal_weight: 0.7,
            external_weight: 0.3,
            min_confidence: 0.5,
            conflict_resolution: ConflictResolution::Average,
        }
    }

    /// Create a merger with custom weights
    ///
    /// # Panics
    ///
    /// Panics if weights don't sum to approximately 1.0
    pub fn with_weights(internal: f32, external: f32) -> Self {
        assert!(
            (internal + external - 1.0).abs() < 0.001,
            "Weights must sum to 1.0, got {} + {} = {}",
            internal,
            external,
            internal + external
        );

        Self {
            internal_weight: internal,
            external_weight: external,
            min_confidence: 0.5,
            conflict_resolution: ConflictResolution::Average,
        }
    }

    /// Set minimum confidence threshold
    #[must_use]
    pub fn with_min_confidence(mut self, threshold: f32) -> Self {
        self.min_confidence = threshold.clamp(0.0, 1.0);
        self
    }

    /// Set conflict resolution strategy
    #[must_use]
    pub fn with_conflict_resolution(mut self, strategy: ConflictResolution) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    /// Merge internal reward with external signals
    ///
    /// # Arguments
    ///
    /// * `internal` - The internal reward score
    /// * `external_sets` - External signal sets from providers
    pub fn merge(
        &self,
        internal: &RewardScore,
        external_sets: &[ExternalSignalSet],
    ) -> MergedReward {
        // Filter low-confidence signals
        let valid_signals: Vec<_> = external_sets
            .iter()
            .filter(|s| s.confidence >= self.min_confidence)
            .collect();

        // Calculate external quality score
        let external_quality = if valid_signals.is_empty() {
            internal.efficiency // No external data = use internal
        } else {
            // Average episode quality from all providers
            let qualities: Vec<_> = valid_signals
                .iter()
                .filter_map(|s| s.episode_quality)
                .collect();

            if qualities.is_empty() {
                internal.efficiency
            } else {
                qualities.iter().sum::<f32>() / qualities.len() as f32
            }
        };

        // Calculate external success rate from tool signals
        let (external_success, external_confidence) = if valid_signals.is_empty() {
            (internal.base, 0.0) // No external data
        } else {
            let total_samples: usize = valid_signals
                .iter()
                .flat_map(|s| &s.tool_signals)
                .map(|t| t.sample_count)
                .sum();

            if total_samples == 0 {
                (internal.base, 0.5)
            } else {
                let weighted_success: f32 = valid_signals
                    .iter()
                    .flat_map(|s| &s.tool_signals)
                    .map(|t| t.success_rate * t.sample_count as f32)
                    .sum();

                let avg_confidence: f32 = valid_signals.iter().map(|s| s.confidence).sum::<f32>()
                    / valid_signals.len() as f32;

                (weighted_success / total_samples as f32, avg_confidence)
            }
        };

        // Apply conflict resolution
        let merged_base = self.resolve_conflict(internal.base, external_success);
        let merged_efficiency = self.resolve_conflict(internal.efficiency, external_quality);

        MergedReward {
            base: merged_base,
            efficiency: merged_efficiency,
            internal_score: internal.total,
            external_score: external_success,
            confidence: if valid_signals.is_empty() {
                0.5
            } else {
                external_confidence
            },
        }
    }

    /// Resolve conflict between internal and external values
    fn resolve_conflict(&self, internal: f32, external: f32) -> f32 {
        match self.conflict_resolution {
            ConflictResolution::PreferExternal => external,
            ConflictResolution::PreferInternal => internal,
            ConflictResolution::Average => (internal + external) / 2.0,
            ConflictResolution::WeightByConfidence => {
                // Weight by confidence (external confidence assumed in external value)
                let internal_weight = self.internal_weight;
                let external_weight = self.external_weight;
                internal * internal_weight + external * external_weight
            }
        }
    }

    /// Create a merger from configuration
    pub fn from_config(config: &ExternalSignalConfig) -> Self {
        Self {
            internal_weight: 1.0 - config.default_weight,
            external_weight: config.default_weight,
            min_confidence: config.min_confidence,
            conflict_resolution: ConflictResolution::Average,
        }
    }
}

impl Default for SignalMerger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_reward() -> RewardScore {
        RewardScore {
            total: 1.0,
            base: 0.8,
            efficiency: 1.2,
            complexity_bonus: 1.1,
            quality_multiplier: 1.0,
            learning_bonus: 0.0,
        }
    }

    #[test]
    fn test_merger_with_no_external_signals() {
        let merger = SignalMerger::new();
        let internal = create_test_reward();
        let external: Vec<ExternalSignalSet> = vec![];

        let merged = merger.merge(&internal, &external);

        // Should use internal values
        assert_eq!(merged.base, internal.base);
        assert_eq!(merged.efficiency, internal.efficiency);
        assert_eq!(merged.confidence, 0.0);
    }

    #[test]
    fn test_merger_with_external_signals() {
        let merger = SignalMerger::with_weights(0.7, 0.3);
        let internal = create_test_reward();

        let external = ExternalSignalSet {
            provider: "test".to_string(),
            tool_signals: vec![super::super::ToolSignal {
                tool_name: "test_tool".to_string(),
                success_rate: 0.9,
                avg_latency_ms: 100.0,
                sample_count: 50,
                metadata: std::collections::HashMap::new(),
            }],
            episode_quality: Some(0.85),
            timestamp: chrono::Utc::now(),
            confidence: 0.8,
        };

        let merged = merger.merge(&internal, &[external]);

        // Should be a blend of internal and external
        assert!(merged.base > internal.base); // External success (0.9) > internal (0.8)
        assert!(merged.base < 0.9);
        assert_eq!(merged.confidence, 0.8);
    }

    #[test]
    fn test_conflict_resolution() {
        let merger =
            SignalMerger::new().with_conflict_resolution(ConflictResolution::PreferExternal);

        let internal = create_test_reward();
        let external = ExternalSignalSet {
            provider: "test".to_string(),
            tool_signals: vec![super::super::ToolSignal {
                tool_name: "tool".to_string(),
                success_rate: 0.5, // Different from internal
                avg_latency_ms: 100.0,
                sample_count: 100,
                metadata: std::collections::HashMap::new(),
            }],
            episode_quality: Some(0.5),
            timestamp: chrono::Utc::now(),
            confidence: 0.9,
        };

        let merged = merger.merge(&internal, &[external]);

        // Should prefer external (0.5) over internal (0.8)
        assert_eq!(merged.base, 0.5);
    }
}
