//! F4.2 — multi-backend operation journal for eviction/delete intents.
//!
//! In-process journal of durable side effects so partial failures can be
//! repaired after restart of the reconcile path. Entries are append-only and
//! idempotent by `(op_id, backend)`.

use crate::memory::eviction::{EvictionBackend, EvictionBackendFailure};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Kind of durable operation recorded in the journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalOpKind {
    /// Capacity eviction delete of an episode (+ embeddings)
    CapacityEviction,
    /// Explicit episode delete
    EpisodeDelete,
    /// Embedding-only cleanup
    EmbeddingDelete,
}

/// Outcome recorded for a single backend attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalOutcome {
    /// Backend acknowledged success
    Success,
    /// Backend returned an error (message retained for operators)
    Failed {
        /// Error text (no secrets)
        error: String,
    },
    /// Intent recorded but not yet attempted
    Pending,
}

/// One journal row for a durable intent/outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Stable operation id (groups multi-backend work for one logical op)
    pub op_id: Uuid,
    /// Episode this op targets
    pub episode_id: Uuid,
    /// Operation kind
    pub kind: JournalOpKind,
    /// Backend surface
    pub backend: EvictionBackend,
    /// Outcome for this backend
    pub outcome: JournalOutcome,
    /// Unix millis when recorded
    pub recorded_at_ms: i64,
}

/// Bounded in-memory operation journal (F4.2).
#[derive(Debug)]
pub struct OperationJournal {
    entries: RwLock<VecDeque<JournalEntry>>,
    capacity: usize,
}

impl Default for OperationJournal {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl OperationJournal {
    /// Create a journal with a maximum retained entry count.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::new()),
            capacity: capacity.max(1),
        }
    }

    /// Append an entry, dropping oldest when over capacity.
    pub async fn record(&self, entry: JournalEntry) {
        let mut q = self.entries.write().await;
        if q.len() >= self.capacity {
            q.pop_front();
        }
        q.push_back(entry);
    }

    /// Record a batch of eviction failures as failed journal rows.
    pub async fn record_eviction_failures(&self, op_id: Uuid, failures: &[EvictionBackendFailure]) {
        let now = chrono_now_ms();
        for f in failures {
            self.record(JournalEntry {
                op_id,
                episode_id: f.episode_id,
                kind: JournalOpKind::CapacityEviction,
                backend: f.backend,
                outcome: JournalOutcome::Failed {
                    error: f.error.clone(),
                },
                recorded_at_ms: now,
            })
            .await;
        }
    }

    /// Record successful eviction deletes for the given ids and backends present.
    pub async fn record_eviction_successes(
        &self,
        op_id: Uuid,
        episode_ids: &[Uuid],
        backends: &[EvictionBackend],
    ) {
        let now = chrono_now_ms();
        for &episode_id in episode_ids {
            for &backend in backends {
                self.record(JournalEntry {
                    op_id,
                    episode_id,
                    kind: JournalOpKind::CapacityEviction,
                    backend,
                    outcome: JournalOutcome::Success,
                    recorded_at_ms: now,
                })
                .await;
            }
        }
    }

    /// Snapshot all entries (oldest first).
    pub async fn snapshot(&self) -> Vec<JournalEntry> {
        self.entries.read().await.iter().cloned().collect()
    }

    /// Entries that still need repair (Failed or Pending).
    pub async fn pending_repairs(&self) -> Vec<JournalEntry> {
        self.entries
            .read()
            .await
            .iter()
            .filter(|e| {
                matches!(
                    e.outcome,
                    JournalOutcome::Failed { .. } | JournalOutcome::Pending
                )
            })
            .cloned()
            .collect()
    }

    /// Number of retained entries.
    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    /// True when empty.
    pub async fn is_empty(&self) -> bool {
        self.entries.read().await.is_empty()
    }

    /// Clear the journal (tests / explicit reset).
    pub async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

fn chrono_now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Shared journal handle type.
pub type SharedJournal = Arc<OperationJournal>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn journal_records_and_lists_failures() {
        let j = OperationJournal::new(100);
        let op = Uuid::new_v4();
        let ep = Uuid::new_v4();
        j.record_eviction_failures(
            op,
            &[EvictionBackendFailure {
                episode_id: ep,
                backend: EvictionBackend::Durable,
                error: "disk full".into(),
            }],
        )
        .await;
        let pending = j.pending_repairs().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].episode_id, ep);
        assert!(matches!(pending[0].outcome, JournalOutcome::Failed { .. }));
    }

    #[tokio::test]
    async fn journal_respects_capacity() {
        let j = OperationJournal::new(3);
        for _ in 0..5 {
            j.record(JournalEntry {
                op_id: Uuid::new_v4(),
                episode_id: Uuid::new_v4(),
                kind: JournalOpKind::EpisodeDelete,
                backend: EvictionBackend::Cache,
                outcome: JournalOutcome::Success,
                recorded_at_ms: 0,
            })
            .await;
        }
        assert_eq!(j.len().await, 3);
    }
}
