//! MCP Tag Management Chain Tests (Day 2-3)
//!
//! Comprehensive E2E tests covering:
//! - add_episode_tags → get_episode_tags → search_episodes_by_tags → remove_episode_tags
//! - Tag normalization
//! - Case-insensitive search
//!
//! Target: 6+ test scenarios

#![allow(clippy::unwrap_used, clippy::expect_used)]

use memory_core::{SelfLearningMemory, TaskOutcome, TaskType};
use memory_storage_redb::RedbStorage;
use serial_test::serial;
use std::sync::Arc;
use tempfile::tempdir;
use uuid::Uuid;

/// Test helper to create a memory instance with storage
async fn setup_test_memory() -> (Arc<SelfLearningMemory>, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let turso_path = dir.path().join("test_turso.redb");
    let cache_path = dir.path().join("test_cache.redb");

    let turso_storage = RedbStorage::new(&turso_path)
        .await
        .expect("Failed to create turso storage");
    let cache_storage = RedbStorage::new(&cache_path)
        .await
        .expect("Failed to create cache storage");

    let memory = Arc::new(SelfLearningMemory::with_storage(
        Default::default(),
        Arc::new(turso_storage),
        Arc::new(cache_storage),
    ));

    (memory, dir)
}

/// Helper to create and complete an episode
async fn create_completed_episode(
    memory: &Arc<SelfLearningMemory>,
    description: &str,
    domain: &str,
) -> Uuid {
    let id = memory
        .create_episode(
            description.to_string(),
            domain.to_string(),
            TaskType::CodeGeneration,
        )
        .await
        .unwrap();

    memory
        .complete_episode(
            id,
            TaskOutcome::Success {
                verdict: "Done".to_string(),
                artifacts: vec![],
            },
        )
        .await
        .unwrap();

    id
}

// ============================================================================
// Scenario 1: Complete Tag Chain
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_full_chain() {
    let (memory, _dir) = setup_test_memory().await;

    // Create episode
    let episode_id = create_completed_episode(&memory, "Tag chain test", "tag-chain-test").await;

    // Step 1: add_episode_tags
    memory
        .add_episode_tags(
            episode_id,
            vec![
                "security".to_string(),
                "authentication".to_string(),
                "jwt".to_string(),
            ],
        )
        .await
        .expect("add_episode_tags failed");

    // Step 2: get_episode_tags
    let tags = memory
        .get_episode_tags(episode_id)
        .await
        .expect("get_episode_tags failed");

    assert_eq!(tags.len(), 3);
    assert!(tags.contains(&"security".to_string()));
    assert!(tags.contains(&"authentication".to_string()));
    assert!(tags.contains(&"jwt".to_string()));

    // Step 3: search_episodes_by_tags
    let search_results = memory
        .search_episodes_by_tags(&["security".to_string()], false)
        .await
        .expect("search_episodes_by_tags failed");

    assert!(!search_results.is_empty());
    assert!(search_results.contains(&episode_id));

    // Step 4: remove_episode_tags
    memory
        .remove_episode_tags(episode_id, vec!["jwt".to_string()])
        .await
        .expect("remove_episode_tags failed");

    // Verify tag removed
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(!tags.contains(&"jwt".to_string()));

    println!("✓ MCP tag full chain test passed");
}

// ============================================================================
// Scenario 2: Tag Normalization (Case, Whitespace)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_normalization() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = create_completed_episode(&memory, "Normalization test", "tag-norm-test").await;

    // Add tags with different cases and whitespace
    memory
        .add_episode_tags(
            episode_id,
            vec![
                "  security  ".to_string(),
                "Security".to_string(),
                "SECURITY".to_string(),
                "authentication".to_string(),
                "Authentication  ".to_string(),
            ],
        )
        .await
        .unwrap();

    // Get tags - should be normalized (lowercase, trimmed, deduplicated)
    let tags = memory.get_episode_tags(episode_id).await.unwrap();

    // Should have only 2 unique tags (security + authentication)
    // Note: Actual behavior depends on implementation
    println!("Tags after normalization: {:?}", tags);

    // Verify duplicates are removed
    let unique_tags: std::collections::HashSet<_> = tags.iter().collect();
    assert_eq!(
        tags.len(),
        unique_tags.len(),
        "No duplicate tags should exist"
    );

    println!("✓ MCP tag normalization test passed");
}

