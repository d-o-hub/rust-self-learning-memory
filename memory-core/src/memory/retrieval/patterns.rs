//! Pattern retrieval implementation

use crate::extraction::{deduplicate_patterns, rank_patterns};
use crate::patterns::Pattern;
use crate::types::TaskContext;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Retrieve relevant patterns for a task context
    ///
    /// Finds patterns that match the given context and ranks them
    /// by relevance and success rate.
    ///
    /// # Arguments
    ///
    /// * `context` - Task context to match against
    /// * `limit` - Maximum number of patterns to return
    ///
    /// # Returns
    ///
    /// Vector of relevant patterns, sorted by relevance and quality
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_patterns(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Vec<Pattern> {
        // Hydrate from all storage backends (lazy loading) so patterns created
        // in a previous process are retrievable in a fresh CLI invocation.
        let all_patterns = match self.get_all_patterns().await {
            Ok(patterns) => patterns,
            Err(e) => {
                debug!("Failed to load patterns from storage: {}", e);
                Vec::new()
            }
        };

        debug!(
            total_patterns = all_patterns.len(),
            limit = limit,
            "Retrieving relevant patterns"
        );

        // Rank patterns by relevance and quality (includes effectiveness scoring)
        let mut ranked = rank_patterns(all_patterns, context);

        // Deduplicate
        ranked = deduplicate_patterns(ranked);

        // Limit results
        ranked.truncate(limit);

        // Record retrieval for in-memory effectiveness tracking (best-effort).
        // Write-back is limited to the process-local fallback so we never hold
        // a lock across durable storage I/O.
        {
            let mut patterns = self.patterns_fallback.write().await;
            for pattern in &mut ranked {
                pattern.record_retrieval();
                patterns.insert(pattern.id(), pattern.clone());
            }
        }

        info!(
            retrieved_count = ranked.len(),
            "Retrieved relevant patterns with effectiveness tracking"
        );

        ranked
    }

    /// Retrieve a single pattern by ID
    ///
    /// # Arguments
    ///
    /// * `pattern_id` - The unique ID of the pattern to retrieve
    ///
    /// # Returns
    ///
    /// The pattern if found, or None if not found
    ///
    /// # Errors
    ///
    /// Returns error if storage operation fails
    #[instrument(skip(self))]
    pub async fn get_pattern(&self, pattern_id: Uuid) -> crate::Result<Option<Pattern>> {
        // Try storage backends first
        if let Some(storage) = &self.turso_storage {
            match storage.get_pattern(pattern_id).await {
                Ok(pattern) => return Ok(pattern),
                Err(e) => {
                    debug!("Failed to get pattern from Turso storage: {}", e);
                    // Fall back to cache or in-memory
                }
            }
        }

        if let Some(cache) = &self.cache_storage {
            match cache.get_pattern(pattern_id).await {
                Ok(pattern) => return Ok(pattern),
                Err(e) => {
                    debug!("Failed to get pattern from cache storage: {}", e);
                    // Fall back to in-memory
                }
            }
        }

        // Fall back to in-memory storage
        let patterns = self.patterns_fallback.read().await;
        Ok(patterns.get(&pattern_id).cloned())
    }
}
