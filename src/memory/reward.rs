use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Task type identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    General,
    Specific(String),
}

/// Agent type identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AgentType {
    Human,
    AI,
    Hybrid,
}

/// Complexity band
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ComplexityBand {
    Low,
    Medium,
    High,
}

/// Statistics about reward distribution per domain
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainStatistics {
    /// Mean reward per task type
    pub mean_reward_per_task: HashMap<TaskType, f64>,
    /// Mean reward per agent type
    pub mean_reward_per_agent: HashMap<AgentType, f64>,
    /// Mean reward per complexity band
    pub mean_reward_per_complexity: HashMap<ComplexityBand, f64>,
    /// Baseline mean reward overall
    pub overall_mean: f64,
    /// Standard deviation overall
    pub overall_std: f64,
}

impl Default for DomainStatistics {
    fn default() -> Self {
        Self {
            mean_reward_per_task: HashMap::new(),
            mean_reward_per_agent: HashMap::new(),
            mean_reward_per_complexity: HashMap::new(),
            overall_mean: 0.0,
            overall_std: 1.0,
        }
    }
}

/// Reward score with separate raw and effective values
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DualRewardScore {
    /// Raw reward before any normalization or decay
    pub raw_reward: f64,
    /// Reward normalized by domain statistics
    pub normalized_reward: f64,
    /// Reward after temporal decay
    pub decayed_reward: f64,
    /// Final effective reward used for ranking
    pub effective_reward: f64,
}

impl DualRewardScore {
    pub fn new(raw: f64) -> Self {
        Self {
            raw_reward: raw,
            normalized_reward: raw,
            decayed_reward: raw,
            effective_reward: raw,
        }
    }
}

/// Configuration for reward calculation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RewardConfig {
    /// Half-life in seconds for temporal decay, keyed by task type
    pub half_life_per_task: HashMap<TaskType, f64>,
    /// Default half-life if not specified
    pub default_half_life_secs: f64,
}

impl Default for RewardConfig {
    fn default() -> Self {
        let mut half_life_per_task = HashMap::new();
        half_life_per_task.insert(TaskType::General, 86400.0); // 1 day
        Self {
            half_life_per_task,
            default_half_life_secs: 86400.0,
        }
    }
}

/// Adaptive reward calculator with temporal decay and normalization
#[derive(Debug, Clone)]
pub struct AdaptiveRewardCalculator {
    pub config: RewardConfig,
    pub stats: DomainStatistics,
}

impl AdaptiveRewardCalculator {
    pub fn new(config: RewardConfig, stats: DomainStatistics) -> Self {
        Self { config, stats }
    }

    /// Calculate the dual reward score for a given raw reward, age, and domain context
    pub fn calculate(
        &self,
        raw_reward: f64,
        age: Duration,
        task_type: &TaskType,
        agent_type: &AgentType,
        complexity: &ComplexityBand,
    ) -> DualRewardScore {
        let mut score = DualRewardScore::new(raw_reward);

        // 1. Normalize by domain statistics
        let normalized = self.normalize(raw_reward, task_type, agent_type, complexity);
        score.normalized_reward = normalized;

        // 2. Apply temporal decay
        let half_life = self.config.half_life_per_task.get(task_type)
            .copied()
            .unwrap_or(self.config.default_half_life_secs);
        let decay_factor = if half_life > 0.0 {
            let age_secs = age.as_secs_f64();
            (-age_secs * std::f64::consts::LN_2 / half_life).exp()
        } else {
            1.0
        };
        let decayed = normalized * decay_factor;
        score.decayed_reward = decayed;

        // 3. Effective reward (combine normalization and decay)
        // Here we use decayed as effective; could also combine with other factors
        score.effective_reward = decayed;

        score
    }

    /// Normalize raw reward using z-score based on domain statistics
    fn normalize(
        &self,
        raw: f64,
        task_type: &TaskType,
        agent_type: &AgentType,
        complexity: &ComplexityBand,
    ) -> f64 {
        let mean = self.combined_mean(task_type, agent_type, complexity);
        let std = self.stats.overall_std.max(1e-8);
        (raw - mean) / std
    }

    /// Compute a combined mean from the statistics (simple average of relevant means)
    fn combined_mean(
        &self,
        task_type: &TaskType,
        agent_type: &AgentType,
        complexity: &ComplexityBand,
    ) -> f64 {
        let mut count = 0;
        let mut sum = 0.0;

        if let Some(m) = self.stats.mean_reward_per_task.get(task_type) {
            sum += m;
            count += 1;
        }
        if let Some(m) = self.stats.mean_reward_per_agent.get(agent_type) {
            sum += m;
            count += 1;
        }
        if let Some(m) = self.stats.mean_reward_per_complexity.get(complexity) {
            sum += m;
            count += 1;
        }
        if count > 0 {
            sum / count as f64
        } else {
            self.stats.overall_mean
        }
    }

