//! Bounded background queue for durable (Turso) episode writes (#967).
//!
//! Episode completion stays responsive by committing local state
//! synchronously and moving remote persistence off the completion path:
//!
//! - The cache/redb write remains synchronous and hard-errors (ADR-075,
//!   local tier of the D1 split in `plans/GOAP_FEATURE_WAVE_2026-09-04.md`).
//! - The Turso write is enqueued here and persisted by a single ordered
//!   background worker in transactional batches with retry/backoff.
//! - The queue is **opt-in** (`MemoryConfig::durable_write_queue`); when it
//!   is `None`, completion keeps the historical all-synchronous behavior.
//!
//! Backpressure is explicit: a full queue rejects with
//! [`Error::QuotaExceeded`](crate::error::Error::QuotaExceeded) instead of
//! silently dropping writes. Retried batches are idempotent (`INSERT OR
//! REPLACE`), so retries cannot create duplicate episodes.

use crate::Episode;
use crate::error::{Error, Result};
use crate::memory::op_journal::SharedJournal;
use crate::storage::StorageBackend;
use std::collections::VecDeque;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Default maximum queued episodes before backpressure rejects writes.
pub const DEFAULT_WRITE_QUEUE_SIZE: usize = 1000;
/// Default episodes per transactional batch commit.
pub const DEFAULT_WRITE_BATCH_SIZE: usize = 50;
/// Default worker poll interval (ms) when the queue is empty.
pub const DEFAULT_WRITE_POLL_INTERVAL_MS: u64 = 100;
/// Default maximum batch retries before a write is declared permanently failed.
pub const DEFAULT_WRITE_MAX_RETRIES: u32 = 3;
/// Default base retry delay (ms); doubled per attempt up to the max.
pub const DEFAULT_WRITE_RETRY_BASE_DELAY_MS: u64 = 100;
/// Default maximum retry delay (ms).
pub const DEFAULT_WRITE_RETRY_MAX_DELAY_MS: u64 = 5000;
/// Maximum permanently-failed episode IDs retained for operator inspection.
const MAX_RETAINED_FAILURES: usize = 256;

/// Configuration for the background durable-write queue.
#[derive(Debug, Clone)]
pub struct WriteQueueConfig {
    /// Maximum queued episodes (0 = unlimited). Full queues reject writes.
    pub max_queue_size: usize,
    /// Episodes per transactional batch commit.
    pub batch_size: usize,
    /// Worker poll interval when the queue is empty (milliseconds).
    pub poll_interval_ms: u64,
    /// Maximum batch retries before permanent failure.
    pub max_retries: u32,
    /// Base retry delay (milliseconds, doubled per attempt).
    pub retry_base_delay_ms: u64,
    /// Maximum retry delay (milliseconds).
    pub retry_max_delay_ms: u64,
}

impl Default for WriteQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_size: DEFAULT_WRITE_QUEUE_SIZE,
            batch_size: DEFAULT_WRITE_BATCH_SIZE,
            poll_interval_ms: DEFAULT_WRITE_POLL_INTERVAL_MS,
            max_retries: DEFAULT_WRITE_MAX_RETRIES,
            retry_base_delay_ms: DEFAULT_WRITE_RETRY_BASE_DELAY_MS,
            retry_max_delay_ms: DEFAULT_WRITE_RETRY_MAX_DELAY_MS,
        }
    }
}

/// Snapshot of durable-write queue operations.
#[derive(Debug, Clone, Default)]
pub struct WriteQueueStats {
    /// Total episodes accepted into the queue.
    pub total_enqueued: u64,
    /// Total episodes acknowledged by the durable backend.
    pub total_written: u64,
    /// Total episodes declared permanently failed.
    pub total_failed: u64,
    /// Total episodes requeued after a failed batch attempt.
    pub total_retried: u64,
    /// Episodes currently waiting (excludes the in-flight batch).
    pub current_depth: usize,
    /// Age of the oldest waiting write in milliseconds (0 when empty).
    pub oldest_enqueued_age_ms: u64,
    /// Permanently-failed episode IDs, oldest first (capped).
    pub failed_episode_ids: Vec<Uuid>,
}

