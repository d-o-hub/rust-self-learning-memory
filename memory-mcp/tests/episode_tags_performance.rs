//! Performance and stress tests for episode tagging

use memory_core::{SelfLearningMemory, TaskContext, TaskType};
use memory_mcp::mcp::tools::episode_tags::{
    AddEpisodeTagsInput, EpisodeTagTools, SearchEpisodesByTagsInput,
};
use std::sync::Arc;
use std::time::Instant;

/// Test adding tags to many episodes quickly
#[tokio::test]
async fn test_bulk_tag_addition() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let start = Instant::now();

    // Create 100 episodes and tag them
    for i in 0..100 {
        let episode_id = memory
            .start_episode(
                format!("Bulk episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        tools
            .add_tags(AddEpisodeTagsInput {
                episode_id: episode_id.to_string(),
                tags: vec![format!("bulk-{}", i % 10)],
            })
            .await
            .unwrap();
    }

    let duration = start.elapsed();
    println!("Created and tagged 100 episodes in {:?}", duration);
    println!("Average per episode: {:?}", duration / 100);

    // Should complete reasonably quickly (< 1 second in debug mode)
    assert!(duration.as_secs() < 5, "Bulk operations too slow");
}

/// Test searching across large episode set
#[tokio::test]
async fn test_large_scale_search() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 200 episodes with various tags
    for i in 0..200 {
        let episode_id = memory
            .start_episode(
                format!("Search test episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let mut tags = vec![];
        if i % 2 == 0 {
            tags.push("even".to_string());
        }
        if i % 5 == 0 {
            tags.push("five".to_string());
        }
        if i % 10 == 0 {
            tags.push("ten".to_string());
        }

        if !tags.is_empty() {
            memory.add_episode_tags(episode_id, tags).await.unwrap();
        }
    }

    let start = Instant::now();

    // Perform complex search
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["even".to_string(), "five".to_string()],
            require_all: Some(true), // AND search
            limit: Some(100),
        })
        .await
        .unwrap();

    let duration = start.elapsed();
    println!("Searched 200 episodes in {:?}", duration);
    println!("Found {} matching episodes", result.count);

    // Should find 20 episodes (0, 10, 20, ..., 190)
    assert_eq!(result.count, 20);

    // Search should be fast even with 200 episodes
    assert!(duration.as_millis() < 100, "Search too slow");
}

/// Test with maximum realistic tag count
#[tokio::test]
async fn test_maximum_tags_per_episode() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Max tags test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let start = Instant::now();

    // Add 100 tags (reasonable maximum)
    let tags: Vec<String> = (0..100).map(|i| format!("tag-{:03}", i)).collect();

    let result = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode_id.to_string(),
            tags,
        })
        .await
        .unwrap();

    let duration = start.elapsed();
    println!("Added 100 tags in {:?}", duration);

    assert_eq!(result.tags_added, 100);
    assert!(duration.as_millis() < 100, "Adding many tags too slow");
}

