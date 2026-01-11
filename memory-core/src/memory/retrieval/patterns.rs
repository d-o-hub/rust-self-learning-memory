//! Pattern retrieval implementation

use crate::extraction::{deduplicate_patterns, rank_patterns};
use crate::pattern::Pattern;
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
        let mut patterns = self.patterns_fallback.write().await;

        debug!(
            total_patterns = patterns.len(),
            limit = limit,
            "Retrieving relevant patterns"
        );

        let all_patterns: Vec<Pattern> = patterns.values().cloned().collect();

        // Rank patterns by relevance and quality (includes effectiveness scoring)
        let mut ranked = rank_patterns(all_patterns, context);

        // Deduplicate
        ranked = deduplicate_patterns(ranked);

        // Record retrieval for effectiveness tracking
        for pattern in &mut ranked.iter_mut().take(limit) {
            pattern.record_retrieval();
            // Update the stored pattern with retrieval count
            patterns.insert(pattern.id(), pattern.clone());
        }

        // Limit results
        ranked.truncate(limit);

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
