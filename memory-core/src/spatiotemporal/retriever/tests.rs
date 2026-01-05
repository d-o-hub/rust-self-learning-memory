//! Tests for hierarchical retrieval

use super::*;
use crate::episode::Episode;
use crate::types::{ComplexityLevel, TaskContext, TaskOutcome, TaskType};
use chrono::{Duration, Utc};

fn create_test_episode(
    domain: &str,
    task_type: TaskType,
    description: &str,
    days_ago: i64,
) -> Episode {
    let context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        domain: domain.to_string(),
        tags: vec![],
    };

    let mut episode = Episode::new(description.to_string(), context, task_type);
    episode.start_time = Utc::now() - Duration::days(days_ago);

    // Complete the episode
    episode.complete(TaskOutcome::Success {
        verdict: "Success".to_string(),
        artifacts: vec![],
    });

    episode
}

#[test]
fn test_retriever_creation() {
    let retriever = HierarchicalRetriever::new();
    assert_eq!(retriever.temporal_bias_weight, 0.3);
    assert_eq!(retriever.max_clusters_to_search, 5);

    let custom = HierarchicalRetriever::with_config(0.5, 10);
    assert_eq!(custom.temporal_bias_weight, 0.5);
    assert_eq!(custom.max_clusters_to_search, 10);
}

#[test]
fn test_domain_filtering() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "task 1", 1),
        create_test_episode("backend", TaskType::CodeGeneration, "task 2", 1),
        create_test_episode("web-api", TaskType::Debugging, "task 3", 1),
    ];

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 5,
    };

    let filtered = retriever.filter_by_domain(&episodes, &query);
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|ep| ep.context.domain == "web-api"));
}

#[test]
fn test_domain_filtering_no_filter() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "task 1", 1),
        create_test_episode("backend", TaskType::CodeGeneration, "task 2", 1),
    ];

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 5,
    };

    let filtered = retriever.filter_by_domain(&episodes, &query);
    assert_eq!(filtered.len(), 2); // No filtering, all episodes returned
}

#[test]
fn test_task_type_filtering() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "task 1", 1),
        create_test_episode("web-api", TaskType::Debugging, "task 2", 1),
        create_test_episode("web-api", TaskType::CodeGeneration, "task 3", 1),
    ];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: None,
        task_type: Some(TaskType::CodeGeneration),
        limit: 5,
    };

    let filtered = retriever.filter_by_task_type(&candidates, &query);
    assert_eq!(filtered.len(), 2);
    assert!(filtered
        .iter()
        .all(|ep| ep.task_type == TaskType::CodeGeneration));
}

#[test]
fn test_task_type_filtering_no_filter() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "task 1", 1),
        create_test_episode("web-api", TaskType::Debugging, "task 2", 1),
    ];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 5,
    };

    let filtered = retriever.filter_by_task_type(&candidates, &query);
    assert_eq!(filtered.len(), 2); // No filtering
}

#[test]
fn test_temporal_clustering_favors_recent() {
    let retriever = HierarchicalRetriever::new();

    // Create episodes with different ages
    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "old task", 30),
        create_test_episode("web-api", TaskType::CodeGeneration, "recent task", 1),
        create_test_episode("web-api", TaskType::CodeGeneration, "medium task", 15),
    ];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 5,
    };

    let clustered = retriever.select_temporal_clusters(&candidates, &query);

    // Should return recent episodes first
    assert!(!clustered.is_empty());

    // Verify most recent episode is included
    let has_recent = clustered
        .iter()
        .any(|ep| ep.task_description == "recent task");
    assert!(has_recent);
}

#[test]
fn test_scoring_domain_match() {
    let retriever = HierarchicalRetriever::new();

    let episodes = [create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "implement auth",
        1,
    )];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "implement auth".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 1,
    };

    let scored = retriever.score_episodes(&candidates, &query);

    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].level_1_score, 1.0); // Perfect domain match
}

#[test]
fn test_scoring_task_type_match() {
    let retriever = HierarchicalRetriever::new();

    let episodes = [create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "task",
        1,
    )];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "task".to_string(),
        query_embedding: None,
        domain: None,
        task_type: Some(TaskType::CodeGeneration),
        limit: 1,
    };

    let scored = retriever.score_episodes(&candidates, &query);

    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].level_2_score, 1.0); // Perfect task type match
}

#[test]
fn test_scoring_temporal_proximity() {
    let retriever = HierarchicalRetriever::new();

    let recent_ep = create_test_episode("web-api", TaskType::CodeGeneration, "recent", 1);
    let old_ep = create_test_episode("web-api", TaskType::CodeGeneration, "old", 25);

    let candidates = vec![&recent_ep, &old_ep];

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 2,
    };

    let scored = retriever.score_episodes(&candidates, &query);

    assert_eq!(scored.len(), 2);

    // Find scores for recent vs old
    let recent_score = scored
        .iter()
        .find(|s| s.episode_id == recent_ep.episode_id)
        .unwrap()
        .level_3_score;
    let old_score = scored
        .iter()
        .find(|s| s.episode_id == old_ep.episode_id)
        .unwrap()
        .level_3_score;

    // Recent should have higher temporal score
    assert!(recent_score > old_score);
}

