//! Efficiency multiplier calculation based on duration and step count

use crate::episode::Episode;
use super::constants::*;

/// Calculator for efficiency metrics
pub struct EfficiencyCalculator {
    /// Weight for duration in efficiency calculation
    pub duration_weight: f32,
    /// Weight for step count in efficiency calculation
    pub step_count_weight: f32,
}

impl EfficiencyCalculator {
    /// Create a new efficiency calculator with given weights
    pub fn new(duration_weight: f32, step_count_weight: f32) -> Self {
        Self {
            duration_weight,
            step_count_weight,
        }
    }

    /// Calculate efficiency multiplier based on duration and step count
    pub fn calculate(&self, episode: &Episode) -> f32 {
        let duration_score = self.calculate_duration_efficiency(episode);
        let step_count_score = self.calculate_step_count_efficiency(episode);

        let combined =
            (duration_score * self.duration_weight) + (step_count_score * self.step_count_weight);

        // Clamp to reasonable bounds
        combined.clamp(MIN_EFFICIENCY_MULTIPLIER, MAX_EFFICIENCY_MULTIPLIER)
    }

    /// Calculate duration efficiency score
    fn calculate_duration_efficiency(&self, episode: &Episode) -> f32 {
        if let Some(duration) = episode.duration() {
            let duration_secs = duration.num_seconds() as f32;

            if duration_secs <= 0.0 {
                return MAX_EFFICIENCY_MULTIPLIER;
            }

            // Efficiency decreases as duration increases
            // Exponential decay: e^(-x/threshold)
            let ratio = duration_secs / EFFICIENT_DURATION_SECS;
            let score = (-ratio / 2.0).exp();

            // Map to multiplier range
            MIN_EFFICIENCY_MULTIPLIER
                + (score * (MAX_EFFICIENCY_MULTIPLIER - MIN_EFFICIENCY_MULTIPLIER))
        } else {
            1.0 // Default if no duration
        }
    }

    /// Calculate step count efficiency score
    fn calculate_step_count_efficiency(&self, episode: &Episode) -> f32 {
        let step_count = episode.steps.len();

        if step_count == 0 {
            return MIN_EFFICIENCY_MULTIPLIER;
        }

        // Efficiency decreases as step count increases
        let ratio = step_count as f32 / EFFICIENT_STEP_COUNT as f32;
        let score = (-ratio / 2.0).exp();

        // Map to multiplier range
        MIN_EFFICIENCY_MULTIPLIER
            + (score * (MAX_EFFICIENCY_MULTIPLIER - MIN_EFFICIENCY_MULTIPLIER))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::ExecutionStep;
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome, TaskType};

    fn create_test_episode() -> Episode {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Simple,
            domain: "testing".to_string(),
            tags: vec![],
        };
        Episode::new("Test task".to_string(), context, TaskType::Testing)
    }

    #[test]
    fn test_efficiency_fast_execution() {
        let calculator = EfficiencyCalculator::new(0.5, 0.5);
        let mut episode = create_test_episode();

        // Add just a few steps
        for i in 0..3 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Quick".to_string(),
            artifacts: vec![],
        });

        let efficiency = calculator.calculate(&episode);
        assert!(efficiency > 1.0);
    }

    #[test]
    fn test_efficiency_slow_execution() {
        let calculator = EfficiencyCalculator::new(0.5, 0.5);
        let mut episode = create_test_episode();

        // Simulate old start time
        episode.start_time = chrono::Utc::now() - chrono::Duration::minutes(5);

        // Add many steps
        for i in 0..50 {
            let mut step = ExecutionStep::new(i + 1, format!("tool_{}", i), "Action".to_string());
            step.result = Some(ExecutionResult::Success {
                output: "OK".to_string(),
            });
            episode.add_step(step);
        }

        episode.complete(TaskOutcome::Success {
            verdict: "Slow".to_string(),
            artifacts: vec![],
        });

        let efficiency = calculator.calculate(&episode);
        assert!(efficiency < 1.0);
    }
}
