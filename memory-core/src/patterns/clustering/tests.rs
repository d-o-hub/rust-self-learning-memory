//! # Clustering Tests
//!
//! Unit tests for pattern clustering functionality.

#[cfg(test)]
mod tests {
    use crate::pattern::PatternEffectiveness;
    use crate::pattern::Pattern;
    use crate::patterns::{
        ClusterCentroid, ClusteringConfig, EpisodeCluster, PatternClusterer,
    };
    use crate::types::{ComplexityLevel, ExecutionResult, TaskContext, TaskOutcome};
    use crate::Episode;
    use crate::ExecutionStep;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_pattern_tool_sequence(tools: Vec<&str>, domain: &str) -> Pattern {
        Pattern::ToolSequence {
            id: Uuid::new_v4(),
            tools: tools.iter().map(|s| (*s).to_string()).collect(),
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                complexity: ComplexityLevel::Moderate,
                framework: None,
                tags: vec!["test".to_string()],
            },
            success_rate: 0.9,
            avg_latency: chrono::Duration::milliseconds(100),
            occurrence_count: 5,
            effectiveness: PatternEffectiveness::default(),
        }
    }

    #[test]
    fn test_pattern_deduplication() {
        let clusterer = PatternClusterer::new();

        // Create similar patterns
        let p1 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p3 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");

        let patterns = vec![p1, p2, p3];

        let deduplicated = clusterer.deduplicate_patterns(patterns);

        // Should merge p1 and p2, keep p3 separate
        assert_eq!(deduplicated.len(), 2);
    }

    #[test]
    fn test_similarity_key_grouping() {
        let clusterer = PatternClusterer::new();

        let p1 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p3 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");

        let patterns = vec![p1, p2, p3];

        let groups = clusterer.group_by_similarity_key(patterns);

        // Should have 2 groups (identical patterns grouped together)
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_find_similar_patterns() {
        let clusterer = PatternClusterer::new();

        let target = create_test_pattern_tool_sequence(vec!["read", "write"], "web-api");
        let p1 = create_test_pattern_tool_sequence(vec!["read", "write", "close"], "web-api");
        let p2 = create_test_pattern_tool_sequence(vec!["compile", "test"], "cli");
        let p3 = create_test_pattern_tool_sequence(vec!["read", "update"], "web-api");

        let candidates = vec![p1, p2, p3];

        let similar = clusterer.find_similar_patterns(&target, &candidates, 2);

        // Should find 2 similar patterns, ordered by similarity
        assert_eq!(similar.len(), 2);
        // First should have higher similarity score
        assert!(similar[0].1 >= similar[1].1);
    }

    #[test]
    fn test_episode_clustering() {
        let clusterer = PatternClusterer::with_config(ClusteringConfig {
            num_clusters: 2,
            ..ClusteringConfig::default()
        });

        // Create test episodes with different contexts
        let episodes = vec![
            create_test_episode("web-api", 5),
            create_test_episode("web-api", 6),
            create_test_episode("cli", 3),
            create_test_episode("cli", 4),
        ];

        let clusters = clusterer.cluster_episodes(episodes);

        // Should create 2 clusters
        assert!(!clusters.is_empty());
        assert!(clusters.len() <= 2);

        // Each cluster should have episodes
        for cluster in &clusters {
            assert!(cluster.size() > 0);
        }
    }

    #[test]
    fn test_cluster_success_rate() {
        let cluster = EpisodeCluster {
            centroid: ClusterCentroid::default(),
            episodes: vec![
                create_successful_episode(),
                create_successful_episode(),
                create_failed_episode(),
            ],
        };

        let success_rate = cluster.success_rate();

        // 2 out of 3 succeeded
        assert!((success_rate - 0.666).abs() < 0.01);
    }

    // Helper functions for tests
    fn create_test_episode(domain: &str, step_count: usize) -> Episode {
        let mut episode = Episode {
            episode_id: Uuid::new_v4(),
            task_type: crate::types::TaskType::CodeGeneration,
            task_description: "Test task".to_string(),
            context: TaskContext {
                domain: domain.to_string(),
                language: Some("rust".to_string()),
                complexity: ComplexityLevel::Moderate,
                framework: None,
                tags: vec![],
            },
            start_time: Utc::now(),
            end_time: None,
            steps: Vec::new(),
            outcome: None,
            reward: None,
            reflection: None,
            patterns: Vec::new(),
            heuristics: Vec::new(),
            applied_patterns: Vec::new(),
            salient_features: None,
            metadata: std::collections::HashMap::new(),
        };

        for i in 0..step_count {
            episode.steps.push(ExecutionStep {
                step_number: i + 1,
                tool: format!("tool_{i}"),
                action: format!("action_{i}"),
                timestamp: Utc::now(),
                result: Some(ExecutionResult::Success {
                    output: "ok".to_string(),
                }),
                latency_ms: 100,
                tokens_used: None,
                metadata: std::collections::HashMap::new(),
                parameters: serde_json::Value::Null,
            });
        }

        episode
    }

    fn create_successful_episode() -> Episode {
        let mut episode = create_test_episode("test", 3);
        episode.outcome = Some(TaskOutcome::Success {
            verdict: "Success".to_string(),
            artifacts: vec![],
        });
        episode
    }

    fn create_failed_episode() -> Episode {
        let mut episode = create_test_episode("test", 3);
        episode.outcome = Some(TaskOutcome::Failure {
            reason: "Failed".to_string(),
            error_details: None,
        });
        episode
    }

    #[test]
    fn test_extract_common_patterns() {
        use crate::types::TaskType;

        // Create test episodes
        let mut episodes = vec![];

        // Common pattern ID that appears in multiple episodes
        let common_pattern_id = Uuid::new_v4();
        let rare_pattern_id = Uuid::new_v4();

        // Create 3 episodes, 2 with the common pattern
        for i in 0..3 {
            let mut episode = Episode::new(
                format!("Task {i}"),
                TaskContext {
                    domain: "test".to_string(),
                    language: None,
                    complexity: ComplexityLevel::Simple,
                    framework: None,
                    tags: vec![],
                },
                TaskType::Testing,
            );

            // First 2 episodes have the common pattern
            if i < 2 {
                episode.patterns.push(common_pattern_id);
            }

            // Only the first episode has the rare pattern
            if i == 0 {
                episode.patterns.push(rare_pattern_id);
            }

            episodes.push(episode);
        }

        let cluster = EpisodeCluster {
            centroid: ClusterCentroid::default(),
            episodes,
        };

        let common = cluster.extract_common_patterns();

        // Should find the common pattern (appears in 2/3 episodes = 66%)
        // The rare pattern should NOT appear (only in 1/3 = 33%)
        assert_eq!(common.len(), 1, "Should extract exactly one common pattern");
        assert_eq!(
            common[0], common_pattern_id,
            "Should extract the pattern that appears in 2 episodes"
        );
    }
}