#[test]
fn test_text_similarity() {
    let retriever = HierarchicalRetriever::new();

    let episodes = [create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "implement authentication system",
        1,
    )];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "authentication".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 1,
    };

    let scored = retriever.score_episodes(&candidates, &query);

    assert_eq!(scored.len(), 1);
    // Should have non-zero text similarity
    assert!(scored[0].level_4_score > 0.0);
}

#[test]
fn test_ranking_sorts_by_relevance() {
    let retriever = HierarchicalRetriever::new();

    let scored = vec![
        HierarchicalScore {
            episode_id: uuid::Uuid::new_v4(),
            relevance_score: 0.5,
            level_1_score: 0.5,
            level_2_score: 0.5,
            level_3_score: 0.5,
            level_4_score: 0.5,
        },
        HierarchicalScore {
            episode_id: uuid::Uuid::new_v4(),
            relevance_score: 0.9,
            level_1_score: 1.0,
            level_2_score: 1.0,
            level_3_score: 0.8,
            level_4_score: 0.8,
        },
        HierarchicalScore {
            episode_id: uuid::Uuid::new_v4(),
            relevance_score: 0.3,
            level_1_score: 0.3,
            level_2_score: 0.3,
            level_3_score: 0.3,
            level_4_score: 0.3,
        },
    ];

    let ranked = retriever.rank_by_combined_score(scored.clone());

    assert_eq!(ranked.len(), 3);
    // Should be sorted in descending order
    assert!(ranked[0].relevance_score >= ranked[1].relevance_score);
    assert!(ranked[1].relevance_score >= ranked[2].relevance_score);

    // Highest score should be first
    assert_eq!(ranked[0].relevance_score, 0.9);
}

#[test]
fn test_temporal_bias_weight_effect() {
    let low_bias = HierarchicalRetriever::with_config(0.1, 5);
    let high_bias = HierarchicalRetriever::with_config(0.7, 5);

    assert_eq!(low_bias.temporal_bias_weight, 0.1);
    assert_eq!(high_bias.temporal_bias_weight, 0.7);
}

#[tokio::test]
async fn test_full_retrieval_workflow() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "implement auth", 1),
        create_test_episode("web-api", TaskType::Debugging, "fix auth bug", 2),
        create_test_episode("backend", TaskType::CodeGeneration, "database schema", 3),
    ];

    let query = RetrievalQuery {
        query_text: "authentication".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 5,
    };

    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    // Should filter to web-api domain only
    assert!(results.len() <= 2);

    // Results should be ranked
    if results.len() > 1 {
        assert!(results[0].relevance_score >= results[1].relevance_score);
    }
}

#[tokio::test]
async fn test_retrieval_empty_episodes() {
    let retriever = HierarchicalRetriever::new();
    let episodes = vec![];

    let query = RetrievalQuery {
        query_text: "test".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: None,
        limit: 5,
    };

    let scored = retriever.retrieve(&query, &episodes).await.unwrap();

    assert_eq!(scored.len(), 0);
}

#[test]
fn test_combined_score_calculation() {
    let retriever = HierarchicalRetriever::new();

    let episodes = [create_test_episode(
        "web-api",
        TaskType::CodeGeneration,
        "implement authentication",
        1,
    )];

    let candidates: Vec<&Episode> = episodes.iter().collect();

    let query = RetrievalQuery {
        query_text: "implement authentication".to_string(),
        query_embedding: None,
        domain: Some("web-api".to_string()),
        task_type: Some(TaskType::CodeGeneration),
        limit: 1,
    };

    let scored = retriever.score_episodes(&candidates, &query);

    assert_eq!(scored.len(), 1);

    // Perfect match should have high relevance score
    let result = &scored[0];
    assert!(result.relevance_score > 0.7);
    assert!(result.level_1_score > 0.9); // Domain match
    assert!(result.level_2_score > 0.9); // Task type match
    assert!(result.level_3_score > 0.9); // Recent (1 day old)
    assert!(result.level_4_score > 0.5); // Text similarity
}

#[tokio::test]
async fn test_retrieval_with_no_filters() {
    let retriever = HierarchicalRetriever::new();

    let episodes = vec![
        create_test_episode("web-api", TaskType::CodeGeneration, "task 1", 1),
        create_test_episode("backend", TaskType::Debugging, "task 2", 2),
        create_test_episode("frontend", TaskType::Testing, "task 3", 3),
    ];

    let query = RetrievalQuery {
        query_text: "task".to_string(),
        query_embedding: None,
        domain: None,
        task_type: None,
        limit: 10,
    };

    let results = retriever.retrieve(&query, &episodes).await.unwrap();

    // No filters, should consider all episodes
    assert_eq!(results.len(), 3);
}