/// One queued durable write with its retry state.
struct QueuedWrite {
    episode: Episode,
    enqueued_at: Instant,
    attempts: u32,
}

/// Background queue draining completed episodes to the durable backend.
///
/// A single ordered worker preserves completion order and keeps retries
/// idempotent. See the module docs for the durability contract.
pub struct DurableWriteQueue {
    config: WriteQueueConfig,
    queue: Arc<Mutex<VecDeque<QueuedWrite>>>,
    turso: Arc<dyn StorageBackend>,
    journal: SharedJournal,
    stats: Arc<RwLock<WriteQueueStats>>,
    /// Batches popped but not yet acknowledged (for accurate drain waits).
    in_flight: Arc<AtomicUsize>,
    shutdown: Arc<RwLock<bool>>,
}

impl DurableWriteQueue {
    /// Create a queue bound to a durable backend and the operation journal.
    #[must_use]
    pub fn new(
        config: WriteQueueConfig,
        turso: Arc<dyn StorageBackend>,
        journal: SharedJournal,
    ) -> Self {
        Self {
            config,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            turso,
            journal,
            stats: Arc::new(RwLock::new(WriteQueueStats::default())),
            in_flight: Arc::new(AtomicUsize::new(0)),
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Enqueue a completed episode for background durable persistence.
    ///
    /// A pending write for the same episode is *replaced*, not duplicated:
    /// completion enqueues once at the durable seam and the synchronous
    /// pattern path re-enqueues with pattern/heuristic IDs attached, so the
    /// newest state always supplants the older one (remote rows are
    /// `INSERT OR REPLACE`, making this coalescing lossless).
    ///
    /// # Errors
    ///
    /// Returns [`Error::QuotaExceeded`](crate::error::Error::QuotaExceeded)
    /// when the queue is at capacity. Callers must treat this as a failed
    /// completion, never as a silent drop.
    #[instrument(skip(self, episode), fields(episode_id = %episode.episode_id))]
    pub async fn enqueue_episode(&self, episode: Episode) -> Result<()> {
        let mut queue = self.queue.lock().await;
        let id = episode.episode_id;
        // Coalesce: drop any still-waiting write for this episode. An
        // in-flight batch already popped keeps its copy; the requeued newer
        // state persists right after, preserving final-state correctness.
        let already_waiting = queue.iter().any(|w| w.episode.episode_id == id);
        if !already_waiting
            && self.config.max_queue_size > 0
            && queue.len() >= self.config.max_queue_size
        {
            warn!(
                queue_size = queue.len(),
                max_size = self.config.max_queue_size,
                "Durable write queue at capacity"
            );
            return Err(Error::QuotaExceeded(format!(
                "durable write queue at capacity ({})",
                self.config.max_queue_size
            )));
        }
        queue.retain(|w| w.episode.episode_id != id);
        queue.push_back(QueuedWrite {
            episode,
            enqueued_at: Instant::now(),
            attempts: 0,
        });
        let depth = queue.len();
        drop(queue);

        let mut stats = self.stats.write().await;
        stats.total_enqueued += 1;
        stats.current_depth = depth;
        debug!(queue_depth = depth, "Enqueued episode for durable write");
        Ok(())
    }

    /// Start the single ordered background worker.
    ///
    /// One worker preserves completion order across batches; throughput
    /// comes from transactional batch commits, not parallelism.
    pub fn start_workers(&self) {
        info!("Starting durable write worker");
        let config = self.config.clone();
        let queue = Arc::clone(&self.queue);
        let turso = Arc::clone(&self.turso);
        let journal = Arc::clone(&self.journal);
        let stats = Arc::clone(&self.stats);
        let in_flight = Arc::clone(&self.in_flight);
        let shutdown = Arc::clone(&self.shutdown);
        tokio::spawn(async move {
            Self::worker_loop(config, queue, turso, journal, stats, in_flight, shutdown).await;
        });
    }

    /// Main worker loop: drain batches, commit transactionally, retry/backoff.
    #[allow(clippy::too_many_arguments)]
    async fn worker_loop(
        config: WriteQueueConfig,
        queue: Arc<Mutex<VecDeque<QueuedWrite>>>,
        turso: Arc<dyn StorageBackend>,
        journal: SharedJournal,
        stats: Arc<RwLock<WriteQueueStats>>,
        in_flight: Arc<AtomicUsize>,
        shutdown: Arc<RwLock<bool>>,
    ) {
        let poll_interval = Duration::from_millis(config.poll_interval_ms);
        loop {
            if *shutdown.read().await {
                // Drain remaining work so shutdown loses nothing already
                // accepted; retries still bound by max_retries, so this
                // terminates.
                if queue.lock().await.is_empty() && in_flight.load(Ordering::SeqCst) == 0 {
                    debug!("Durable write worker shutting down gracefully");
                    break;
                }
            }

            let batch = Self::pop_batch(&config, &queue, &stats).await;
            if batch.is_empty() {
                sleep(poll_interval).await;
                continue;
            }
            in_flight.store(batch.len(), Ordering::SeqCst);
            Self::write_batch(&config, &turso, &journal, &stats, &queue, batch).await;
            in_flight.store(0, Ordering::SeqCst);
        }
    }

    /// Pop up to `batch_size` writes, oldest first.
    async fn pop_batch(
        config: &WriteQueueConfig,
        queue: &Mutex<VecDeque<QueuedWrite>>,
        stats: &RwLock<WriteQueueStats>,
    ) -> Vec<QueuedWrite> {
        let mut guard = queue.lock().await;
        let take = config.batch_size.max(1).min(guard.len());
        let batch: Vec<QueuedWrite> = guard.drain(..take).collect();
        let depth = guard.len();
        drop(guard);
        stats.write().await.current_depth = depth;
        batch
    }

    /// Retain one permanently-failed ID, dropping oldest past the cap.
    fn retain_failure(stats: &mut WriteQueueStats, id: Uuid) {
        if stats.failed_episode_ids.len() >= MAX_RETAINED_FAILURES {
            stats.failed_episode_ids.remove(0);
        }
        stats.failed_episode_ids.push(id);
    }

    /// Commit one batch with retry/backoff; park permanent failures.
    async fn write_batch(
        config: &WriteQueueConfig,
        turso: &Arc<dyn StorageBackend>,
        journal: &SharedJournal,
        stats: &RwLock<WriteQueueStats>,
        queue: &Arc<Mutex<VecDeque<QueuedWrite>>>,
        mut batch: Vec<QueuedWrite>,
    ) {
        let episodes: Vec<Episode> = batch.iter().map(|w| w.episode.clone()).collect();
        let ids: Vec<Uuid> = batch.iter().map(|w| w.episode.episode_id).collect();
        match turso.store_episodes_batch(&episodes).await {
            Ok(()) => {
                let op_id = Uuid::new_v4();
                journal.record_durable_write_successes(op_id, &ids).await;
                let mut stats = stats.write().await;
                stats.total_written += batch.len() as u64;
                info!(
                    written = batch.len(),
                    queue_depth = stats.current_depth,
                    "Durable write batch committed"
                );
            }
            Err(e) => {
                let error_text = e.to_string();
                let mut retryable = Vec::with_capacity(batch.len());
                let mut permanent = Vec::with_capacity(batch.len());
                for mut write in batch.drain(..) {
                    write.attempts += 1;
                    if write.attempts > config.max_retries {
                        permanent.push(write);
                    } else {
                        retryable.push(write);
                    }
                }

                if !permanent.is_empty() {
                    let op_id = Uuid::new_v4();
                    let failures: Vec<(Uuid, String)> = permanent
                        .iter()
                        .map(|w| (w.episode.episode_id, error_text.clone()))
                        .collect();
                    journal
                        .record_durable_write_failures(op_id, &failures)
                        .await;
                    let mut stats = stats.write().await;
                    stats.total_failed += permanent.len() as u64;
                    for (id, _) in &failures {
                        Self::retain_failure(&mut stats, *id);
                    }
                    warn!(
                        failed = permanent.len(),
                        error = %error_text,
                        "Durable writes exhausted retries and were parked as permanent failures"
                    );
                }

                if !retryable.is_empty() {
                    // Exponential backoff, capped; requeue at the front to
                    // preserve completion order ahead of newer writes.
                    let attempts = retryable[0].attempts;
                    let backoff_ms = config
                        .retry_base_delay_ms
                        .saturating_mul(1u64 << attempts.min(10))
                        .min(config.retry_max_delay_ms);
                    sleep(Duration::from_millis(backoff_ms)).await;
                    let mut queue = queue.lock().await;
                    let retried = retryable.len();
                    for write in retryable.into_iter().rev() {
                        queue.push_front(write);
                    }
                    let depth = queue.len();
                    drop(queue);
                    let mut stats = stats.write().await;
                    stats.total_retried += retried as u64;
                    stats.current_depth = depth;
                    debug!(
                        retried,
                        backoff_ms,
                        queue_depth = depth,
                        "Durable write batch failed; requeued with backoff"
                    );
                }
            }
        }
    }

    /// Current number of waiting writes (excludes the in-flight batch).
    pub async fn queue_size(&self) -> usize {
        self.queue.lock().await.len()
    }

    /// Snapshot of queue statistics, including oldest-waiting age.
    pub async fn get_stats(&self) -> WriteQueueStats {
        let queue = self.queue.lock().await;
        let oldest_age_ms = queue
            .front()
            .map(|w| {
                w.enqueued_at
                    .elapsed()
                    .as_millis()
                    .min(u128::from(u64::MAX)) as u64
            })
            .unwrap_or(0);
        let depth = queue.len();
        drop(queue);
        let mut stats = self.stats.read().await.clone();
        stats.current_depth = depth;
        stats.oldest_enqueued_age_ms = oldest_age_ms;
        stats
    }

    /// Signal the worker to drain remaining work and exit.
    pub async fn shutdown(&self) {
        info!("Initiating durable write queue shutdown");
        *self.shutdown.write().await = true;
    }

    /// Wait until no writes are waiting or in flight.
    ///
    /// # Returns
    ///
    /// `true` if the queue drained, `false` on timeout.
    pub async fn wait_until_empty(&self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.queue_size().await == 0 && self.in_flight.load(Ordering::SeqCst) == 0 {
                return true;
            }
            sleep(Duration::from_millis(50)).await;
        }
        false
    }

    /// Drain the queue and surface permanent failures.
    ///
    /// Used by CLI shutdown, tests, and operators that need the remote
    /// durability guarantee on demand (the D1 split). Permanent
    /// failures recorded during the drain surface as
    /// [`Error::Storage`](crate::error::Error::Storage); inspect the
    /// operation journal and [`get_stats`](Self::get_stats) for the IDs.
    ///
    /// # Errors
    ///
    /// Returns error on drain timeout or when writes failed permanently
    /// during the drain.
    pub async fn flush(&self, timeout: Duration) -> Result<()> {
        let baseline_failed = self.get_stats().await.total_failed;
        if !self.wait_until_empty(timeout).await {
            let depth = self.queue_size().await;
            return Err(Error::Storage(format!(
                "durable write queue did not drain within {timeout:?} ({depth} waiting)"
            )));
        }
        let failed_now = self.get_stats().await.total_failed;
        if failed_now > baseline_failed {
            return Err(Error::Storage(format!(
                "durable write queue drained with {} permanent failure(s); see operation journal",
                failed_now - baseline_failed
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_queue_config_defaults_are_bounded_and_retrying() {
        let config = WriteQueueConfig::default();

        assert_eq!(config.max_queue_size, DEFAULT_WRITE_QUEUE_SIZE);
        assert_eq!(config.batch_size, DEFAULT_WRITE_BATCH_SIZE);
        assert_eq!(config.poll_interval_ms, DEFAULT_WRITE_POLL_INTERVAL_MS);
        assert_eq!(config.max_retries, DEFAULT_WRITE_MAX_RETRIES);
        assert_eq!(
            config.retry_base_delay_ms,
            DEFAULT_WRITE_RETRY_BASE_DELAY_MS
        );
        assert_eq!(config.retry_max_delay_ms, DEFAULT_WRITE_RETRY_MAX_DELAY_MS);
        assert!(config.max_queue_size > 0);
        assert!(config.batch_size > 0);
    }
}
