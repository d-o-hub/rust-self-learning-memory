//! Sync watermark support for resilient storage.

use async_trait::async_trait;
use do_memory_core::{Episode, Result, SyncWatermarkBackend};
use uuid::Uuid;

use crate::resilient::ResilientStorage;

#[async_trait]
impl SyncWatermarkBackend for ResilientStorage {
    async fn query_episodes_modified_since(
        &self,
        since: chrono::DateTime<chrono::Utc>,
        cursor: Option<(chrono::DateTime<chrono::Utc>, Uuid)>,
        limit: Option<usize>,
    ) -> Result<Vec<(Episode, chrono::DateTime<chrono::Utc>)>> {
        self.circuit_call(move |s| async move {
            s.query_episodes_modified_since(since, cursor, limit).await
        })
        .await
    }

    async fn load_sync_watermark(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        self.circuit_call(|s| async move { s.load_sync_watermark().await })
            .await
    }

    async fn save_sync_watermark(&self, watermark: chrono::DateTime<chrono::Utc>) -> Result<()> {
        self.circuit_call(move |s| async move { s.save_sync_watermark(watermark).await })
            .await
    }
}
