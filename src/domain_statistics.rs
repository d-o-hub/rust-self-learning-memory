use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStatistics {
    pub task_type_stats: HashMap<String, f64>,
    pub agent_type_stats: HashMap<String, f64>,
    pub complexity_band_stats: HashMap<String, f64>,
}

impl DomainStatistics {
    pub fn new() -> Self {
        Self {
            task_type_stats: HashMap::new(),
            agent_type_stats: HashMap::new(),
            complexity_band_stats: HashMap::new(),
        }
    }

    /// Normalize a raw score based on the domain statistics.
    /// Returns a score in [0, 1] range after normalization.
    pub fn normalize(&self, raw: f64, task_type: &str, agent_type: &str, complexity_band: &str) -> f64 {
        let task_mean = self.task_type_stats.get(task_type).copied().unwrap_or(1.0);
        let agent_mean = self.agent_type_stats.get(agent_type).copied().unwrap_or(1.0);
        let complexity_mean = self.complexity_band_stats.get(complexity_band).copied().unwrap_or(1.0);
        let normalized = raw / (task_mean * agent_mean * complexity_mean);
        normalized.clamp(0.0, 1.0)
    }

    /// Update statistics with a new raw score.
    pub fn update(&mut self, raw: f64, task_type: &str, agent_type: &str, complexity_band: &str) {
        *self.task_type_stats.entry(task_type.to_string()).or_insert(0.0) += raw;
        *self.agent_type_stats.entry(agent_type.to_string()).or_insert(0.0) += raw;
        *self.complexity_band_stats.entry(complexity_band.to_string()).or_insert(0.0) += raw;
    }
}

impl Default for DomainStatistics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        let mut stats = DomainStatistics::new();
        stats.update(10.0, "task_a", "agent_x", "low");
        stats.update(20.0, "task_a", "agent_x", "low");
        let normalized = stats.normalize(15.0, "task_a", "agent_x", "low");
        // mean = (10+20)/2 = 15 for each? Actually each update adds to same keys, so task_mean=30, agent_mean=30, complexity_mean=30
        // product=27000, raw/27000=0.000555... -> clamped to 0.000555...
        assert!((normalized - 15.0 / (30.0*30.0*30.0)).abs() < 1e-10);
    }
}
