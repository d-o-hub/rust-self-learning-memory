use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::episode::PatternId;
use crate::types::{Evidence, OutcomeStats, TaskContext};

/// Pattern types extracted from episodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Pattern {
    /// Sequence of tools used successfully
    ToolSequence {
        id: PatternId,
        tools: Vec<String>,
        context: TaskContext,
        success_rate: f32,
        avg_latency: Duration,
        occurrence_count: usize,
    },
    /// Decision point with outcome statistics
    DecisionPoint {
        id: PatternId,
        condition: String,
        action: String,
        outcome_stats: OutcomeStats,
        context: TaskContext,
    },
    /// Error recovery pattern
    ErrorRecovery {
        id: PatternId,
        error_type: String,
        recovery_steps: Vec<String>,
        success_rate: f32,
        context: TaskContext,
    },
    /// Context-based pattern
    ContextPattern {
        id: PatternId,
        context_features: Vec<String>,
        recommended_approach: String,
        evidence: Vec<Uuid>, // Episode IDs
        success_rate: f32,
    },
}

impl Pattern {
    /// Get the unique ID of this pattern
    pub fn id(&self) -> PatternId {
        match self {
            Pattern::ToolSequence { id, .. } => *id,
            Pattern::DecisionPoint { id, .. } => *id,
            Pattern::ErrorRecovery { id, .. } => *id,
            Pattern::ContextPattern { id, .. } => *id,
        }
    }

    /// Get the success rate of this pattern
    pub fn success_rate(&self) -> f32 {
        match self {
            Pattern::ToolSequence { success_rate, .. } => *success_rate,
            Pattern::DecisionPoint { outcome_stats, .. } => outcome_stats.success_rate(),
            Pattern::ErrorRecovery { success_rate, .. } => *success_rate,
            Pattern::ContextPattern { success_rate, .. } => *success_rate,
        }
    }

    /// Get the context associated with this pattern
    pub fn context(&self) -> Option<&TaskContext> {
        match self {
            Pattern::ToolSequence { context, .. } => Some(context),
            Pattern::DecisionPoint { context, .. } => Some(context),
            Pattern::ErrorRecovery { context, .. } => Some(context),
            Pattern::ContextPattern { .. } => None,
        }
    }

    /// Check if this pattern is relevant to a given context
    pub fn is_relevant_to(&self, query_context: &TaskContext) -> bool {
        if let Some(pattern_context) = self.context() {
            // Match on domain
            if pattern_context.domain == query_context.domain {
                return true;
            }

            // Match on language
            if pattern_context.language == query_context.language
                && pattern_context.language.is_some()
            {
                return true;
            }

            // Match on tags
            let common_tags: Vec<_> = pattern_context
                .tags
                .iter()
                .filter(|t| query_context.tags.contains(t))
                .collect();

            if !common_tags.is_empty() {
                return true;
            }
        }

        false
    }

    /// Get a similarity key for pattern deduplication
    /// Patterns with identical keys are considered duplicates
    pub fn similarity_key(&self) -> String {
        match self {
            Pattern::ToolSequence { tools, context, .. } => {
                format!("tool_seq:{}:{}", tools.join(","), context.domain)
            }
            Pattern::DecisionPoint {
                condition,
                action,
                context,
                ..
            } => {
                format!("decision:{}:{}:{}", condition, action, context.domain)
            }
            Pattern::ErrorRecovery {
                error_type,
                recovery_steps,
                context,
                ..
            } => {
                format!(
                    "error_recovery:{}:{}:{}",
                    error_type,
                    recovery_steps.join(","),
                    context.domain
                )
            }
            Pattern::ContextPattern {
                context_features,
                recommended_approach,
                ..
            } => {
                format!(
                    "context:{}:{}",
                    context_features.join(","),
                    recommended_approach
                )
            }
        }
    }

    /// Calculate similarity score between this pattern and another (0.0 to 1.0)
    /// Uses edit distance for sequences and context matching
    pub fn similarity_score(&self, other: &Self) -> f32 {
        // Different pattern types have zero similarity
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return 0.0;
        }

