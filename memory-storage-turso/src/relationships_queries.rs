//! Weighted neighbor queries for episode relationships in Turso.
//!
//! Extracted from the `relationships` module to keep the parent module under
//! the enforced 500-line ceiling.

use crate::{Result, TursoStorage};
use uuid::Uuid;

impl TursoStorage {
    /// Get weighted neighbors (episodes and patterns) for an episode
    pub async fn get_weighted_neighbors(&self, episode_id: Uuid) -> Result<Vec<(Uuid, f32, bool)>> {
        self.with_connection(async |conn| {
            let mut neighbors = Vec::new();

            // 1. Get episode neighbors
            const SQL_EP: &str =
                "SELECT to_episode_id, weight FROM episode_relationships WHERE from_episode_id = ?";
            let stmt_ep = conn.prepare(SQL_EP).await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
            })?;
            let mut rows_ep = stmt_ep
                .query(libsql::params![episode_id.to_string()])
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
                })?;
            while let Some(row) = rows_ep.next().await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e))
            })? {
                let id_str: String = row.get(0).unwrap();
                let weight: Option<f64> = row.get(1).ok();
                let id = Uuid::parse_str(&id_str).unwrap();
                neighbors.push((id, weight.map(|w| w as f32).unwrap_or(1.0), false));
            }

            // 2. Get pattern neighbors
            const SQL_PT: &str =
                "SELECT pattern_id, weight FROM episode_pattern_relationships WHERE episode_id = ?";
            let stmt_pt = conn.prepare(SQL_PT).await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to prepare query: {}", e))
            })?;
            let mut rows_pt = stmt_pt
                .query(libsql::params![episode_id.to_string()])
                .await
                .map_err(|e| {
                    do_memory_core::Error::Storage(format!("Failed to execute query: {}", e))
                })?;
            while let Some(row) = rows_pt.next().await.map_err(|e| {
                do_memory_core::Error::Storage(format!("Failed to fetch row: {}", e))
            })? {
                let id_str: String = row.get(0).unwrap();
                let weight: Option<f64> = row.get(1).ok();
                let id = Uuid::parse_str(&id_str).unwrap();
                neighbors.push((id, weight.map(|w| w as f32).unwrap_or(1.0), true));
            }

            Ok(neighbors)
        })
        .await
    }
}
