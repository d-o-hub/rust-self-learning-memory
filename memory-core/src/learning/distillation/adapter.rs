//! Trajectory adapter for contrastive learning (WG-126).

use crate::learning::distillation::TrajectoryRepresentation;
use crate::types::TaskType;
use anyhow::Result;

/// Adapter for aligning trajectory representations using contrastive learning.
pub struct TrajectoryAdapter {
    /// Task type this adapter is optimized for
    pub task_type: TaskType,
    /// Adaptation matrix for embedding alignment
    pub matrix: Vec<Vec<f32>>,
    /// Number of trajectories trained on
    pub trained_count: usize,
}

impl TrajectoryAdapter {
    /// Create a new trajectory adapter with identity matrix.
    pub fn new(task_type: TaskType, dimension: usize) -> Self {
        let mut matrix = vec![vec![0.0; dimension]; dimension];
        for (i, row) in matrix.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        Self {
            task_type,
            matrix,
            trained_count: 0,
        }
    }

    /// Apply the adapter to a trajectory representation.
    pub fn adapt(&self, representation: TrajectoryRepresentation) -> TrajectoryRepresentation {
        match representation {
            TrajectoryRepresentation::Embedding(emb) => {
                let dim = emb.len();
                let mut adapted = vec![0.0; dim];
                for (i, adapted_val) in adapted.iter_mut().enumerate() {
                    for (j, emb_val) in emb.iter().enumerate() {
                        *adapted_val += emb_val * self.matrix[j][i];
                    }
                }
                TrajectoryRepresentation::Embedding(adapted)
            }
            // HDC adaptation is more complex, returning unchanged for now
            #[cfg(feature = "csm")]
            TrajectoryRepresentation::Hyperdim(hvec) => TrajectoryRepresentation::Hyperdim(hvec),
        }
    }
}

/// A triplet of trajectories for contrastive learning.
pub struct TrajectoryTriplet {
    /// Anchor trajectory
    pub anchor: TrajectoryRepresentation,
    /// Positive trajectory (similar outcome/task)
    pub positive: TrajectoryRepresentation,
    /// Negative trajectory (dissimilar outcome/task)
    pub negative: TrajectoryRepresentation,
}

/// Trainer for trajectory adapters.
pub struct TrajectoryTrainer {
    pub learning_rate: f32,
    pub margin: f32,
    pub epochs: usize,
}

impl Default for TrajectoryTrainer {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            margin: 0.5,
            epochs: 50,
        }
    }
}

impl TrajectoryTrainer {
    /// Train an adapter using a set of triplets.
    pub fn train(
        &self,
        adapter: &mut TrajectoryAdapter,
        triplets: &[TrajectoryTriplet],
    ) -> Result<()> {
        if triplets.is_empty() {
            return Ok(());
        }

        for _ in 0..self.epochs {
            for triplet in triplets {
                self.update_step(adapter, triplet);
            }
        }

        adapter.trained_count += triplets.len();
        Ok(())
    }

    fn update_step(&self, adapter: &mut TrajectoryAdapter, triplet: &TrajectoryTriplet) {
        #[allow(clippy::infallible_destructuring_match)]
        let (anchor, pos, neg) = match (&triplet.anchor, &triplet.positive, &triplet.negative) {
            (
                TrajectoryRepresentation::Embedding(a),
                TrajectoryRepresentation::Embedding(p),
                TrajectoryRepresentation::Embedding(n),
            ) => (a, p, n),
            #[cfg(feature = "csm")]
            _ => return, // Mix of types not supported yet
        };

        let dim = anchor.len();

        // Current adapted versions
        #[allow(clippy::infallible_destructuring_match)]
        let a_adapted = match adapter.adapt(TrajectoryRepresentation::Embedding(anchor.clone())) {
            TrajectoryRepresentation::Embedding(v) => v,
            #[cfg(feature = "csm")]
            _ => unreachable!(),
        };
        #[allow(clippy::infallible_destructuring_match)]
        let p_adapted = match adapter.adapt(TrajectoryRepresentation::Embedding(pos.clone())) {
            TrajectoryRepresentation::Embedding(v) => v,
            #[cfg(feature = "csm")]
            _ => unreachable!(),
        };
        #[allow(clippy::infallible_destructuring_match)]
        let n_adapted = match adapter.adapt(TrajectoryRepresentation::Embedding(neg.clone())) {
            TrajectoryRepresentation::Embedding(v) => v,
            #[cfg(feature = "csm")]
            _ => unreachable!(),
        };

        let d_pos = euclidean_distance(&a_adapted, &p_adapted);
        let d_neg = euclidean_distance(&a_adapted, &n_adapted);

        // Triplet loss: max(0, d_pos - d_neg + margin)
        if d_pos - d_neg + self.margin > 0.0 {
            // Simplified gradient update
            for i in 0..dim {
                let diff_a_p = a_adapted[i] - p_adapted[i];
                let diff_a_n = a_adapted[i] - n_adapted[i];
                for j in 0..dim {
                    let grad_pos = (anchor[j] - pos[j]) * diff_a_p;
                    let grad_neg = (anchor[j] - neg[j]) * diff_a_n;
                    adapter.matrix[j][i] -= self.learning_rate * (grad_pos - grad_neg);
                }
            }
        }
    }
}

fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}
