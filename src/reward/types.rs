use serde::{Deserialize, Serialize};

/// Represents the different components of a reward score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualRewardScore {
    pub raw_reward: f64,
    pub normalized_reward: f64,
    pub decayed_reward: f64,
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