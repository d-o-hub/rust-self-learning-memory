//! Adaptive reward calculator implementation

use crate::episode::Episode;
use crate::types::{ComplexityLevel, RewardScore, TaskOutcome};
use tracing::{debug, instrument};

use crate::reward::domain_stats::DomainStatistics;

/// Adaptive reward calculator that adjusts thresholds based on domain statistics
///
/// Instead of using fixed thresholds (e.g., 60s = efficient), this calculator
/// uses domain-specific baselines derived from historical episodes.
#[derive(Clone)]
pub struct AdaptiveRewardCalculator {
    /// Weight for duration in efficiency calculation
    pub duration_weight: f32,
    /// Weight for step count in efficiency calculation
    pub step_count_weight: f32,
    /// Fallback to fixed thresholds if domain has insufficient data
    pub fallback_duration_secs: f32,
    /// Fallback step count threshold
    pub fallback_step_count: usize,
}

impl Default for AdaptiveRewardCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveRewardCalculator {
    /// Create a new adaptive calculator with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            duration_weight: 0.5,
            step_count_weight: 0.5,
            fallback_duration_secs: 60.0,
            fallback_step_count: 10,
        }
    }

    /// Create with custom weights and fallbacks
    #[must_use]
    pub fn with_config(
        duration_weight: f32,
        step_count_weight: f32,
        fallback_duration_secs: f32,
        fallback_step_count: usize,
    ) -> Self {
        Self {
            duration_weight,
            step_count_weight,
            fallback_duration_secs,
            fallback_step_count,
        }
    }

    /// Calculate reward using domain-specific statistics
    ///
    /// If domain_stats is None or unreliable, falls back to fixed thresholds
    #[instrument(skip(self, episode, domain_stats), fields(episode_id = %episode.episode_id))]
    pub fn calculate(
        &self,
        episode: &Episode,
        domain_stats: Option<&DomainStatistics>,
    ) -> RewardScore {
        let base = self.calculate_base_reward(episode);
        let efficiency = if let Some(stats) = domain_stats {
            if stats.is_reliable() {
                self.calculate_adaptive_efficiency(episode, stats)
            } else {
                self.calculate_fixed_efficiency(episode)
            }
        } else {
            self.calculate_fixed_efficiency(episode)
        };
        let complexity_bonus = self.calculate_complexity_bonus(episode);
        let quality_multiplier = self.calculate_quality_multiplier(episode);
        let learning_bonus = self.calculate_learning_bonus(episode);
        let raw_reward =
            (base * efficiency * complexity_bonus * quality_multiplier) + learning_bonus;
        let normalized_reward = if let Some(stats) = domain_stats {
            self.calculate_normalized_reward(raw_reward, stats, episode)
        } else {
            raw_reward
        };
        let half_life = domain_stats.map(|s| s.decay_half_life_days).unwrap_or(30.0);
        let decayed_reward =
            self.calculate_decayed_reward(normalized_reward, episode.start_time, half_life);
        let effective_reward = decayed_reward;
        let total = raw_reward;
        debug!(
            base = base,
            total = total,
            "Calculated adaptive reward score"
        );
        RewardScore {
            total,
            base,
            efficiency,
            complexity_bonus,
            quality_multiplier,
            learning_bonus,
            abstention_score: 0.0,
            raw_reward,
            normalized_reward,
            decayed_reward,
            effective_reward,
        }
    }

    /// Calculate base reward from outcome (same as fixed calculator)
    fn calculate_base_reward(&self, episode: &Episode) -> f32 {
        match &episode.outcome {
            Some(TaskOutcome::Success { .. }) => 1.0,
            Some(TaskOutcome::PartialSuccess {
                completed, failed, ..
            }) => {
                let total = completed.len() + failed.len();
                if total == 0 {
                    0.5
                } else {
                    completed.len() as f32 / total as f32
                }
            }
            Some(TaskOutcome::Failure { .. }) => 0.0,
            Some(TaskOutcome::Abstained { .. }) => 0.3,
            None => 0.0,
        }
    }

    /// Calculate efficiency using adaptive thresholds from domain statistics
    fn calculate_adaptive_efficiency(&self, episode: &Episode, stats: &DomainStatistics) -> f32 {
        // Use p50 (median) as the "efficient" baseline
        // Episodes faster/shorter than median get efficiency boost
        // Episodes slower/longer than median get efficiency penalty

        let duration_score = if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;

            if duration_secs <= 0.0 {
                return 1.5; // Maximum efficiency for instant completion
            }

            // Compare to domain baseline (p50)
            let baseline = stats.p50_duration_secs.max(1.0); // Avoid division by zero
            let ratio = duration_secs / baseline;

            // Efficiency score: better than median = >1.0, worse = <1.0
            // Use exponential decay similar to fixed calculator
            let score = (-ratio / 2.0).exp();

            // Map to reasonable range (0.5 to 1.5)
            0.5 + (score * 1.0)
        } else {
            1.0
        };

        let step_count_score = {
            let step_count = episode.steps.len();

            if step_count == 0 {
                return 0.5;
            }

            // Compare to domain baseline
            let baseline = stats.p50_step_count.max(1); // Avoid division by zero
            let ratio = step_count as f32 / baseline as f32;

            let score = (-ratio / 2.0).exp();
            0.5 + (score * 1.0)
        };

        // Combine with weights
        let combined =
            (duration_score * self.duration_weight) + (step_count_score * self.step_count_weight);

        // Clamp to reasonable bounds
        combined.clamp(0.5, 1.5)
    }

    /// Calculate efficiency using fixed thresholds (fallback)
    fn calculate_fixed_efficiency(&self, episode: &Episode) -> f32 {
        let duration_score = if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;

            if duration_secs <= 0.0 {
                return 1.5;
            }

            let ratio = duration_secs / self.fallback_duration_secs;
            let score = (-ratio / 2.0).exp();
            0.5 + (score * 1.0)
        } else {
            1.0
        };

        let step_count_score = {
            let step_count = episode.steps.len();

            if step_count == 0 {
                return 0.5;
            }

            let ratio = step_count as f32 / self.fallback_step_count as f32;
            let score = (-ratio / 2.0).exp();
            0.5 + (score * 1.0)
        };

        let combined =
            (duration_score * self.duration_weight) + (step_count_score * self.step_count_weight);

        combined.clamp(0.5, 1.5)
    }

    /// Calculate complexity bonus
    fn calculate_complexity_bonus(&self, episode: &Episode) -> f32 {
        match episode.context.complexity {
            ComplexityLevel::Simple => 1.0,
            ComplexityLevel::Moderate => 1.1,
            ComplexityLevel::Complex => 1.2,
        }
    }

    /// Calculate quality multiplier (same as fixed calculator)
    fn calculate_quality_multiplier(&self, episode: &Episode) -> f32 {
        let mut quality: f32 = 1.0;

        // Analyze artifacts for quality indicators
        if let Some(TaskOutcome::Success { artifacts, .. }) = &episode.outcome {
            let has_test_coverage = artifacts
                .iter()
                .any(|a| a.contains("coverage") || a.contains("test"));
            if has_test_coverage {
                quality += 0.1;
            }

            if artifacts.len() >= 3 {
                quality += 0.05;
            }

            if let Some(coverage_str) = episode.metadata.get("test_coverage") {
                if let Ok(coverage) = coverage_str.parse::<f32>() {
                    #[allow(clippy::excessive_nesting)]
                    if coverage > 80.0 {
                        quality += 0.15;
                    } else if coverage > 60.0 {
                        quality += 0.1;
                    }
                }
            }
        }

        // Quality based on error handling
        let total_steps = episode.steps.len();
        if total_steps > 0 {
            let error_rate = episode.failed_steps_count() as f32 / total_steps as f32;

            if error_rate > 0.3 {
                quality -= 0.2;
            } else if error_rate > 0.1 {
                quality -= 0.1;
            } else if error_rate == 0.0 {
                quality += 0.1;
            }
        }

        if episode.metadata.contains_key("clippy_warnings") {
            if let Some(warnings) = episode.metadata.get("clippy_warnings") {
                if warnings == "0" {
                    quality += 0.05;
                }
            }
        }

        quality.clamp(0.5, 1.5)
    }

    /// Calculate learning bonus (same as fixed calculator)
    fn calculate_learning_bonus(&self, episode: &Episode) -> f32 {
        let mut bonus = 0.0;

        // Bonus for discovering new patterns
        let pattern_count = episode.patterns.len();
        if pattern_count > 0 {
            bonus += (pattern_count as f32 * 0.1).min(0.3);
        }

        // Bonus for novel tool sequences
        if let Some(novelty) = self.calculate_novelty_bonus(episode) {
            bonus += novelty;
        }

        // Bonus for efficient problem solving
        let total_steps = episode.steps.len();
        if total_steps > 0 {
            let success_rate = episode.successful_steps_count() as f32 / total_steps as f32;

            if success_rate > 0.9 && total_steps >= 5 {
                bonus += 0.2;
            } else if success_rate == 1.0 && total_steps >= 3 {
                bonus += 0.15;
            }
        }

        // Bonus for error recovery
        if self.detect_error_recovery(episode) {
            bonus += 0.15;
        }

        // Bonus for optimization
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;
            if duration_secs < 30.0 && total_steps > 0 && total_steps < 10 {
                bonus += 0.1;
            }
        }

        bonus.min(0.5)
    }

    /// Calculate novelty bonus
    fn calculate_novelty_bonus(&self, episode: &Episode) -> Option<f32> {
        if episode.steps.len() < 3 {
            return None;
        }

        let unique_tools: std::collections::HashSet<_> =
            episode.steps.iter().map(|s| &s.tool).collect();

        if unique_tools.len() >= 5 {
            Some(0.15)
        } else if unique_tools.len() >= 3 {
            Some(0.1)
        } else {
            None
        }
    }

    /// Detect error recovery
    fn detect_error_recovery(&self, episode: &Episode) -> bool {
        for i in 0..episode.steps.len().saturating_sub(1) {
            let current = &episode.steps[i];
            let next = &episode.steps[i + 1];

            if !current.is_success() && next.is_success() {
                return true;
            }
        }
        false
    }

    fn calculate_normalized_reward(
        &self,
        raw_reward: f32,
        stats: &DomainStatistics,
        episode: &Episode,
    ) -> f32 {
        if !stats.is_reliable() {
            return raw_reward;
        }

        // 1. Domain-level normalization (z-score)
        let domain_z = if stats.reward_std_dev > 0.01 {
            (raw_reward - stats.avg_reward) / stats.reward_std_dev
        } else {
            0.0
        };

        // 2. Category-specific normalization
        let mut category_z_scores = Vec::new();

        // Agent type
        if let Some(agent_type) = episode.metadata.get("agent_type") {
            if let Some((avg, std_dev, count)) = stats.agent_type_stats.get(agent_type) {
                if *count >= 3 && *std_dev > 0.01 {
                    category_z_scores.push((raw_reward - *avg) / *std_dev);
                }
            }
        }

        // Task type
        if let Some((avg, std_dev, count)) = stats.task_type_stats.get(&episode.task_type.to_string())
        {
            if *count >= 3 && *std_dev > 0.01 {
                category_z_scores.push((raw_reward - *avg) / *std_dev);
            }
        }

        // Complexity
        if let Some((avg, std_dev, count)) = stats
            .complexity_stats
            .get(&episode.context.complexity.to_string())
        {
            if *count >= 3 && *std_dev > 0.01 {
                category_z_scores.push((raw_reward - *avg) / *std_dev);
            }
        }

        // Average the z-scores
        let final_z = if category_z_scores.is_empty() {
            domain_z
        } else {
            let sum: f32 = category_z_scores.iter().sum();
            (domain_z + (sum / category_z_scores.len() as f32)) / 2.0
        };

        // Map z-score to normalized reward around 1.0
        // z=0 -> 1.0, z=1 -> 1.2, z=-1 -> 0.8
        (1.0 + (final_z * 0.2)).clamp(0.1, 2.0)
    }

    fn calculate_decayed_reward(
        &self,
        reward: f32,
        start_time: chrono::DateTime<chrono::Utc>,
        half_life_days: f32,
    ) -> f32 {
        let now = chrono::Utc::now();
        let age_days = (now - start_time).num_days() as f32;
        if age_days <= 0.0 {
            return reward;
        }
        reward * 0.5f32.powf(age_days / half_life_days)
    }
}
