//! Capacity model and background monitor for [`super::adaptive`].
//!
//! The semaphore is the single source of truth for effective capacity:
//! `current_max` always equals `available permits + checked out permits`, and
//! every resize adjusts the permit count and `current_max` together so reported
//! capacity, utilization and actual concurrency stay in agreement.

use super::adaptive::AdaptivePoolConfig;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tokio::sync::{Notify, Semaphore};
use tracing::{debug, info};

#[derive(Debug)]
pub(super) struct AdaptiveMetrics {
    pub(super) utilization_percent: AtomicU64,
    pub(super) active_connections: AtomicU32,
    pub(super) max_connections: AtomicU32,
    pub(super) scale_up_count: AtomicU32,
    pub(super) scale_down_count: AtomicU32,
    pub(super) avg_wait_time_us: AtomicU64,
    pub(super) total_acquired: AtomicU64,
    pub(super) total_released: AtomicU64,
    pub(super) wait_time_total_us: AtomicU64,
    pub(super) wait_count: AtomicU64,
}

impl Default for AdaptiveMetrics {
    fn default() -> Self {
        Self {
            utilization_percent: AtomicU64::new(0),
            active_connections: AtomicU32::new(0),
            max_connections: AtomicU32::new(0),
            scale_up_count: AtomicU32::new(0),
            scale_down_count: AtomicU32::new(0),
            avg_wait_time_us: AtomicU64::new(0),
            total_acquired: AtomicU64::new(0),
            total_released: AtomicU64::new(0),
            wait_time_total_us: AtomicU64::new(0),
            wait_count: AtomicU64::new(0),
        }
    }
}

/// Instants of the last successful resize.
///
/// Measured on the Tokio clock so cooldowns are monotonic in production and
/// testable under `tokio::time::pause` (a `std::time::Instant` reads real time
/// and would never elapse in a paused test).
#[derive(Debug, Default)]
struct ScaleState {
    last_scale_up: Option<tokio::time::Instant>,
    last_scale_down: Option<tokio::time::Instant>,
}

/// Shared capacity state, cloned into the monitor task.
pub(super) struct AdaptiveCore {
    config: Arc<AdaptivePoolConfig>,
    semaphore: Arc<Semaphore>,
    current_max: Arc<AtomicU32>,
    pub(super) metrics: Arc<AdaptiveMetrics>,
    scale_state: Mutex<ScaleState>,
}

impl AdaptiveCore {
    pub(super) fn new(config: Arc<AdaptivePoolConfig>) -> Self {
        let initial = config.min_connections;
        let metrics = Arc::new(AdaptiveMetrics::default());
        metrics.max_connections.store(initial, Ordering::Relaxed);

        Self {
            config,
            semaphore: Arc::new(Semaphore::new(initial as usize)),
            current_max: Arc::new(AtomicU32::new(initial)),
            metrics,
            scale_state: Mutex::new(ScaleState::default()),
        }
    }

    pub(super) fn semaphore(&self) -> &Arc<Semaphore> {
        &self.semaphore
    }

    pub(super) fn current_max(&self) -> &Arc<AtomicU32> {
        &self.current_max
    }

    pub(super) fn check_interval(&self) -> std::time::Duration {
        self.config.check_interval
    }

    /// Effective capacity: the number of permits currently issued by the pool.
    pub(super) fn effective_max(&self) -> u32 {
        self.current_max.load(Ordering::Acquire)
    }

    /// Store utilization using the same effective target scaling uses.
    pub(super) fn refresh_utilization(&self, active: u32) {
        let max = self.effective_max().max(1);
        let utilization = (f64::from(active) / f64::from(max)) * 100.0;
        self.metrics
            .utilization_percent
            .store(utilization as u64, Ordering::Relaxed);
    }

    /// Grow capacity when allowed, adding the matching semaphore permits.
    ///
    /// Returns `true` when the pool actually resized.
    pub(super) fn scale_up(&self) -> bool {
        let mut state = self.scale_state.lock();

        if let Some(last) = state.last_scale_up
            && last.elapsed() < self.config.scale_up_cooldown
        {
            return false;
        }

        let current = self.current_max.load(Ordering::Acquire);
        if current >= self.config.max_connections {
            return false;
        }

        let new_max = (current + self.config.scale_up_increment).min(self.config.max_connections);

        // Add permits first so a concurrent checkout cannot observe the new
        // target before its capacity exists.
        self.semaphore.add_permits((new_max - current) as usize);
        self.current_max.store(new_max, Ordering::Release);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        state.last_scale_up = Some(tokio::time::Instant::now());
        self.metrics.scale_up_count.fetch_add(1, Ordering::Relaxed);

        debug!("Scaling up: {} -> {} connections", current, new_max);
        true
    }

    /// Shrink capacity when allowed, reclaiming only idle permits.
    ///
    /// `Semaphore::forget_permits` removes at most the available permits, so
    /// checked-out connections are never stranded. When active work occupies
    /// the difference, the reduction is deferred to a later tick. Returns
    /// `true` when the pool actually resized.
    pub(super) fn scale_down(&self) -> bool {
        let mut state = self.scale_state.lock();

        if let Some(last) = state.last_scale_down
            && last.elapsed() < self.config.scale_down_cooldown
        {
            return false;
        }

        let current = self.current_max.load(Ordering::Acquire);
        if current <= self.config.min_connections {
            return false;
        }

        let desired = current
            .saturating_sub(self.config.scale_down_decrement)
            .max(self.config.min_connections);
        if desired >= current {
            return false;
        }

        let reclaimed = self.semaphore.forget_permits((current - desired) as usize);
        if reclaimed == 0 {
            // Every permit is checked out: defer the reduction instead of
            // stranding active work.
            debug!(
                "Deferring scale down from {} connections: all permits are active",
                current
            );
            return false;
        }

        let new_max = current - reclaimed as u32;
        self.current_max.store(new_max, Ordering::Release);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        state.last_scale_down = Some(tokio::time::Instant::now());
        self.metrics
            .scale_down_count
            .fetch_add(1, Ordering::Relaxed);

        debug!(
            "Scaling down: {} -> {} connections (reclaimed {} idle permits)",
            current, new_max, reclaimed
        );
        true
    }

    pub(super) fn check_and_scale(&self) -> bool {
        let active = self.metrics.active_connections.load(Ordering::Relaxed);
        self.refresh_utilization(active);

        let max = self.effective_max().max(1);
        let utilization = f64::from(active) / f64::from(max);

        if utilization >= self.config.scale_up_threshold {
            self.scale_up()
        } else if utilization <= self.config.scale_down_threshold {
            self.scale_down()
        } else {
            false
        }
    }
}

/// Handle to the background scaling monitor so shutdown can stop it.
pub(super) struct MonitorHandle {
    pub(super) shutdown: Arc<Notify>,
    pub(super) task: tokio::task::JoinHandle<()>,
}

/// Spawn the monitor loop that drives `check_and_scale` every `check_interval`.
pub(super) fn spawn_monitor(core: &Arc<AdaptiveCore>) -> MonitorHandle {
    let shutdown = Arc::new(Notify::new());
    let shutdown_signal = Arc::clone(&shutdown);
    let interval = core.check_interval();
    let core = Arc::clone(core);

    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                () = shutdown_signal.notified() => break,
                () = tokio::time::sleep(interval) => {
                    core.check_and_scale();
                }
            }
        }
        info!("Adaptive pool monitor stopped");
    });

    MonitorHandle { shutdown, task }
}
