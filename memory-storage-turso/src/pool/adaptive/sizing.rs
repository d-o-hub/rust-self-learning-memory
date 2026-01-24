//! Adaptive connection pool sizing logic
//!
//! This module contains the sizing and scaling logic for the adaptive pool.

use super::AdaptivePoolConfig;
use crate::pool::adaptive::AdaptiveMetrics;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Sizing logic for the adaptive connection pool
pub(super) struct PoolSizer {
    config: Arc<AdaptivePoolConfig>,
    current_max: Arc<AtomicU32>,
    metrics: Arc<AdaptiveMetrics>,
}

impl PoolSizer {
    /// Create a new pool sizer
    pub(super) fn new(
        config: Arc<AdaptivePoolConfig>,
        current_max: Arc<AtomicU32>,
        metrics: Arc<AdaptiveMetrics>,
    ) -> Self {
        Self {
            config,
            current_max,
            metrics,
        }
    }

    /// Scale up the pool if needed
    pub(super) async fn scale_up(&self) {
        let now = Instant::now();
        let last_up = self.metrics.last_scale_up.load(Ordering::Relaxed);

        // Use duration since a fixed epoch
        let epoch_duration = Duration::from_nanos(last_up);
        let last_up_time = Instant::now() - epoch_duration;

        if now.duration_since(last_up_time) < self.config.scale_up_cooldown {
            return;
        }

        let current_max = self.current_max.load(Ordering::Relaxed);

        if current_max >= self.config.max_connections {
            return;
        }

        let new_max =
            (current_max + self.config.scale_up_increment).min(self.config.max_connections);

        tracing::info!("Scaling up: {} -> {} connections", current_max, new_max);

        self.current_max.store(new_max, Ordering::Relaxed);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        self.metrics
            .last_scale_up
            .store(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
        self.metrics.scale_up_count.fetch_add(1, Ordering::Relaxed);

        tracing::debug!("Scale up complete: {} connections", new_max);
    }

    /// Scale down the pool if needed
    pub(super) async fn scale_down(&self) {
        let now = Instant::now();
        let last_down = self.metrics.last_scale_down.load(Ordering::Relaxed);

        let epoch_duration = Duration::from_nanos(last_down);
        let last_down_time = Instant::now() - epoch_duration;

        if now.duration_since(last_down_time) < self.config.scale_down_cooldown {
            return;
        }

        let current_max = self.current_max.load(Ordering::Relaxed);
        let active = self.metrics.active_connections.load(Ordering::Relaxed);

        let min_allowed = active.max(self.config.min_connections);
        let new_max =
            (current_max.saturating_sub(self.config.scale_down_decrement)).max(min_allowed);

        if new_max >= current_max {
            return;
        }

        tracing::info!(
            "Scaling down: {} -> {} connections (active: {})",
            current_max,
            new_max,
            active
        );

        self.current_max.store(new_max, Ordering::Relaxed);
        self.metrics
            .max_connections
            .store(new_max, Ordering::Relaxed);
        self.metrics
            .last_scale_down
            .store(now.elapsed().as_nanos() as u64, Ordering::Relaxed);
        self.metrics
            .scale_down_count
            .fetch_add(1, Ordering::Relaxed);

        tracing::debug!("Scale down complete: {} connections", new_max);
    }

    /// Check and perform scaling if needed
    pub(super) async fn check_and_scale(&self) {
        let active = self.metrics.active_connections.load(Ordering::Relaxed);
        let max = self.current_max.load(Ordering::Relaxed);
        let utilization = active as f64 / max as f64;

        if utilization >= self.config.scale_up_threshold {
            self.scale_up().await;
        } else if utilization <= self.config.scale_down_threshold {
            self.scale_down().await;
        }
    }
}
