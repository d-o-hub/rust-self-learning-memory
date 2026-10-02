//! Episode completion receipts (issue #1080).
//!
//! Split out of [`completion`](super::completion) to keep that module under
//! the repository's per-file LOC ceiling.

use super::SelfLearningMemory;
use uuid::Uuid;

/// Durability reached by a completion when it returned (issue #1080).
///
/// The receipt that carries this value is sourced from live durable-write
/// queue statistics, so [`EpisodeDurability::Queued`] can never be mistaken
/// for a committed remote write. See ADR-075 (D2 follow-up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodeDurability {
    /// No durable backend is configured: the episode is committed to the
    /// local backends only.
    Local,
    /// Every configured backend (including the durable one) committed the
    /// episode before the call returned — the synchronous path, or the
    /// durable write queue is disabled.
    Committed,
    /// Local state committed; the durable backend accepted the completion's
    /// write into the bounded background queue and it has not committed yet.
    ///
    /// Callers that require remote durability must drain the queue with
    /// [`flush_durable_writes`](SelfLearningMemory::flush_durable_writes)
    /// (or [`complete_episode`](SelfLearningMemory::complete_episode) after
    /// draining) and treat drain errors as failed completions.
    Queued,
}

/// Receipt for a completed episode, reporting the durability reached when
/// the completion returned (issue #1080).
///
/// Issued by
/// [`complete_episode_checked`](SelfLearningMemory::complete_episode_checked).
#[derive(Debug, Clone)]
pub struct EpisodeCompletionReceipt {
    /// The completed episode.
    pub episode_id: Uuid,
    /// Durability state at return time.
    pub durability: EpisodeDurability,
    /// Waiting durable writes after this completion; `0` unless
    /// [`EpisodeDurability::Queued`].
    pub queue_depth: usize,
}

impl SelfLearningMemory {
    /// Build the completion receipt from live durable-write queue state.
    ///
    /// `Committed` is claimed only on the synchronous path (durable backend
    /// configured, queue disabled); with the queue enabled the completion
    /// write is reported as [`EpisodeDurability::Queued`] until a drain
    /// confirms it.
    pub(super) async fn completion_receipt(&self, episode_id: Uuid) -> EpisodeCompletionReceipt {
        let (durability, queue_depth) = match (&self.turso_storage, &self.durable_write_queue) {
            (Some(_), Some(queue)) => (
                EpisodeDurability::Queued,
                queue.get_stats().await.current_depth,
            ),
            (Some(_), None) => (EpisodeDurability::Committed, 0),
            (None, _) => (EpisodeDurability::Local, 0),
        };

        EpisodeCompletionReceipt {
            episode_id,
            durability,
            queue_depth,
        }
    }
}