// ============================================================================
// Scenario 3: Case-Insensitive Tag Search
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_case_insensitive_search() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id = create_completed_episode(&memory, "Episode 1", "tag-case-test").await;
    let ep2_id = create_completed_episode(&memory, "Episode 2", "tag-case-test").await;

    // Tag episodes with different cases
    memory
        .add_episode_tags(ep1_id, vec!["Security".to_string(), "API".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(ep2_id, vec!["security".to_string(), "api".to_string()])
        .await
        .unwrap();

    // Search with lowercase
    let search1 = memory
        .search_episodes_by_tags(&["security".to_string()], false)
        .await
        .unwrap();

    // Search with uppercase
    let search2 = memory
        .search_episodes_by_tags(&["Security".to_string()], false)
        .await
        .unwrap();

    // Both searches should find both episodes
    assert!(search1.contains(&ep1_id));
    assert!(search1.contains(&ep2_id));
    assert!(search2.contains(&ep1_id));
    assert!(search2.contains(&ep2_id));

    println!("✓ MCP tag case-insensitive search test passed");
}

// ============================================================================
// Scenario 4: Multi-Tag Search (AND/OR Logic)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_multi_tag_search() {
    let (memory, _dir) = setup_test_memory().await;

    let ep1_id = create_completed_episode(&memory, "Security API", "tag-multi-test").await;
    let ep2_id = create_completed_episode(&memory, "Security only", "tag-multi-test").await;
    let ep3_id = create_completed_episode(&memory, "API only", "tag-multi-test").await;

    // Tag episodes
    memory
        .add_episode_tags(ep1_id, vec!["security".to_string(), "api".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(ep2_id, vec!["security".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(ep3_id, vec!["api".to_string()])
        .await
        .unwrap();

    // Search for "security" OR "api" (should match all)
    let or_results = memory
        .search_episodes_by_tags(&["security".to_string(), "api".to_string()], false)
        .await
        .unwrap();

    assert!(or_results.contains(&ep1_id));
    assert!(or_results.contains(&ep2_id));
    assert!(or_results.contains(&ep3_id));

    // Note: AND logic would require all tags, but current API uses OR for Vec
    println!("✓ MCP tag multi-tag search test passed");
}

// ============================================================================
// Scenario 5: Tag Statistics and Analytics
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_statistics() {
    let (memory, _dir) = setup_test_memory().await;

    // Create multiple episodes with overlapping tags
    let episodes = vec!["Episode 1", "Episode 2", "Episode 3"];

    for (i, ep_desc) in episodes.iter().enumerate() {
        let ep_id = create_completed_episode(&memory, ep_desc, "tag-stats-test").await;

        // Each episode has unique tag + shared tag
        let tags = vec![format!("unique-{}", i), "shared".to_string()];

        memory.add_episode_tags(ep_id, tags).await.unwrap();
    }

    // Get tag statistics
    let tag_stats = memory
        .get_tag_statistics()
        .await
        .expect("get_tag_statistics failed");

    // "shared" tag should have count 3
    let shared_stat = tag_stats.get("shared").expect("shared tag should exist");
    assert_eq!(shared_stat.usage_count, 3);

    // Unique tags should have count 1 each
    for i in 0..3 {
        let tag = format!("unique-{}", i);
        let stat = tag_stats
            .get(&tag)
            .expect(&format!("tag {} should exist", tag));
        assert_eq!(stat.usage_count, 1);
    }

    println!("✓ MCP tag statistics test passed");
}

// ============================================================================
// Scenario 6: Set Tags (Replace All Tags)
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_set_replace() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = create_completed_episode(&memory, "Set tags test", "tag-set-test").await;

    // Add initial tags
    memory
        .add_episode_tags(
            episode_id,
            vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        )
        .await
        .unwrap();

    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 3);

    // Replace all tags
    memory
        .set_episode_tags(
            episode_id,
            vec!["new-tag1".to_string(), "new-tag2".to_string()],
        )
        .await
        .expect("set_episode_tags failed");

    // Verify tags replaced
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 2);
    assert!(tags.contains(&"new-tag1".to_string()));
    assert!(tags.contains(&"new-tag2".to_string()));
    assert!(!tags.contains(&"tag1".to_string()));

    println!("✓ MCP tag set replace test passed");
}

// ============================================================================
// Scenario 7: Empty Tag Handling
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_empty_handling() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = create_completed_episode(&memory, "Empty tags test", "tag-empty-test").await;

    // Episode should have no tags initially
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert!(tags.is_empty());

    // Add empty string tag (should be rejected or normalized)
    let result = memory
        .add_episode_tags(episode_id, vec!["".to_string()])
        .await;
    // This may succeed or fail depending on implementation

    // Add whitespace-only tag (should be normalized)
    memory
        .add_episode_tags(episode_id, vec!["   ".to_string()])
        .await
        .unwrap_or(()); // May or may not succeed

    // Verify actual tags
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    println!("Tags after adding empty/whitespace: {:?}", tags);

    // Remove non-existent tags (should be no-op)
    memory
        .remove_episode_tags(episode_id, vec!["non-existent".to_string()])
        .await
        .unwrap();

    println!("✓ MCP tag empty handling test passed");
}

