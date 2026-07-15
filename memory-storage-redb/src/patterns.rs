//! Pattern storage operations for redb cache

use crate::{Error, PATTERNS_TABLE, RedbStorage};
use do_memory_core::{Pattern, Result, episode::PatternId};
use redb::{ReadableDatabase, ReadableTable};
use std::sync::Arc;
use tracing::debug;
use tracing::info;

use crate::episodes::RedbQuery;

impl RedbStorage {
    /// Store a pattern in cache
    pub async fn store_pattern(&self, pattern: &Pattern) -> Result<()> {
        debug!("Storing pattern in cache: {}", pattern.id());
        let db = Arc::clone(&self.db);
        let pattern_id = pattern.id().to_string();
        let pattern_bytes = postcard::to_allocvec(pattern)
            .map_err(|e| Error::Storage(format!("Failed to serialize pattern: {}", e)))?;

        tokio::task::spawn_blocking(move || {
            let write_txn = db
                .begin_write()
                .map_err(|e| Error::Storage(format!("Failed to begin write transaction: {}", e)))?;

            {
                let mut table = write_txn
                    .open_table(PATTERNS_TABLE)
                    .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

                table
                    .insert(pattern_id.as_str(), pattern_bytes.as_slice())
                    .map_err(|e| Error::Storage(format!("Failed to insert pattern: {}", e)))?;
            }

            write_txn
                .commit()
                .map_err(|e| Error::Storage(format!("Failed to commit transaction: {}", e)))?;

            Ok::<(), Error>(())
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))??;

        info!("Successfully cached pattern: {}", pattern.id());
        Ok(())
    }

    /// Retrieve a pattern from cache
    pub async fn get_pattern(&self, pattern_id: PatternId) -> Result<Option<Pattern>> {
        debug!("Retrieving pattern from cache: {}", pattern_id);
        let db = Arc::clone(&self.db);
        let pattern_id_str = pattern_id.to_string();

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            match table
                .get(pattern_id_str.as_str())
                .map_err(|e| Error::Storage(format!("Failed to get pattern: {}", e)))?
            {
                Some(bytes_guard) => {
                    let _bytes = bytes_guard.value();
                    let pattern: Pattern =
                        postcard::from_bytes(bytes_guard.value()).map_err(|e| {
                            Error::Storage(format!("Failed to deserialize pattern: {}", e))
                        })?;
                    Ok(Some(pattern))
                }
                None => Ok(None),
            }
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }

    /// Get all patterns from cache (with optional limit)
    pub async fn get_all_patterns(&self, query: &RedbQuery) -> Result<Vec<Pattern>> {
        debug!("Retrieving all patterns from cache");
        let db = Arc::clone(&self.db);
        let limit = query.limit;

        tokio::task::spawn_blocking(move || {
            let read_txn = db
                .begin_read()
                .map_err(|e| Error::Storage(format!("Failed to begin read transaction: {}", e)))?;

            let table = read_txn
                .open_table(PATTERNS_TABLE)
                .map_err(|e| Error::Storage(format!("Failed to open patterns table: {}", e)))?;

            let mut patterns = Vec::new();
            let iter = table
                .iter()
                .map_err(|e| Error::Storage(format!("Failed to iterate patterns: {}", e)))?;

            for (count, result) in iter.enumerate() {
                if let Some(max) = limit {
                    if count >= max {
                        break;
                    }
                }

                let (_, bytes_guard) = result
                    .map_err(|e| Error::Storage(format!("Failed to read pattern entry: {}", e)))?;

                // Skip unreadable rows (e.g. pre-v4 internally-tagged postcard) so one
                // stale entry cannot fail the entire list (issue #831).
                match postcard::from_bytes::<Pattern>(bytes_guard.value()) {
                    Ok(pattern) => patterns.push(pattern),
                    Err(e) => {
                        debug!("Skipping undecodable pattern cache entry: {}", e);
                    }
                }
            }

            Ok(patterns)
        })
        .await
        .map_err(|e| Error::Storage(format!("Task join error: {}", e)))?
    }
}

#[cfg(test)]
mod pattern_persistence_tests {
    use super::*;
    use do_memory_core::StorageBackend;
    use do_memory_core::patterns::PatternEffectiveness;
    use do_memory_core::types::{ComplexityLevel, TaskContext};
    use uuid::Uuid;

    fn sample_pattern() -> Pattern {
        let context = TaskContext {
            language: Some("rust".to_string()),
            framework: Some("tokio".to_string()),
            complexity: ComplexityLevel::Moderate,
            domain: "cli".to_string(),
            tags: vec!["regression".to_string()],
        };
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["cargo".to_string(), "test".to_string()],
            context,
            success_rate: 1.0,
            avg_latency: chrono::Duration::milliseconds(10),
            occurrence_count: 1,
            effectiveness: PatternEffectiveness::new(),
        }
    }

    /// Issue #831: patterns must survive postcard serialize + get_all_patterns.
    #[tokio::test]
    async fn store_and_list_patterns_across_trait() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("patterns.redb");
        let storage = RedbStorage::new(&path).await.expect("open redb");

        let pattern = sample_pattern();
        let id = pattern.id();
        storage.store_pattern(&pattern).await.expect("store");

        // Via concrete API
        let listed = storage
            .get_all_patterns(&RedbQuery::default())
            .await
            .expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id(), id);

        // Via StorageBackend trait (what memory-core uses for CLI list)
        let via_trait = StorageBackend::get_all_patterns(&storage)
            .await
            .expect("trait list");
        assert_eq!(via_trait.len(), 1);
        assert_eq!(via_trait[0].id(), id);
    }
}
