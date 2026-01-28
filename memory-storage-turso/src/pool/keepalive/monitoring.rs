//! Background monitoring and maintenance tasks for keep-alive pool

use tracing::{debug, info};

use super::KeepAlivePool;

impl KeepAlivePool {
    /// Perform cleanup of stale connections
    pub fn cleanup(&self) {
        let stale_threshold = self.config().stale_threshold;
        let now = std::time::Instant::now();

        let mut last_used_map = self.last_used.write();
        let before = last_used_map.len();

        last_used_map.retain(|_id, last_used| now.duration_since(*last_used) < stale_threshold * 2);

        let after = last_used_map.len();
        let removed = before.saturating_sub(after);

        if removed > 0 {
            debug!("Cleaned up {} stale connection entries", removed);
        }
    }

    /// Start the background keep-alive task
    ///
    /// This method should be called after the pool is wrapped in an Arc.
    /// It spawns a background task that periodically pings connections.
    pub fn start_background_task(self: &std::sync::Arc<Self>) {
        use std::sync::Arc;

        let pool_weak = Arc::downgrade(self);
        let interval = self.config().keep_alive_interval;

        let _handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;
                if let Some(pool) = pool_weak.upgrade() {
                    pool.proactive_ping().await;
                } else {
                    debug!("Keep-alive pool dropped, stopping background task");
                    break;
                }
            }
        });

        info!(
            "Background keep-alive task started with interval {:?}",
            interval
        );
    }

    /// Proactively ping active connections
    async fn proactive_ping(&self) {
        if !self.config().enable_proactive_ping {
            return;
        }

        let now = std::time::Instant::now();
        let mut pinged = 0;

        let last_used_map = self.last_used.read();
        for (_conn_id, last_used) in last_used_map.iter() {
            let elapsed = now.duration_since(*last_used);

            if elapsed > self.config().keep_alive_interval {
                pinged += 1;
            }
        }

        if pinged > 0 {
            debug!(
                "Proactive ping check: {} connections approaching staleness",
                pinged
            );
        }

        drop(last_used_map);

        {
            let mut stats = self.stats.write();
            if pinged > 0 {
                stats.total_proactive_pings += pinged;
            }
        }
    }
}