        match (self, other) {
            (
                Pattern::ToolSequence {
                    tools: tools1,
                    context: ctx1,
                    ..
                },
                Pattern::ToolSequence {
                    tools: tools2,
                    context: ctx2,
                    ..
                },
            ) => {
                let sequence_similarity = sequence_similarity(tools1, tools2);
                let context_similarity = context_similarity(ctx1, ctx2);
                // Weight: 70% sequence, 30% context
                sequence_similarity * 0.7 + context_similarity * 0.3
            }
            (
                Pattern::DecisionPoint {
                    condition: cond1,
                    action: act1,
                    context: ctx1,
                    ..
                },
                Pattern::DecisionPoint {
                    condition: cond2,
                    action: act2,
                    context: ctx2,
                    ..
                },
            ) => {
                let condition_sim = string_similarity(cond1, cond2);
                let action_sim = string_similarity(act1, act2);
                let context_sim = context_similarity(ctx1, ctx2);
                // Weight: 40% condition, 40% action, 20% context
                condition_sim * 0.4 + action_sim * 0.4 + context_sim * 0.2
            }
            (
                Pattern::ErrorRecovery {
                    error_type: err1,
                    recovery_steps: steps1,
                    context: ctx1,
                    ..
                },
                Pattern::ErrorRecovery {
                    error_type: err2,
                    recovery_steps: steps2,
                    context: ctx2,
                    ..
                },
            ) => {
                let error_sim = string_similarity(err1, err2);
                let steps_sim = sequence_similarity(steps1, steps2);
                let context_sim = context_similarity(ctx1, ctx2);
                // Weight: 40% error type, 40% recovery steps, 20% context
                error_sim * 0.4 + steps_sim * 0.4 + context_sim * 0.2
            }
            (
                Pattern::ContextPattern {
                    context_features: feat1,
                    recommended_approach: rec1,
                    ..
                },
                Pattern::ContextPattern {
                    context_features: feat2,
                    recommended_approach: rec2,
                    ..
                },
            ) => {
                let features_sim = sequence_similarity(feat1, feat2);
                let approach_sim = string_similarity(rec1, rec2);
                // Weight: 60% features, 40% approach
                features_sim * 0.6 + approach_sim * 0.4
            }
            _ => 0.0,
        }
    }

    /// Calculate confidence score for this pattern
    /// Confidence = success_rate * sqrt(sample_size)
    pub fn confidence(&self) -> f32 {
        let success_rate = self.success_rate();
        let sample_size = self.sample_size() as f32;

        if sample_size == 0.0 {
            return 0.0;
        }

        success_rate * sample_size.sqrt()
    }

    /// Get the sample size (number of occurrences) for this pattern
    fn sample_size(&self) -> usize {
        match self {
            Pattern::ToolSequence {
                occurrence_count, ..
            } => *occurrence_count,
            Pattern::DecisionPoint { outcome_stats, .. } => outcome_stats.total_count,
            Pattern::ErrorRecovery { .. } => {
                // For error recovery, we estimate from context complexity
                // This is a fallback; ideally we'd track occurrences
                1
            }
            Pattern::ContextPattern { evidence, .. } => evidence.len(),
        }
    }

    /// Merge this pattern with another similar pattern
    /// Combines evidence and updates statistics
    pub fn merge_with(&mut self, other: &Self) {
        // Can only merge patterns of the same type
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return;
        }

        match (self, other) {
            (
                Pattern::ToolSequence {
                    success_rate: sr1,
                    occurrence_count: oc1,
                    avg_latency: lat1,
                    ..
                },
                Pattern::ToolSequence {
                    success_rate: sr2,
                    occurrence_count: oc2,
                    avg_latency: lat2,
                    ..
                },
            ) => {
                let total_count = *oc1 + *oc2;
                // Weighted average of success rates
                *sr1 = (*sr1 * *oc1 as f32 + *sr2 * *oc2 as f32) / total_count as f32;
                // Weighted average of latencies
                *lat1 = Duration::milliseconds(
                    (lat1.num_milliseconds() * *oc1 as i64 + lat2.num_milliseconds() * *oc2 as i64)
                        / total_count as i64,
                );
                *oc1 = total_count;
            }
            (
                Pattern::DecisionPoint {
                    outcome_stats: stats1,
                    ..
                },
                Pattern::DecisionPoint {
                    outcome_stats: stats2,
                    ..
                },
            ) => {
                stats1.success_count += stats2.success_count;
                stats1.failure_count += stats2.failure_count;
                stats1.total_count += stats2.total_count;
                // Weighted average of durations
                stats1.avg_duration_secs = (stats1.avg_duration_secs
                    * (stats1.total_count - stats2.total_count) as f32
                    + stats2.avg_duration_secs * stats2.total_count as f32)
                    / stats1.total_count as f32;
            }
            (
                Pattern::ErrorRecovery {
                    success_rate: sr1, ..
                },
                Pattern::ErrorRecovery {
                    success_rate: sr2, ..
                },
            ) => {
                // Simple average for error recovery patterns
                *sr1 = (*sr1 + *sr2) / 2.0;
                // Keep the richer context (more tags)
                // Context is already part of self
            }
            (
                Pattern::ContextPattern {
                    evidence: ev1,
                    success_rate: sr1,
                    ..
                },
                Pattern::ContextPattern {
                    evidence: ev2,
                    success_rate: sr2,
                    ..
                },
            ) => {
                let size1 = ev1.len();
                let size2 = ev2.len();
                // Combine evidence
                ev1.extend_from_slice(ev2);
                // Weighted average of success rates
                *sr1 = (*sr1 * size1 as f32 + *sr2 * size2 as f32) / (size1 + size2) as f32;
            }
            _ => {}
        }
    }
}

