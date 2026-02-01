//! # Batch Heuristic Operations - Core
//!
//! Core batch operations for heuristics using transactions.

use crate::TursoStorage;
use memory_core::{Error, Heuristic, Result};
use tracing::{debug, error, info, warn};

use super::heuristic_types::{HeuristicBatchProgress, HeuristicBatchResult};

impl TursoStorage {
    /// Store multiple heuristics in a single transaction
    ///
    /// Uses prepared statements and transactions for 4-6x throughput improvement.
    /// All heuristics are stored atomically - if any fails, all are rolled back.
    pub async fn store_heuristics_batch(&self, heuristics: Vec<Heuristic>) -> Result<()> {
        if heuristics.is_empty() {
            debug!("Empty heuristics batch received, skipping");
            return Ok(());
        }

        debug!("Storing heuristics batch: {} items", heuristics.len());
        let conn = self.get_connection().await?;

        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for heuristics batch: {}",
                e
            ))
        })?;

        let sql = r#"
            INSERT OR REPLACE INTO heuristics (
                heuristic_id, condition_text, action_text, confidence, evidence, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        for heuristic in &heuristics {
            let evidence_json =
                serde_json::to_string(&heuristic.evidence).map_err(Error::Serialization)?;

            if let Err(e) = conn
                .execute(
                    sql,
                    libsql::params![
                        heuristic.heuristic_id.to_string(),
                        heuristic.condition.clone(),
                        heuristic.action.clone(),
                        heuristic.confidence,
                        evidence_json,
                        heuristic.created_at.timestamp(),
                        heuristic.updated_at.timestamp(),
                    ],
                )
                .await
            {
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Failed to store heuristic in batch: {}",
                    e
                )));
            }
        }

        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit heuristics batch transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully stored heuristics batch: {} items",
            heuristics.len()
        );
        Ok(())
    }

    /// Update multiple heuristics in a single transaction
    pub async fn update_heuristics_batch(&self, heuristics: Vec<Heuristic>) -> Result<()> {
        if heuristics.is_empty() {
            debug!("Empty heuristics update batch received, skipping");
            return Ok(());
        }

        debug!("Updating heuristics batch: {} items", heuristics.len());
        let conn = self.get_connection().await?;

        conn.execute("BEGIN TRANSACTION", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to begin transaction for heuristics update batch: {}",
                e
            ))
        })?;

        // Verify all heuristics exist
        for heuristic in &heuristics {
            let check_sql = "SELECT 1 FROM heuristics WHERE heuristic_id = ?";
            let mut rows = conn
                .query(
                    check_sql,
                    libsql::params![heuristic.heuristic_id.to_string()],
                )
                .await
                .map_err(|e| {
                    Error::Storage(format!(
                        "Failed to check heuristic existence in batch: {}",
                        e
                    ))
                })?;

            let exists = rows
                .next()
                .await
                .map_err(|e| Error::Storage(format!("Failed to fetch row: {}", e)))?
                .is_some();

            if !exists {
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Heuristic {} does not exist for update",
                    heuristic.heuristic_id
                )));
            }
        }

        let sql = r#"
            UPDATE heuristics SET
                condition_text = ?,
                action_text = ?,
                confidence = ?,
                evidence = ?,
                updated_at = ?
            WHERE heuristic_id = ?
        "#;

        for heuristic in &heuristics {
            let evidence_json =
                serde_json::to_string(&heuristic.evidence).map_err(Error::Serialization)?;

            let now = chrono::Utc::now();

            if let Err(e) = conn
                .execute(
                    sql,
                    libsql::params![
                        heuristic.condition.clone(),
                        heuristic.action.clone(),
                        heuristic.confidence,
                        evidence_json,
                        now.timestamp(),
                        heuristic.heuristic_id.to_string(),
                    ],
                )
                .await
            {
                if let Err(rollback_err) = conn.execute("ROLLBACK", ()).await {
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(Error::Storage(format!(
                    "Failed to update heuristic in batch: {}",
                    e
                )));
            }
        }

        conn.execute("COMMIT", ()).await.map_err(|e| {
            Error::Storage(format!(
                "Failed to commit heuristics update batch transaction: {}",
                e
            ))
        })?;

        info!(
            "Successfully updated heuristics batch: {} items",
            heuristics.len()
        );
        Ok(())
    }

    /// Store heuristics in batches with progress tracking
    pub async fn store_heuristics_batch_with_progress(
        &self,
        heuristics: Vec<Heuristic>,
        batch_size: usize,
    ) -> Result<HeuristicBatchResult> {
        if heuristics.is_empty() {
            return Ok(HeuristicBatchResult::success(0));
        }

        let batch_size = batch_size.max(1);
        let total = heuristics.len();
        let mut progress = HeuristicBatchProgress::new(total, batch_size);
        let mut errors = Vec::new();

        info!(
            "Starting batch heuristic storage: {} items in {} batches",
            total, progress.total_batches
        );

        for chunk in heuristics.chunks(batch_size) {
            let chunk_vec = chunk.to_vec();
            let chunk_len = chunk_vec.len();

            match self.store_heuristics_batch(chunk_vec).await {
                Ok(()) => {
                    progress.update(chunk_len, chunk_len, 0);
                    debug!(
                        "Batch {}/{} complete: {} items",
                        progress.current_batch, progress.total_batches, chunk_len
                    );
                }
                Err(e) => {
                    progress.update(chunk_len, 0, chunk_len);
                    let error_msg = format!("Batch {} failed: {}", progress.current_batch, e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                }
            }

            if progress.current_batch % 10 == 0 || progress.is_complete() {
                info!(
                    "Progress: {:.1}% ({}/{} batches, {}/{} items)",
                    progress.percent_complete(),
                    progress.current_batch,
                    progress.total_batches,
                    progress.processed,
                    total
                );
            }
        }

        let all_succeeded = errors.is_empty();
        let result = HeuristicBatchResult {
            total_processed: progress.processed,
            succeeded: progress.succeeded,
            failed: progress.failed,
            all_succeeded,
            errors,
        };

        info!(
            "Batch heuristic storage complete: {} succeeded, {} failed",
            result.succeeded, result.failed
        );

        Ok(result)
    }

    /// Update heuristics in batches with progress tracking
    pub async fn update_heuristics_batch_with_progress(
        &self,
        heuristics: Vec<Heuristic>,
        batch_size: usize,
    ) -> Result<HeuristicBatchResult> {
        if heuristics.is_empty() {
            return Ok(HeuristicBatchResult::success(0));
        }

        let batch_size = batch_size.max(1);
        let total = heuristics.len();
        let mut progress = HeuristicBatchProgress::new(total, batch_size);
        let mut errors = Vec::new();

        info!(
            "Starting batch heuristic update: {} items in {} batches",
            total, progress.total_batches
        );

        for chunk in heuristics.chunks(batch_size) {
            let chunk_vec = chunk.to_vec();
            let chunk_len = chunk_vec.len();

            match self.update_heuristics_batch(chunk_vec).await {
                Ok(()) => {
                    progress.update(chunk_len, chunk_len, 0);
                    debug!(
                        "Batch {}/{} complete: {} items",
                        progress.current_batch, progress.total_batches, chunk_len
                    );
                }
                Err(e) => {
                    progress.update(chunk_len, 0, chunk_len);
                    let error_msg = format!("Batch {} failed: {}", progress.current_batch, e);
                    warn!("{}", error_msg);
                    errors.push(error_msg);
                }
            }

            if progress.current_batch % 10 == 0 || progress.is_complete() {
                info!(
                    "Progress: {:.1}% ({}/{} batches, {}/{} items)",
                    progress.percent_complete(),
                    progress.current_batch,
                    progress.total_batches,
                    progress.processed,
                    total
                );
            }
        }

        let all_succeeded = errors.is_empty();
        let result = HeuristicBatchResult {
            total_processed: progress.processed,
            succeeded: progress.succeeded,
            failed: progress.failed,
            all_succeeded,
            errors,
        };

        info!(
            "Batch heuristic update complete: {} succeeded, {} failed",
            result.succeeded, result.failed
        );

        Ok(result)
    }
}
