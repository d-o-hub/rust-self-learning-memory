//! Persistence manager for automatic cache persistence

use std::sync::Arc;

use parking_lot::RwLock;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use super::{CachePersistence, CacheSnapshot, PersistenceConfig, PersistenceStats};

/// Manager for automatic cache persistence
///
/// Handles periodic saves, shutdown persistence, and recovery.
pub struct PersistenceManager {
    config: PersistenceConfig,
    persistence: CachePersistence,
    last_snapshot: Arc<RwLock<Option<CacheSnapshot>>>,
    background_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    shutdown_flag: Arc<RwLock<bool>>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub fn new(config: PersistenceConfig) -> Self {
        let persistence = CachePersistence::new(config.clone());

        Self {
            config,
            persistence,
            last_snapshot: Arc::new(RwLock::new(None)),
            background_task: Arc::new(RwLock::new(None)),
            shutdown_flag: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with default configuration
    pub fn with_default_config() -> Self {
        Self::new(PersistenceConfig::default())
    }

    /// Start background persistence task
    ///
    /// This starts a task that periodically saves the cache based on
    /// the configured strategy and interval.
    pub fn start_background_task(
        &self,
        snapshot_provider: Arc<dyn Fn() -> Option<CacheSnapshot> + Send + Sync>,
    ) {
        if !self.config.enabled {
            debug!("Persistence disabled, not starting background task");
            return;
        }

        if *self.shutdown_flag.read() {
            warn!("Cannot start background task: shutdown in progress");
            return;
        }

        let interval_duration = self.config.save_interval;
        let persistence = CachePersistence::new(self.config.clone());
        let shutdown_flag: Arc<parking_lot::RwLock<bool>> = Arc::clone(&self.shutdown_flag);
        let last_snapshot: Arc<parking_lot::RwLock<Option<CacheSnapshot>>> =
            Arc::clone(&self.last_snapshot);

        let handle = tokio::spawn(async move {
            let mut ticker = interval(interval_duration);

            loop {
                ticker.tick().await;

                if *shutdown_flag.read() {
                    debug!("Background persistence task shutting down");
                    break;
                }

                // Get snapshot from provider
                if let Some(snapshot) = snapshot_provider() {
                    if persistence.should_save(snapshot.len()) {
                        match persistence.save_snapshot(&snapshot, None) {
                            Ok(count) => {
                                debug!("Background save completed: {} entries", count);
                                let mut last = last_snapshot.write();
                                *last = Some(snapshot);
                            }
                            Err(e) => {
                                error!("Background save failed: {}", e);
                            }
                        }
                    }
                }
            }
        });

        let mut task = self.background_task.write();
        *task = Some(handle);

        info!(
            "Started background persistence task with interval {:?}",
            interval_duration
        );
    }

    /// Stop the background persistence task
    pub fn stop_background_task(&self) {
        // Set shutdown flag
        {
            let mut flag = self.shutdown_flag.write();
            *flag = true;
        }

        // Abort background task
        let mut task = self.background_task.write();
        if let Some(handle) = task.take() {
            handle.abort();
            info!("Background persistence task stopped");
        }
    }

    /// Perform graceful shutdown with final cache save
    ///
    /// # Arguments
    ///
    /// * `final_snapshot` - The final cache snapshot to save
    pub fn shutdown(&self, final_snapshot: Option<CacheSnapshot>) {
        info!("Starting persistence manager shutdown");

        // Stop background task
        self.stop_background_task();

        // Save final snapshot if provided and enabled
        if let Some(snapshot) = final_snapshot {
            if self.config.enabled {
                info!("Saving final cache snapshot ({} entries)", snapshot.len());
                match self.persistence.save_snapshot(&snapshot, None) {
                    Ok(count) => {
                        info!("Final cache snapshot saved: {} entries", count);
                    }
                    Err(e) => {
                        error!("Failed to save final cache snapshot: {}", e);
                    }
                }
            }
        }

        info!("Persistence manager shutdown complete");
    }

    /// Recover cache from persisted snapshot
    ///
    /// # Returns
    ///
    /// The recovered cache snapshot, or None if no snapshot exists
    pub fn recover(&self) -> crate::Result<Option<CacheSnapshot>> {
        if !self.config.enabled {
            debug!("Persistence disabled, skipping recovery");
            return Ok(None);
        }

        info!("Attempting to recover cache from persistence");

        match self.persistence.load_snapshot(None) {
            Ok(Some(snapshot)) => {
                info!(
                    "Cache recovered: {} entries from snapshot created at {}",
                    snapshot.len(),
                    snapshot.created_at
                );

                // Update last snapshot
                {
                    let mut last = self.last_snapshot.write();
                    *last = Some(snapshot.clone());
                }

                Ok(Some(snapshot))
            }
            Ok(None) => {
                info!("No cache snapshot found for recovery");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to recover cache: {}", e);
                Err(e)
            }
        }
    }

    /// Check if a recovery snapshot exists
    pub fn has_recovery_snapshot(&self) -> bool {
        self.config.enabled && self.config.persistence_path.exists()
    }

    /// Get the persistence configuration
    pub fn config(&self) -> &PersistenceConfig {
        &self.config
    }

    /// Get persistence statistics
    pub fn stats(&self) -> PersistenceStats {
        self.persistence.stats()
    }

    /// Get the last saved snapshot
    pub fn last_snapshot(&self) -> Option<CacheSnapshot> {
        self.last_snapshot.read().clone()
    }

    /// Force an immediate save
    pub fn force_save(&self, snapshot: &CacheSnapshot) -> crate::Result<usize> {
        let result = self.persistence.save_snapshot(snapshot, None);

        if result.is_ok() {
            let mut last = self.last_snapshot.write();
            *last = Some(snapshot.clone());
        }

        result
    }

    /// Delete persisted snapshot
    pub fn delete_persisted(&self) -> crate::Result<bool> {
        self.persistence.delete_snapshot(None)
    }
}

impl Default for PersistenceManager {
    fn default() -> Self {
        Self::new(PersistenceConfig::default())
    }
}

impl Drop for PersistenceManager {
    fn drop(&mut self) {
        // Ensure background task is stopped
        self.stop_background_task();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_snapshot() -> CacheSnapshot {
        CacheSnapshot {
            version: 1,
            created_at: 1234567890,
            entries: vec![],
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_manager_creation() {
        let manager = PersistenceManager::default();
        assert!(manager.config().enabled);
        assert!(manager.last_snapshot().is_none());
    }

    #[test]
    fn test_force_save() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            persistence_path: temp_dir.path().join("cache.snapshot"),
            ..Default::default()
        };

        let manager = PersistenceManager::new(config);
        let snapshot = create_test_snapshot();

        let saved = manager.force_save(&snapshot).unwrap();
        assert_eq!(saved, 0);
        assert!(manager.last_snapshot().is_some());
    }

    #[test]
    fn test_delete_persisted() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            persistence_path: temp_dir.path().join("cache.snapshot"),
            ..Default::default()
        };

        let manager = PersistenceManager::new(config);
        let snapshot = create_test_snapshot();

        // Save then delete
        manager.force_save(&snapshot).unwrap();
        assert!(manager.has_recovery_snapshot());

        let deleted = manager.delete_persisted().unwrap();
        assert!(deleted);
        assert!(!manager.has_recovery_snapshot());
    }

    #[test]
    fn test_disabled_manager() {
        let config = PersistenceConfig::disabled();
        let manager = PersistenceManager::new(config);

        assert!(!manager.config().enabled);
        assert!(manager.recover().unwrap().is_none());
    }
}
