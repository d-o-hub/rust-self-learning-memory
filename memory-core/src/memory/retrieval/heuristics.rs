//! Heuristic retrieval implementation

use crate::types::TaskContext;
use tracing::{debug, info, instrument};

use super::super::SelfLearningMemory;

impl SelfLearningMemory {
    /// Retrieve relevant heuristics for a given task context
    ///
    /// Finds heuristics that match the given context and ranks them
    /// by confidence weighted by relevance score.
    ///
    /// # Algorithm
    ///
    /// 1. Query heuristics from in-memory fallback
    /// 2. Calculate relevance score based on context similarity:
    ///    - Domain exact match: +1.0
    ///    - Language exact match: +0.8
    ///    - Framework match: +0.5
    ///    - Tag overlap: +0.3 per matching tag
    /// 3. Rank by: confidence × `relevance_score`
    /// 4. Return top N heuristics sorted by score DESC
    ///
    /// # Arguments
    ///
    /// * `context` - Task context to match against
    /// * `limit` - Maximum number of heuristics to return
    ///
    /// # Returns
    ///
    /// Vector of relevant heuristics, sorted by relevance and confidence
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel};
    ///
    /// # async fn example() {
    /// let memory = SelfLearningMemory::new();
    ///
    /// let context = TaskContext {
    ///     language: Some("rust".to_string()),
    ///     framework: Some("tokio".to_string()),
    ///     complexity: ComplexityLevel::Moderate,
    ///     domain: "async-processing".to_string(),
    ///     tags: vec!["concurrency".to_string()],
    /// };
    ///
    /// // Retrieve top 5 relevant heuristics
    /// let heuristics = memory.retrieve_relevant_heuristics(&context, 5).await;
    ///
    /// for heuristic in heuristics {
    ///     println!("Condition: {}", heuristic.condition);
    ///     println!("Action: {}", heuristic.action);
    ///     println!("Confidence: {}", heuristic.confidence);
    /// }
    /// # }
    /// ```
    #[instrument(skip(self))]
    pub async fn retrieve_relevant_heuristics(
        &self,
        context: &TaskContext,
        limit: usize,
    ) -> Vec<crate::pattern::Heuristic> {
        let heuristics = self.heuristics_fallback.read().await;

        debug!(
            total_heuristics = heuristics.len(),
            limit = limit,
            "Retrieving relevant heuristics"
        );

        // Calculate weighted score for each heuristic
        let mut scored_heuristics: Vec<_> = heuristics
            .values()
            .map(|h| {
                let relevance = self.calculate_heuristic_relevance(h, context);
                let weighted_score = h.confidence * relevance;
                (h.clone(), weighted_score)
            })
            .filter(|(_, score)| *score > 0.0) // Only include relevant heuristics
            .collect();

        // Sort by weighted score (descending)
        scored_heuristics
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        let result: Vec<_> = scored_heuristics
            .into_iter()
            .take(limit)
            .map(|(h, _)| h)
            .collect();

        info!(
            retrieved_count = result.len(),
            "Retrieved relevant heuristics"
        );

        result
    }
}