/// Calculate similarity between two sequences using normalized edit distance
fn sequence_similarity(seq1: &[String], seq2: &[String]) -> f32 {
    if seq1.is_empty() && seq2.is_empty() {
        return 1.0;
    }
    if seq1.is_empty() || seq2.is_empty() {
        return 0.0;
    }

    let distance = edit_distance(seq1, seq2);
    let max_len = seq1.len().max(seq2.len());

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance (Levenshtein) between two sequences
fn edit_distance(seq1: &[String], seq2: &[String]) -> usize {
    let len1 = seq1.len();
    let len2 = seq2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if seq1[i - 1] == seq2[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1) // deletion
                .min(matrix[i][j - 1] + 1) // insertion
                .min(matrix[i - 1][j - 1] + cost); // substitution
        }
    }

    matrix[len1][len2]
}

/// Calculate similarity between two strings using normalized edit distance
fn string_similarity(s1: &str, s2: &str) -> f32 {
    if s1.is_empty() && s2.is_empty() {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    let distance = char_edit_distance(&chars1, &chars2);
    let max_len = chars1.len().max(chars2.len());

    1.0 - (distance as f32 / max_len as f32)
}

/// Calculate edit distance for character sequences
fn char_edit_distance(chars1: &[char], chars2: &[char]) -> usize {
    let len1 = chars1.len();
    let len2 = chars2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

/// Calculate context similarity between two task contexts
fn context_similarity(ctx1: &TaskContext, ctx2: &TaskContext) -> f32 {
    let mut score = 0.0;
    let mut weight_sum = 0.0;

    // Domain match (weight: 0.4)
    if ctx1.domain == ctx2.domain {
        score += 0.4;
    }
    weight_sum += 0.4;

    // Language match (weight: 0.3)
    match (&ctx1.language, &ctx2.language) {
        (Some(l1), Some(l2)) if l1 == l2 => score += 0.3,
        (None, None) => score += 0.15, // Partial credit for both being None
        _ => {}
    }
    weight_sum += 0.3;

    // Tags overlap (weight: 0.3)
    if !ctx1.tags.is_empty() || !ctx2.tags.is_empty() {
        let common_tags: Vec<_> = ctx1.tags.iter().filter(|t| ctx2.tags.contains(t)).collect();

        let total_unique_tags = ctx1
            .tags
            .iter()
            .chain(ctx2.tags.iter())
            .collect::<std::collections::HashSet<_>>()
            .len();

        if total_unique_tags > 0 {
            let jaccard = common_tags.len() as f32 / total_unique_tags as f32;
            score += jaccard * 0.3;
        }
    }
    weight_sum += 0.3;

    if weight_sum > 0.0 {
        score / weight_sum
    } else {
        0.0
    }
}

/// Heuristic rule learned from patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Heuristic {
    /// Unique heuristic ID
    pub heuristic_id: Uuid,
    /// Condition to check (as natural language or code)
    pub condition: String,
    /// Recommended action
    pub action: String,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
    /// Evidence supporting this heuristic
    pub evidence: Evidence,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
}

impl Heuristic {
    /// Create a new heuristic
    pub fn new(condition: String, action: String, confidence: f32) -> Self {
        let now = Utc::now();
        Self {
            heuristic_id: Uuid::new_v4(),
            condition,
            action,
            confidence,
            evidence: Evidence {
                episode_ids: Vec::new(),
                success_rate: 0.0,
                sample_size: 0,
            },
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the heuristic with new evidence
    pub fn update_evidence(&mut self, episode_id: Uuid, success: bool) {
        self.evidence.episode_ids.push(episode_id);
        self.evidence.sample_size += 1;

        // Recalculate success rate
        let successes = if success {
            (self.evidence.success_rate * (self.evidence.sample_size - 1) as f32) + 1.0
        } else {
            self.evidence.success_rate * (self.evidence.sample_size - 1) as f32
        };

        self.evidence.success_rate = successes / self.evidence.sample_size as f32;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ComplexityLevel;

    #[test]
    fn test_pattern_id() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string(), "tool2".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
        };

        assert!(pattern.success_rate() > 0.8);
        assert!(pattern.context().is_some());
    }

    #[test]
    fn test_pattern_similarity_key() {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                ..Default::default()
            },
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                ..Default::default()
            },
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 3,
        };

        // Same tools and domain = same key
        assert_eq!(pattern1.similarity_key(), pattern2.similarity_key());
    }

    #[test]
    fn test_pattern_similarity_score() {
        let pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                ..Default::default()
            },
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 5,
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                ..Default::default()
            },
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(120),
            occurrence_count: 3,
        };

        let similarity = pattern1.similarity_score(&pattern2);

        // Identical tools and context should have high similarity
        assert!(similarity > 0.9);
    }

    #[test]
    fn test_pattern_confidence() {
        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["tool1".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 16, // sqrt(16) = 4
        };

        let confidence = pattern.confidence();

        // 0.8 * sqrt(16) = 0.8 * 4 = 3.2
        assert!((confidence - 3.2).abs() < 0.01);
    }

    #[test]
    fn test_pattern_merge() {
        let mut pattern1 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext::default(),
            success_rate: 0.8,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 10,
        };

        let pattern2 = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec!["read".to_string(), "write".to_string()],
            context: TaskContext::default(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(200),
            occurrence_count: 10,
        };

        pattern1.merge_with(&pattern2);

        // Should have combined occurrence count
        match pattern1 {
            Pattern::ToolSequence {
                occurrence_count,
                success_rate,
                ..
            } => {
                assert_eq!(occurrence_count, 20);
                // Average: (0.8 * 10 + 0.9 * 10) / 20 = 0.85
                assert!((success_rate - 0.85).abs() < 0.01);
            }
            _ => panic!("Expected ToolSequence"),
        }
    }

    #[test]
    fn test_sequence_similarity() {
        let seq1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let seq2 = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        assert_eq!(sequence_similarity(&seq1, &seq2), 1.0);

        let seq3 = vec!["a".to_string(), "b".to_string(), "d".to_string()];
        let sim = sequence_similarity(&seq1, &seq3);
        // 2 out of 3 match
        assert!(sim > 0.6 && sim < 0.7);
    }

    #[test]
    fn test_string_similarity() {
        assert_eq!(string_similarity("hello", "hello"), 1.0);
        assert_eq!(string_similarity("", ""), 1.0);
        assert_eq!(string_similarity("abc", ""), 0.0);

        // "hello" vs "hallo" - one character different
        let sim = string_similarity("hello", "hallo");
        assert!(sim > 0.7 && sim < 0.9);
    }

    #[test]
    fn test_context_similarity() {
        let ctx1 = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["async".to_string(), "http".to_string()],
            ..Default::default()
        };

        let ctx2 = TaskContext {
            domain: "web-api".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["async".to_string(), "rest".to_string()],
            ..Default::default()
        };

        let similarity = context_similarity(&ctx1, &ctx2);

        // Same domain, same language, some tag overlap
        assert!(similarity > 0.7);
    }

    #[test]
    fn test_pattern_relevance() {
        let pattern_context = TaskContext {
            language: Some("rust".to_string()),
            framework: None,
            complexity: ComplexityLevel::Moderate,
            domain: "web-api".to_string(),
            tags: vec!["async".to_string()],
        };

        let pattern = Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: vec![],
            context: pattern_context.clone(),
            success_rate: 0.9,
            avg_latency: Duration::milliseconds(100),
            occurrence_count: 1,
        };

        // Should match on domain
        let query_context = TaskContext {
            domain: "web-api".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context));

        // Should match on language
        let query_context2 = TaskContext {
            language: Some("rust".to_string()),
            domain: "cli".to_string(),
            ..Default::default()
        };
        assert!(pattern.is_relevant_to(&query_context2));

        // Should not match
        let query_context3 = TaskContext {
            language: Some("python".to_string()),
            domain: "data-science".to_string(),
            ..Default::default()
        };
        assert!(!pattern.is_relevant_to(&query_context3));
    }

    #[test]
    fn test_heuristic_evidence_update() {
        let mut heuristic = Heuristic::new(
            "When refactoring async code".to_string(),
            "Use tokio::spawn for CPU-intensive tasks".to_string(),
            0.7,
        );

        assert_eq!(heuristic.evidence.sample_size, 0);

        // Add successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 1);
        assert_eq!(heuristic.evidence.success_rate, 1.0);

        // Add failed evidence
        heuristic.update_evidence(Uuid::new_v4(), false);
        assert_eq!(heuristic.evidence.sample_size, 2);
        assert_eq!(heuristic.evidence.success_rate, 0.5);

        // Add more successful evidence
        heuristic.update_evidence(Uuid::new_v4(), true);
        assert_eq!(heuristic.evidence.sample_size, 3);
        assert!((heuristic.evidence.success_rate - 0.666).abs() < 0.01);
    }
}
