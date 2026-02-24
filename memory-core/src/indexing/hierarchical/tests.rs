//! Tests for hierarchical indexing.

use crate::episode::Episode;
use crate::indexing::hierarchical::{HierarchicalIndex, HierarchicalQuery};
use crate::types::TaskType;
use crate::types::{ComplexityLevel, TaskContext};
use chrono::Utc;

fn create_test_episode(domain: &str, task_type: TaskType) -> Episode {
    let context = TaskContext {
        domain: domain.to_string(),
        complexity: ComplexityLevel::Simple,
        tags: vec![],
        ..Default::default()
    };
    Episode::new("Test episode".to_string(), context, task_type)
}

#[test]
fn test_index_creation() {
    let index = HierarchicalIndex::new();
    assert!(index.is_empty());
    assert_eq!(index.len(), 0);
    assert_eq!(index.domain_count(), 0);
}

#[test]
fn test_insert_and_query_domain() {
    let mut index = HierarchicalIndex::new();

    let episode1 = create_test_episode("web-api", TaskType::CodeGeneration);
    let episode2 = create_test_episode("web-api", TaskType::Debugging);
    let episode3 = create_test_episode("data-processing", TaskType::Analysis);

    let id1 = episode1.episode_id;
    let id2 = episode2.episode_id;

    index.insert(&episode1);
    index.insert(&episode2);
    index.insert(&episode3);

    assert_eq!(index.len(), 3);
    assert_eq!(index.domain_count(), 2);

    // Query by domain
    let results = index.query_by_domain("web-api", 100);
    assert_eq!(results.len(), 2);
    assert!(results.contains(&id1));
    assert!(results.contains(&id2));

    // Query non-existent domain
    let results = index.query_by_domain("nonexistent", 100);
    assert!(results.is_empty());
}

#[test]
fn test_query_by_task_type() {
    let mut index = HierarchicalIndex::new();

    let episode1 = create_test_episode("web-api", TaskType::CodeGeneration);
    let episode2 = create_test_episode("web-api", TaskType::CodeGeneration);
    let episode3 = create_test_episode("web-api", TaskType::Debugging);

    let id1 = episode1.episode_id;
    let id2 = episode2.episode_id;

    index.insert(&episode1);
    index.insert(&episode2);
    index.insert(&episode3);

    // Query by task type
    let results = index.query_by_task_type("web-api", TaskType::CodeGeneration, 100);
    assert_eq!(results.len(), 2);
    assert!(results.contains(&id1));
    assert!(results.contains(&id2));

    // Query different task type
    let results = index.query_by_task_type("web-api", TaskType::Analysis, 100);
    assert!(results.is_empty());
}

#[test]
fn test_hierarchical_query() {
    let mut index = HierarchicalIndex::new();

    let episode = create_test_episode("web-api", TaskType::CodeGeneration);
    let id = episode.episode_id;

    index.insert(&episode);

    // Query with domain filter
    let query = HierarchicalQuery::new()
        .with_domain("web-api")
        .with_limit(10);
    let results = index.query(&query);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], id);

    // Query with domain and task type
    let query = HierarchicalQuery::new()
        .with_domain("web-api")
        .with_task_type(TaskType::CodeGeneration)
        .with_limit(10);
    let results = index.query(&query);
    assert_eq!(results.len(), 1);

    // Query with non-matching filters
    let query = HierarchicalQuery::new()
        .with_domain("web-api")
        .with_task_type(TaskType::Debugging)
        .with_limit(10);
    let results = index.query(&query);
    assert!(results.is_empty());
}

#[test]
fn test_remove() {
    let mut index = HierarchicalIndex::new();

    let episode = create_test_episode("web-api", TaskType::CodeGeneration);

    index.insert(&episode);
    assert_eq!(index.len(), 1);

    let removed = index.remove(&episode);
    assert!(removed);
    assert_eq!(index.len(), 0);

    // Remove non-existent episode
    let removed = index.remove(&episode);
    assert!(!removed);
}

#[test]
fn test_domain_episode_count() {
    let mut index = HierarchicalIndex::new();

    for _ in 0..5 {
        index.insert(&create_test_episode("domain-a", TaskType::CodeGeneration));
    }

    for _ in 0..3 {
        index.insert(&create_test_episode("domain-b", TaskType::CodeGeneration));
    }

    assert_eq!(index.domain_episode_count("domain-a"), 5);
    assert_eq!(index.domain_episode_count("domain-b"), 3);
    assert_eq!(index.domain_episode_count("nonexistent"), 0);
}

#[test]
fn test_domains_list() {
    let mut index = HierarchicalIndex::new();

    index.insert(&create_test_episode("domain-a", TaskType::CodeGeneration));
    index.insert(&create_test_episode("domain-b", TaskType::CodeGeneration));
    index.insert(&create_test_episode("domain-c", TaskType::CodeGeneration));

    let domains = index.domains();
    assert_eq!(domains.len(), 3);
    assert!(domains.contains(&"domain-a".to_string()));
    assert!(domains.contains(&"domain-b".to_string()));
    assert!(domains.contains(&"domain-c".to_string()));
}

#[test]
fn test_task_types_for_domain() {
    let mut index = HierarchicalIndex::new();

    index.insert(&create_test_episode("web-api", TaskType::CodeGeneration));
    index.insert(&create_test_episode("web-api", TaskType::CodeGeneration));
    index.insert(&create_test_episode("web-api", TaskType::Debugging));
    index.insert(&create_test_episode("web-api", TaskType::Analysis));

    let task_types = index.task_types_for_domain("web-api");
    assert_eq!(task_types.len(), 3);
    assert!(task_types.contains(&TaskType::CodeGeneration));
    assert!(task_types.contains(&TaskType::Debugging));
    assert!(task_types.contains(&TaskType::Analysis));
}

#[test]
fn test_clear() {
    let mut index = HierarchicalIndex::new();

    for _ in 0..10 {
        index.insert(&create_test_episode("domain", TaskType::CodeGeneration));
    }

    assert_eq!(index.len(), 10);
    assert_eq!(index.domain_count(), 1);

    index.clear();

    assert!(index.is_empty());
    assert_eq!(index.domain_count(), 0);
}

#[test]
fn test_query_time_range() {
    let mut index = HierarchicalIndex::new();
    let now = Utc::now();

    // Insert episodes at different times
    for i in 0..5 {
        let mut episode = create_test_episode("web-api", TaskType::CodeGeneration);
        episode.start_time = now - chrono::Duration::hours(i);
        index.insert(&episode);
    }

    // Query last 2 hours
    let start = now - chrono::Duration::hours(2);
    let results = index.query_by_time_range(start, now, 100);

    // Should find episodes from hours 0, 1, 2
    assert_eq!(results.len(), 3);
}

#[test]
fn test_memory_usage_estimate() {
    let mut index = HierarchicalIndex::new();

    let base_usage = index.memory_usage_estimate();

    for _ in 0..100 {
        index.insert(&create_test_episode("domain", TaskType::CodeGeneration));
    }

    let usage_with_data = index.memory_usage_estimate();

    // Memory usage should increase with data
    assert!(usage_with_data > base_usage);
}