    /// Rank a list of episodes by effective reward (descending)
    pub fn rank_episodes<'a>(
        &self,
        episodes: &'a mut [Episode],
    ) -> Vec<&'a Episode> {
        episodes.sort_by(|a, b| {
            b.effective_reward
                .partial_cmp(&a.effective_reward)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        episodes.iter().collect()
    }

    /// Recompute rewards for a slice of episodes (for migration)
    pub fn recompute_rewards(&self, episodes: &mut [Episode]) {
        for ep in episodes.iter_mut() {
            // Use stored creation time as age: now - created_at
            let age = SystemTime::now()
                .duration_since(ep.created_at)
                .unwrap_or_default();
            let score = self.calculate(
                ep.raw_reward,
                age,
                &ep.task_type,
                &ep.agent_type,
                &ep.complexity,
            );
            ep.effective_reward = score.effective_reward;
            ep.raw_reward = score.raw_reward;
            // Store other fields if needed (ep.dual_score = score)
        }
    }
}

/// A memory episode with reward-related fields
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Episode {
    pub id: u64,
    pub created_at: SystemTime,
    pub raw_reward: f64,
    pub effective_reward: f64,
    pub task_type: TaskType,
    pub agent_type: AgentType,
    pub complexity: ComplexityBand,
    // Additional fields...
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_normalize_and_decay() {
        let mut stats = DomainStatistics::default();
        stats.overall_mean = 10.0;
        stats.overall_std = 2.0;
        stats.mean_reward_per_task.insert(TaskType::General, 12.0);

        let mut half_life_per_task = HashMap::new();
        half_life_per_task.insert(TaskType::General, 3600.0); // 1 hour
        let config = RewardConfig {
            half_life_per_task,
            default_half_life_secs: 86400.0,
        };

        let calculator = AdaptiveRewardCalculator::new(config, stats);

        let raw = 15.0;
        let age = Duration::from_secs(1800); // 30 minutes
        let task = TaskType::General;
        let agent = AgentType::Human;
        let complexity = ComplexityBand::Medium;

        let score = calculator.calculate(raw, age, &task, &agent, &complexity);

        // Normalized: (15 - (12+10+0)/1?) Actually combined mean = (12 + 0 + 0)/1? No, no agent/complexity means, so overall mean=10? Wait stats have overall_mean=10 but also mean_per_task=12. combined_mean uses only task mean (since count=1) => 12. So normalized = (15-12)/2 = 1.5
        assert!((score.normalized_reward - 1.5).abs() < 1e-6);

        // Decay factor: exp(-1800 * ln2 / 3600) = exp(-0.5*ln2) = 2^{-0.5} = 0.7071
        let expected_decayed = 1.5 * 0.7071067811865476;
        assert!((score.decayed_reward - expected_decayed).abs() < 1e-6);
        assert!((score.effective_reward - expected_decayed).abs() < 1e-6);
    }

    #[test]
    fn test_ranking() {
        let calculator = AdaptiveRewardCalculator::default();
        let mut episodes = vec![
            Episode {
                id: 1,
                created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
                raw_reward: 100.0,
                effective_reward: 10.0,
                task_type: TaskType::General,
                agent_type: AgentType::Human,
                complexity: ComplexityBand::Low,
            },
            Episode {
                id: 2,
                created_at: SystemTime::UNIX_EPOCH + Duration::from_secs(2000),
                raw_reward: 200.0,
                effective_reward: 20.0,
                task_type: TaskType::General,
                agent_type: AgentType::AI,
                complexity: ComplexityBand::High,
            },
        ];
        let ranked = calculator.rank_episodes(&mut episodes);
        assert_eq!(ranked[0].id, 2);
        assert_eq!(ranked[1].id, 1);
    }

    #[test]
    fn test_recompute() {
        let calculator = AdaptiveRewardCalculator::default();
        let mut episodes = vec![
            Episode {
                id: 1,
                created_at: SystemTime::now() - Duration::from_secs(3600),
                raw_reward: 100.0,
                effective_reward: 0.0, // outdated
                task_type: TaskType::General,
                agent_type: AgentType::Human,
                complexity: ComplexityBand::Low,
            },
        ];
        calculator.recompute_rewards(&mut episodes);
        assert!(episodes[0].effective_reward > 0.0);
    }
}