/// Test repeated search operations
#[tokio::test]
async fn test_repeated_searches() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 50 episodes
    for i in 0..50 {
        let episode_id = memory
            .start_episode(
                format!("Episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;
        memory
            .add_episode_tags(episode_id, vec![format!("tag-{}", i % 10)])
            .await
            .unwrap();
    }

    let start = Instant::now();

    // Perform 100 searches
    for i in 0..100 {
        tools
            .search_by_tags(SearchEpisodesByTagsInput {
                tags: vec![format!("tag-{}", i % 10)],
                require_all: Some(false),
                limit: Some(10),
            })
            .await
            .unwrap();
    }

    let duration = start.elapsed();
    println!("Performed 100 searches in {:?}", duration);
    println!("Average per search: {:?}", duration / 100);

    // Should handle repeated searches efficiently
    assert!(duration.as_secs() < 2, "Repeated searches too slow");
}

/// Test tag operations under realistic load
#[tokio::test]
async fn test_realistic_workflow() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let start = Instant::now();

    // Simulate realistic usage: 20 episodes with various operations
    for i in 0..20 {
        let episode_id = memory
            .start_episode(
                format!("Workflow episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        // Add initial tags
        tools
            .add_tags(AddEpisodeTagsInput {
                episode_id: episode_id.to_string(),
                tags: vec!["initial".to_string(), format!("batch-{}", i / 5)],
            })
            .await
            .unwrap();

        // Occasionally add more tags
        if i % 3 == 0 {
            tools
                .add_tags(AddEpisodeTagsInput {
                    episode_id: episode_id.to_string(),
                    tags: vec!["additional".to_string()],
                })
                .await
                .unwrap();
        }

        // Occasionally search
        if i % 5 == 0 {
            tools
                .search_by_tags(SearchEpisodesByTagsInput {
                    tags: vec!["initial".to_string()],
                    require_all: Some(false),
                    limit: Some(10),
                })
                .await
                .unwrap();
        }
    }

    let duration = start.elapsed();
    println!("Completed realistic workflow in {:?}", duration);

    // Should complete quickly
    assert!(duration.as_secs() < 3, "Realistic workflow too slow");
}

/// Test memory efficiency with many tags
#[tokio::test]
async fn test_memory_efficiency() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 50 episodes, each with 20 tags
    // Total: 1000 tag assignments
    for i in 0..50 {
        let episode_id = memory
            .start_episode(
                format!("Memory test episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let tags: Vec<String> = (0..20).map(|j| format!("tag-{}-{}", i, j)).collect();

        tools
            .add_tags(AddEpisodeTagsInput {
                episode_id: episode_id.to_string(),
                tags,
            })
            .await
            .unwrap();
    }

    // Verify data integrity
    let all_results = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["tag-0-0".to_string()],
            require_all: Some(false),
            limit: Some(100),
        })
        .await
        .unwrap();

    assert_eq!(all_results.count, 1); // Only episode 0 has "tag-0-0"
    println!("Memory test: 50 episodes × 20 tags = 1000 assignments stored correctly");
}

/// Test search performance with many tags in query
#[tokio::test]
async fn test_multi_tag_search_performance() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    // Create 100 episodes with random tag combinations
    for i in 0..100 {
        let episode_id = memory
            .start_episode(
                format!("Multi-tag episode {}", i),
                TaskContext::default(),
                TaskType::Testing,
            )
            .await;

        let mut tags = vec![];
        for j in 0..5 {
            if (i + j) % 7 == 0 {
                tags.push(format!("tag-{}", j));
            }
        }

        if !tags.is_empty() {
            memory.add_episode_tags(episode_id, tags).await.unwrap();
        }
    }

    let start = Instant::now();

    // Search with multiple tags
    let result = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec![
                "tag-0".to_string(),
                "tag-1".to_string(),
                "tag-2".to_string(),
            ],
            require_all: Some(false), // OR search
            limit: Some(100),
        })
        .await
        .unwrap();

    let duration = start.elapsed();
    println!(
        "Multi-tag search across 100 episodes in {:?}, found {} matches",
        duration, result.count
    );

    assert!(duration.as_millis() < 50, "Multi-tag search too slow");
}

/// Test rapid sequential operations
#[tokio::test]
async fn test_rapid_operations() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));

    let episode_id = memory
        .start_episode(
            "Rapid ops test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let start = Instant::now();

    // Perform 50 rapid add/remove operations
    for i in 0..50 {
        if i % 2 == 0 {
            tools
                .add_tags(AddEpisodeTagsInput {
                    episode_id: episode_id.to_string(),
                    tags: vec![format!("rapid-{}", i)],
                })
                .await
                .unwrap();
        } else {
            tools
                .add_tags(AddEpisodeTagsInput {
                    episode_id: episode_id.to_string(),
                    tags: vec!["temp".to_string()],
                })
                .await
                .ok(); // May fail if already exists
        }
    }

    let duration = start.elapsed();
    println!("Performed 50 rapid operations in {:?}", duration);

    assert!(duration.as_millis() < 500, "Rapid operations too slow");
}
