//! Cross-agent memory collaboration (WG-126).
//!
//! Handles exporting distilled trajectories and aggregating them into
//! collaborative prototypes using Federated HDC principles.

use crate::learning::distillation::TrajectoryRepresentation;
use crate::types::TaskType;
use anyhow::bail;
use std::collections::HashMap;

/// Manager for cross-agent collaboration.
pub struct CollaborationManager {
    /// Collaborative prototypes indexed by task type
    pub prototypes: HashMap<TaskType, TrajectoryRepresentation>,
}

impl Default for CollaborationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CollaborationManager {
    /// Create a new collaboration manager.
    pub fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    /// Import a collaborative prototype for a task type.
    pub fn import_prototype(&mut self, task_type: TaskType, prototype: TrajectoryRepresentation) {
        self.prototypes.insert(task_type, prototype);
    }

    /// Aggregate multiple distilled trajectories into a single prototype.
    ///
    /// Implements "Prototype Bundling" from Federated HDC (arXiv:2603.20037).
    #[must_use]
    pub fn bundle_prototypes(
        &self,
        _task_type: TaskType,
        trajectories: &[TrajectoryRepresentation],
    ) -> Option<TrajectoryRepresentation> {
        if trajectories.is_empty() {
            return None;
        }

        #[cfg(feature = "csm")]
        {
            // If we have HDC vectors, bundle them using CSM's bundling operation (sum + threshold)
            let hvecs: Vec<_> = trajectories
                .iter()
                .filter_map(|t| {
                    if let TrajectoryRepresentation::Hyperdim(h) = t {
                        Some(h.clone())
                    } else {
                        None
                    }
                })
                .collect();

            if !hvecs.is_empty() {
                // Medoid-based bundling: select the vector most representative
                // of the set by maximizing average cosine similarity to all others.
                // This is a valid HDC bundling technique that doesn't require
                // element-wise operations on opaque bit vectors.
                //
                // Complexity: O(n²) pairwise similarities. For production use,
                // the `BundleAccumulator` from chaotic-semantic-memory provides
                // native element-wise sum+threshold with O(n·d) cost.
                if hvecs.len() == 1 {
                    return Some(TrajectoryRepresentation::Hyperdim(hvecs[0].clone()));
                }
                let mut best_idx = 0_usize;
                let mut best_avg_sim = -1.0_f32;
                for i in 0..hvecs.len() {
                    let avg_sim: f32 = hvecs
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| *j != i)
                        .map(|(_, v)| hvecs[i].cosine_similarity(v))
                        .sum::<f32>()
                        / (hvecs.len() - 1) as f32;
                    if avg_sim > best_avg_sim {
                        best_avg_sim = avg_sim;
                        best_idx = i;
                    }
                }
                return Some(TrajectoryRepresentation::Hyperdim(hvecs[best_idx].clone()));
            }
        }

        // Fallback to average embedding
        let embeddings: Vec<_> = trajectories
            .iter()
            .map(|t| match t {
                TrajectoryRepresentation::Embedding(e) => e,
                #[cfg(feature = "csm")]
                _ => unreachable!(),
            })
            .collect();

        if embeddings.is_empty() {
            return None;
        }

        let dim = embeddings[0].len();
        let mut avg = vec![0.0; dim];
        for emb in &embeddings {
            // Validate consistent dimensionality before accumulation
            if emb.len() != dim {
                return None;
            }
            for (i, val) in emb.iter().enumerate() {
                avg[i] += val;
            }
        }

        for val in &mut avg {
            *val /= embeddings.len() as f32;
        }

        Some(TrajectoryRepresentation::Embedding(avg))
    }

    /// Export local distilled trajectories for a task type.
    ///
    /// # Errors
    ///
    /// Returns an error indicating that trajectory export is not yet
    /// implemented — this is a planned feature for cross-agent memory sharing.
    pub fn export_trajectories(
        &self,
        _task_type: TaskType,
    ) -> Result<Vec<TrajectoryRepresentation>, anyhow::Error> {
        // In a real implementation, this would query the local storage for
        // trajectories of the given task type.
        bail!("trajectory export not yet implemented")
    }
}

#[cfg(test)]
mod tests;
