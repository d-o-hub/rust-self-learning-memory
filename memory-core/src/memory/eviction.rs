//! Capacity eviction durable delete + partial-failure reconciliation (S1.4b).

use crate::error::Result;
use crate::storage::StorageBackend;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

/// Which storage surface failed during capacity eviction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvictionBackend {
    /// Cache / redb layer
    Cache,
    /// Durable Turso layer
    Durable,
    /// Embedding row for the episode
    EmbeddingCache,
    /// Embedding row on durable storage
    EmbeddingDurable,
}

impl std::fmt::Display for EvictionBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cache => write!(f, "cache"),
            Self::Durable => write!(f, "durable"),
            Self::EmbeddingCache => write!(f, "embedding_cache"),
            Self::EmbeddingDurable => write!(f, "embedding_durable"),
        }
    }
}

/// Single backend delete failure for an evicted episode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvictionBackendFailure {
    /// Episode that was removed from in-memory map
    pub episode_id: Uuid,
    /// Backend that failed
    pub backend: EvictionBackend,
    /// Error message (no secrets expected)
    pub error: String,
}

/// Outcome of attempting durable deletes for capacity-evicted episodes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvictionOutcome {
    /// Episodes removed from the in-memory map
    pub evicted_ids: Vec<Uuid>,
    /// Per-backend failures that need reconciliation
    pub failures: Vec<EvictionBackendFailure>,
}

impl EvictionOutcome {
    /// True when every configured backend delete succeeded.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.failures.is_empty()
    }

    /// True when in-memory eviction happened but backends partially failed.
    #[must_use]
    pub fn needs_reconciliation(&self) -> bool {
        !self.failures.is_empty()
    }
}

/// Delete capacity-evicted episodes from configured backends (S1.4 / S1.4b).
///
/// In-memory removal is the caller's responsibility. This function is
/// idempotent per backend: repeated deletes of the same id are allowed.
pub async fn delete_evicted_from_backends(
    evicted_ids: &[Uuid],
    cache: Option<&Arc<dyn StorageBackend>>,
    durable: Option<&Arc<dyn StorageBackend>>,
) -> EvictionOutcome {
    let mut outcome = EvictionOutcome {
        evicted_ids: evicted_ids.to_vec(),
        failures: Vec::new(),
    };

    for &evicted_id in evicted_ids {
        if let Some(cache) = cache {
            if let Err(e) = cache.delete_episode(evicted_id).await {
                outcome.failures.push(EvictionBackendFailure {
                    episode_id: evicted_id,
                    backend: EvictionBackend::Cache,
                    error: e.to_string(),
                });
            }
            let embedding_key = evicted_id.to_string();
            if let Err(e) = cache.delete_embedding(&embedding_key).await {
                outcome.failures.push(EvictionBackendFailure {
                    episode_id: evicted_id,
                    backend: EvictionBackend::EmbeddingCache,
                    error: e.to_string(),
                });
            }
        }
        if let Some(durable) = durable {
            if let Err(e) = durable.delete_episode(evicted_id).await {
                outcome.failures.push(EvictionBackendFailure {
                    episode_id: evicted_id,
                    backend: EvictionBackend::Durable,
                    error: e.to_string(),
                });
            }
            let embedding_key = evicted_id.to_string();
            if let Err(e) = durable.delete_embedding(&embedding_key).await {
                outcome.failures.push(EvictionBackendFailure {
                    episode_id: evicted_id,
                    backend: EvictionBackend::EmbeddingDurable,
                    error: e.to_string(),
                });
            }
        }
    }

    if outcome.needs_reconciliation() {
        warn!(
            evicted_count = outcome.evicted_ids.len(),
            failure_count = outcome.failures.len(),
            "Capacity eviction partially failed; backends need reconciliation"
        );
    } else if !outcome.evicted_ids.is_empty() {
        info!(
            evicted_ids = ?outcome.evicted_ids,
            "Capacity-evicted episodes deleted from memory and storage backends"
        );
    }

    outcome
}

use super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Pending capacity-eviction backend failures (S1.4b).
    pub async fn pending_eviction_failures(&self) -> Vec<EvictionBackendFailure> {
        self.pending_eviction_failures.read().await.clone()
    }

    /// Retry durable deletes for prior partial capacity evictions.
    ///
    /// Returns the failures that still remain after the retry attempt.
    ///
    /// # Errors
    /// Propagates unexpected storage errors only when the reconciliation
    /// helper itself fails; per-backend delete errors are captured as
    /// remaining failures.
    pub async fn reconcile_pending_evictions(&self) -> Result<Vec<EvictionBackendFailure>> {
        let snapshot = {
            let pending = self.pending_eviction_failures.read().await;
            pending.clone()
        };
        if snapshot.is_empty() {
            return Ok(Vec::new());
        }

        let remaining = reconcile_eviction_failures(
            &snapshot,
            self.cache_storage.as_ref(),
            self.turso_storage.as_ref(),
        )
        .await?;

        {
            let mut pending = self.pending_eviction_failures.write().await;
            *pending = remaining.clone();
        }

        if remaining.is_empty() {
            info!("Capacity eviction reconciliation complete; no pending failures");
        } else {
            warn!(
                remaining = remaining.len(),
                "Capacity eviction reconciliation still has pending failures"
            );
        }

        Ok(remaining)
    }
}

/// Retry pending backend deletes; returns failures that still remain.
pub async fn reconcile_eviction_failures(
    failures: &[EvictionBackendFailure],
    cache: Option<&Arc<dyn StorageBackend>>,
    durable: Option<&Arc<dyn StorageBackend>>,
) -> Result<Vec<EvictionBackendFailure>> {
    let mut remaining = Vec::new();

    for failure in failures {
        let retry_err = match failure.backend {
            EvictionBackend::Cache => {
                if let Some(cache) = cache {
                    cache.delete_episode(failure.episode_id).await.err()
                } else {
                    None
                }
            }
            EvictionBackend::Durable => {
                if let Some(durable) = durable {
                    durable.delete_episode(failure.episode_id).await.err()
                } else {
                    None
                }
            }
            EvictionBackend::EmbeddingCache => {
                if let Some(cache) = cache {
                    cache
                        .delete_embedding(&failure.episode_id.to_string())
                        .await
                        .err()
                } else {
                    None
                }
            }
            EvictionBackend::EmbeddingDurable => {
                if let Some(durable) = durable {
                    durable
                        .delete_embedding(&failure.episode_id.to_string())
                        .await
                        .err()
                } else {
                    None
                }
            }
        };

        if let Some(e) = retry_err {
            remaining.push(EvictionBackendFailure {
                episode_id: failure.episode_id,
                backend: failure.backend,
                error: e.to_string(),
            });
        }
    }

    Ok(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eviction_outcome_complete_when_no_failures() {
        let o = EvictionOutcome {
            evicted_ids: vec![Uuid::nil()],
            failures: vec![],
        };
        assert!(o.is_complete());
        assert!(!o.needs_reconciliation());
    }

    #[test]
    fn eviction_outcome_needs_reconciliation_with_failures() {
        let o = EvictionOutcome {
            evicted_ids: vec![Uuid::nil()],
            failures: vec![EvictionBackendFailure {
                episode_id: Uuid::nil(),
                backend: EvictionBackend::Durable,
                error: "disk full".into(),
            }],
        };
        assert!(!o.is_complete());
        assert!(o.needs_reconciliation());
        assert_eq!(o.failures[0].backend.to_string(), "durable");
    }
}
