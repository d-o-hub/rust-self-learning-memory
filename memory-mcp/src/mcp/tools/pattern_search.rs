//! Pattern search MCP tool implementation

use memory_core::memory::{PatternSearchResult, SearchConfig};
use memory_core::{SelfLearningMemory, TaskContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input parameters for search_patterns tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPatternsInput {
    /// Natural language query describing what pattern to search for
    pub query: String,
    /// Domain to search in (e.g., "web-api", "cli")
    pub domain: String,
    /// Optional tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
    /// Maximum number of results (default: 5)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Minimum relevance score (0.0 to 1.0, default: 0.3)
    #[serde(default = "default_min_relevance")]
    pub min_relevance: f32,
    /// Whether to filter by domain (default: false)
    #[serde(default)]
    pub filter_by_domain: bool,
}

fn default_limit() -> usize {
    5
}

fn default_min_relevance() -> f32 {
    0.3
}

/// Output from search_patterns tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPatternsOutput {
    /// Found patterns with relevance scores
    pub results: Vec<PatternResult>,
    /// Total patterns searched
    pub total_searched: usize,
    /// Query that was executed
    pub query: String,
}

/// Individual pattern result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResult {
    /// Pattern ID
    pub id: String,
    /// Pattern type
    pub pattern_type: String,
    /// Pattern description/summary
    pub description: String,
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f32,
    /// Score breakdown
    pub score_breakdown: ScoreBreakdownResult,
    /// Success rate of pattern
    pub success_rate: f32,
    /// Domain this pattern is from
    pub domain: Option<String>,
    /// Times this pattern has been applied
    pub times_applied: usize,
}

/// Score breakdown for pattern result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreBreakdownResult {
    pub semantic_similarity: f32,
    pub context_match: f32,
    pub effectiveness: f32,
    pub recency: f32,
    pub success_rate: f32,
}

/// Execute search_patterns tool
pub async fn execute(
    memory: &SelfLearningMemory,
    input: SearchPatternsInput,
) -> anyhow::Result<Value> {
    // Build context
    let context = TaskContext {
        domain: input.domain.clone(),
        language: None,
        framework: None,
        complexity: memory_core::ComplexityLevel::Moderate,
        tags: input.tags.clone(),
    };

    // Build config
    let config = SearchConfig {
        min_relevance: input.min_relevance,
        filter_by_domain: input.filter_by_domain,
        ..SearchConfig::default()
    };

    // Execute search
    let results = memory
        .search_patterns_with_config(&input.query, context, config, input.limit)
        .await?;

    // Convert results
    let pattern_results: Vec<PatternResult> = results
        .iter()
        .map(pattern_search_result_to_pattern_result)
        .collect();

    let output = SearchPatternsOutput {
        results: pattern_results,
        total_searched: results.len(),
        query: input.query.clone(),
    };

    Ok(serde_json::to_value(output)?)
}

/// Convert PatternSearchResult to PatternResult
fn pattern_search_result_to_pattern_result(result: &PatternSearchResult) -> PatternResult {
    use memory_core::Pattern;

    let pattern = &result.pattern;
    let (id, pattern_type, description, domain) = match pattern {
        Pattern::ToolSequence {
            id, tools, context, ..
        } => (
            id.to_string(),
            "tool_sequence".to_string(),
            format!("Tool sequence: {}", tools.join(" → ")),
            Some(context.domain.clone()),
        ),
        Pattern::DecisionPoint {
            id,
            condition,
            action,
            context,
            ..
        } => (
            id.to_string(),
            "decision_point".to_string(),
            format!("Decision: {} → {}", condition, action),
            Some(context.domain.clone()),
        ),
        Pattern::ErrorRecovery {
            id,
            error_type,
            recovery_steps,
            context,
            ..
        } => (
            id.to_string(),
            "error_recovery".to_string(),
            format!(
                "Error recovery for {}: {}",
                error_type,
                recovery_steps.join(", ")
            ),
            Some(context.domain.clone()),
        ),
        Pattern::ContextPattern {
            id,
            recommended_approach,
            ..
        } => (
            id.to_string(),
            "context_pattern".to_string(),
            format!("Context pattern: {}", recommended_approach),
            None,
        ),
    };

    let effectiveness = pattern.effectiveness();

    PatternResult {
        id,
        pattern_type,
        description,
        relevance_score: result.relevance_score,
        score_breakdown: ScoreBreakdownResult {
            semantic_similarity: result.score_breakdown.semantic_similarity,
            context_match: result.score_breakdown.context_match,
            effectiveness: result.score_breakdown.effectiveness,
            recency: result.score_breakdown.recency,
            success_rate: result.score_breakdown.success_rate,
        },
        success_rate: pattern.success_rate(),
        domain,
        times_applied: effectiveness.times_applied,
    }
}

/// Input parameters for recommend_patterns tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendPatternsInput {
    /// Description of the task you're working on
    pub task_description: String,
    /// Domain of the task
    pub domain: String,
    /// Optional tags for context
    #[serde(default)]
    pub tags: Vec<String>,
    /// Maximum number of recommendations (default: 3)
    #[serde(default = "default_recommendation_limit")]
    pub limit: usize,
}

fn default_recommendation_limit() -> usize {
    3
}

/// Execute recommend_patterns tool
pub async fn execute_recommend(
    memory: &SelfLearningMemory,
    input: RecommendPatternsInput,
) -> anyhow::Result<Value> {
    // Build context
    let context = TaskContext {
        domain: input.domain.clone(),
        language: None,
        framework: None,
        complexity: memory_core::ComplexityLevel::Moderate,
        tags: input.tags.clone(),
    };

    // Execute recommendation
    let results = memory
        .recommend_patterns_for_task(&input.task_description, context, input.limit)
        .await?;

    // Convert results
    let pattern_results: Vec<PatternResult> = results
        .iter()
        .map(pattern_search_result_to_pattern_result)
        .collect();

    let output = SearchPatternsOutput {
        results: pattern_results,
        total_searched: results.len(),
        query: input.task_description.clone(),
    };

    Ok(serde_json::to_value(output)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_patterns_input_defaults() {
        let input = SearchPatternsInput {
            query: "test".to_string(),
            domain: "test".to_string(),
            tags: vec![],
            limit: default_limit(),
            min_relevance: default_min_relevance(),
            filter_by_domain: false,
        };

        assert_eq!(input.limit, 5);
        assert_eq!(input.min_relevance, 0.3);
    }

    #[test]
    fn test_recommend_patterns_input_defaults() {
        let input = RecommendPatternsInput {
            task_description: "test".to_string(),
            domain: "test".to_string(),
            tags: vec![],
            limit: default_recommendation_limit(),
        };

        assert_eq!(input.limit, 3);
    }
}
