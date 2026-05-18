//! Trajectory distillation for agent-agnostic memory (WG-126).
//!
//! Inspired by MemCollab (arXiv:2603.23234): distill episode trajectories
//! (step sequences, signatures, outcomes) into compact representations
//! that can be shared across agents.

use serde::{Deserialize, Serialize};
use crate::episode::Episode;
use crate::retrieval::signature::ExecutionSignature;

#[cfg(feature = "csm")]
use chaotic_semantic_memory::HVec10240;

/// Compact representation of an execution trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrajectoryRepresentation {
    /// Hyperdimensional vector representation (Tier 2/3 style)
    #[cfg(feature = "csm")]
    Hyperdim(HVec10240),
    /// Standard embedding vector representation (Tier 4 style)
    Embedding(Vec<f32>),
}

impl TrajectoryRepresentation {
    /// Compute similarity with another trajectory representation.
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f32 {
        match (self, other) {
            #[cfg(feature = "csm")]
            (Self::Hyperdim(a), Self::Hyperdim(b)) => a.cosine_similarity(b),
            (Self::Embedding(a), Self::Embedding(b)) => {
                crate::embeddings_simple::cosine_similarity(a, b)
            }
            #[cfg(feature = "csm")]
            _ => 0.0, // Incompatible types
        }
    }
}

/// Distiller for converting episodes into compact trajectory representations.
#[derive(Clone)]
pub struct TrajectoryDistiller {
    /// Whether to use HDC for distillation
    pub use_hdc: bool,
}

impl TrajectoryDistiller {
    /// Create a new trajectory distiller.
    #[must_use]
    pub fn new(use_hdc: bool) -> Self {
        Self { use_hdc }
    }

    /// Distill an episode into a compact trajectory representation.
    ///
    /// Combines execution signature, tool sequences, and outcomes.
    pub fn distill(&self, episode: &Episode) -> TrajectoryRepresentation {
        // Generate execution signature first (captures tools, errors, patterns)
        let mut signature = ExecutionSignature::new(episode.episode_id);

        for step in &episode.steps {
            signature.add_tool(&step.tool);
            signature.record_step(step.is_success());
            // In a real implementation, we would extract errors from step results here
        }

        signature.set_avg_latency(
            episode.steps.iter().map(|s| s.latency_ms).sum::<u64>() /
            episode.steps.len().max(1) as u64
        );

        // Convert signature + episode data to text for encoding/embedding
        let _distillation_text = format!(
            "Task: {} | Tools: {:?} | Steps: {} | Success: {} | Outcome: {:?}",
            episode.task_description,
            signature.tools,
            signature.total_steps,
            signature.success_rate(),
            episode.outcome
        );

        #[cfg(feature = "csm")]
        if self.use_hdc {
            let encoder = chaotic_semantic_memory::encoder::TextEncoder::new();
            return TrajectoryRepresentation::Hyperdim(encoder.encode(&_distillation_text));
        }

        // Fallback to simple embedding (mocked for now as we don't have async here)
        // In a real implementation, this would use the semantic service
        TrajectoryRepresentation::Embedding(vec![0.0; 1536])
    }
}

pub mod adapter;

#[cfg(test)]
mod tests;