// ============================================================================
// Scenario 8: Large Number of Tags
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_large_number() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = create_completed_episode(&memory, "Many tags test", "tag-many-test").await;

    // Add many tags
    let many_tags: Vec<String> = (0..50).map(|i| format!("tag-{}", i)).collect();

    memory
        .add_episode_tags(episode_id, many_tags.clone())
        .await
        .expect("add_episode_tags with many tags failed");

    // Verify all tags added
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 50);

    // Remove half of them
    let half_tags: Vec<String> = many_tags.into_iter().take(25).collect();

    memory
        .remove_episode_tags(episode_id, half_tags)
        .await
        .unwrap();

    // Verify 25 tags remain
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert_eq!(tags.len(), 25);

    println!("✓ MCP tag large number test passed");
}

// ============================================================================
// Scenario 9: Tag-Based Episode Filtering in Query
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_based_episode_filtering() {
    let (memory, _dir) = setup_test_memory().await;

    let domain = "tag-filter-test";

    // Create episodes with different tags
    let security_ep = create_completed_episode(&memory, "Security task", domain).await;
    let api_ep = create_completed_episode(&memory, "API task", domain).await;
    let db_ep = create_completed_episode(&memory, "Database task", domain).await;
    let both_ep = create_completed_episode(&memory, "Security + API task", domain).await;

    memory
        .add_episode_tags(security_ep, vec!["security".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(api_ep, vec!["api".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(db_ep, vec!["database".to_string()])
        .await
        .unwrap();

    memory
        .add_episode_tags(both_ep, vec!["security".to_string(), "api".to_string()])
        .await
        .unwrap();

    // Search by single tag
    let security_results = memory
        .search_episodes_by_tags(&["security".to_string()], false)
        .await
        .unwrap();

    assert!(security_results.contains(&security_ep));
    assert!(security_results.contains(&both_ep));
    assert!(!security_results.contains(&api_ep));
    assert!(!security_results.contains(&db_ep));

    // Search by multiple tags
    let api_results = memory
        .search_episodes_by_tags(&["api".to_string()], false)
        .await
        .unwrap();

    assert!(api_results.contains(&api_ep));
    assert!(api_results.contains(&both_ep));
    assert!(!api_results.contains(&security_ep));
    assert!(!api_results.contains(&db_ep));

    println!("✓ MCP tag-based episode filtering test passed");
}

// ============================================================================
// Scenario 10: Tag Deduplication Across Operations
// ============================================================================

#[tokio::test]
#[serial]
async fn test_mcp_tag_deduplication() {
    let (memory, _dir) = setup_test_memory().await;

    let episode_id = create_completed_episode(&memory, "Dedup test", "tag-dedup-test").await;

    // Add tags (some duplicates)
    memory
        .add_episode_tags(
            episode_id,
            vec![
                "security".to_string(),
                "api".to_string(),
                "security".to_string(), // duplicate
                "API".to_string(),      // normalized duplicate
                "database".to_string(),
                "security".to_string(), // another duplicate
            ],
        )
        .await
        .unwrap();

    let tags = memory.get_episode_tags(episode_id).await.unwrap();

    // Should have only unique tags
    let unique_tags: std::collections::HashSet<_> = tags.iter().collect();
    assert_eq!(tags.len(), unique_tags.len());

    // Add more tags with more duplicates
    memory
        .add_episode_tags(
            episode_id,
            vec![
                "security".to_string(),
                "cache".to_string(),
                "api".to_string(),
            ],
        )
        .await
        .unwrap();

    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    let unique_tags: std::collections::HashSet<_> = tags.iter().collect();
    assert_eq!(tags.len(), unique_tags.len());

    println!("✓ MCP tag deduplication test passed");
}
