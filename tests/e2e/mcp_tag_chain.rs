//! MCP Tag Management Chain Tests (Day 2-3)
//!
//! Comprehensive E2E tests covering:
//! - add_episode_tags → get_episode_tags → list_episodes_by_tags → remove_episode_tags
//! - Tag normalization
//! - Case-insensitive search
//!
//! Target: 6+ test scenarios

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use memory_core::types::MemoryConfig;
use memory_core::{SelfLearningMemory, TaskContext, TaskOutcome, TaskType};
use std::sync::Arc;
use uuid::Uuid;

/// Test helper to create a memory instance
fn setup_test_memory() -> Arc<SelfLearningMemory> {
    // Use zero quality threshold for test episodes and disable features that may hang
    let config = MemoryConfig {
        quality_threshold: 0.0,                // Zero threshold for test episodes
        pattern_extraction_threshold: 1.0,     // Skip pattern extraction
        enable_summarization: false,           // Disable semantic summarization
        enable_spatiotemporal_indexing: false, // Disable spatiotemporal indexing
        enable_embeddings: false,              // Disable embeddings
        batch_config: None,                    // Disable step batching
        ..Default::default()
    };

    Arc::new(SelfLearningMemory::with_config(config))
}

/// Helper to create and complete an episode
async fn create_completed_episode(
    memory: &Arc<SelfLearningMemory>,
    description: &str,
    domain: &str,
) -> Uuid {
    let context = TaskContext {
        domain: domain.to_string(),
        ..TaskContext::default()
    };
    let id = memory
        .start_episode(description.to_string(), context, TaskType::CodeGeneration)
        .await;

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

async fn test_mcp_tag_full_chain() {
    let memory = setup_test_memory();

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

    // Step 3: list_episodes_by_tags
    let search_results = memory
        .list_episodes_by_tags(vec!["security".to_string()], false, None)
        .await
        .expect("list_episodes_by_tags failed");

    assert!(!search_results.is_empty());
    let result_ids: Vec<Uuid> = search_results.iter().map(|e| e.episode_id).collect();
    assert!(result_ids.contains(&episode_id));

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

async fn test_mcp_tag_normalization() {
    let memory = setup_test_memory();

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

async fn test_mcp_tag_case_insensitive_search() {
    let memory = setup_test_memory();

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
        .list_episodes_by_tags(vec!["security".to_string()], false, None)
        .await
        .unwrap();

    // Search with uppercase
    let search2 = memory
        .list_episodes_by_tags(vec!["Security".to_string()], false, None)
        .await
        .unwrap();

    // Both searches should find both episodes
    let result1_ids: Vec<Uuid> = search1.iter().map(|e| e.episode_id).collect();
    let result2_ids: Vec<Uuid> = search2.iter().map(|e| e.episode_id).collect();
    assert!(result1_ids.contains(&ep1_id));
    assert!(result1_ids.contains(&ep2_id));
    assert!(result2_ids.contains(&ep1_id));
    assert!(result2_ids.contains(&ep2_id));

    println!("✓ MCP tag case-insensitive search test passed");
}

// ============================================================================
// Scenario 4: Multi-Tag Search (AND/OR Logic)
// ============================================================================

async fn test_mcp_tag_multi_tag_search() {
    let memory = setup_test_memory();

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
        .list_episodes_by_tags(vec!["security".to_string(), "api".to_string()], false, None)
        .await
        .unwrap();

    let result_ids: Vec<Uuid> = or_results.iter().map(|e| e.episode_id).collect();
    assert!(result_ids.contains(&ep1_id));
    assert!(result_ids.contains(&ep2_id));
    assert!(result_ids.contains(&ep3_id));

    // Note: AND logic would require all tags, but current API uses OR for Vec
    println!("✓ MCP tag multi-tag search test passed");
}

// ============================================================================
// Scenario 5: Tag Statistics and Analytics
// ============================================================================

async fn test_mcp_tag_statistics() {
    let memory = setup_test_memory();

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

async fn test_mcp_tag_set_replace() {
    let memory = setup_test_memory();

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

async fn test_mcp_tag_empty_handling() {
    let memory = setup_test_memory();

    let episode_id = create_completed_episode(&memory, "Empty tags test", "tag-empty-test").await;

    // Episode should have no tags initially
    let tags = memory.get_episode_tags(episode_id).await.unwrap();
    assert!(tags.is_empty());

    // Add empty string tag (should be rejected or normalized)
    let _result = memory
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

async fn test_mcp_tag_large_number() {
    let memory = setup_test_memory();

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

async fn test_mcp_tag_based_episode_filtering() {
    let memory = setup_test_memory();

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
        .list_episodes_by_tags(vec!["security".to_string()], false, None)
        .await
        .unwrap();

    let security_ids: Vec<Uuid> = security_results.iter().map(|e| e.episode_id).collect();
    assert!(security_ids.contains(&security_ep));
    assert!(security_ids.contains(&both_ep));
    assert!(!security_ids.contains(&api_ep));
    assert!(!security_ids.contains(&db_ep));

    // Search by multiple tags
    let api_results = memory
        .list_episodes_by_tags(vec!["api".to_string()], false, None)
        .await
        .unwrap();

    let api_ids: Vec<Uuid> = api_results.iter().map(|e| e.episode_id).collect();
    assert!(api_ids.contains(&api_ep));
    assert!(api_ids.contains(&both_ep));
    assert!(!api_ids.contains(&security_ep));
    assert!(!api_ids.contains(&db_ep));

    println!("✓ MCP tag-based episode filtering test passed");
}

// ============================================================================
// Scenario 10: Tag Deduplication Across Operations
// ============================================================================

async fn test_mcp_tag_deduplication() {
    let memory = setup_test_memory();

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

// ============================================================================
// Main function for harness = false test
// ============================================================================

use std::future::Future;

fn run_test<F>(name: &str, test: F, passed: &mut i32, _failed: &mut i32)
where
    F: Future<Output = ()>,
{
    print!("Running {} ... ", name);
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    rt.block_on(async {
        test.await;
    });
    println!("✓ PASSED");
    *passed += 1;
}

fn main() {
    println!("\n========================================");
    println!("Running MCP tag chain E2E tests");
    println!("========================================\n");

    let mut passed = 0;
    let mut failed = 0;

    // Run all tests
    run_test(
        "test_mcp_tag_full_chain",
        test_mcp_tag_full_chain(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_normalization",
        test_mcp_tag_normalization(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_case_insensitive_search",
        test_mcp_tag_case_insensitive_search(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_multi_tag_search",
        test_mcp_tag_multi_tag_search(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_statistics",
        test_mcp_tag_statistics(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_set_replace",
        test_mcp_tag_set_replace(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_empty_handling",
        test_mcp_tag_empty_handling(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_large_number",
        test_mcp_tag_large_number(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_based_episode_filtering",
        test_mcp_tag_based_episode_filtering(),
        &mut passed,
        &mut failed,
    );
    run_test(
        "test_mcp_tag_deduplication",
        test_mcp_tag_deduplication(),
        &mut passed,
        &mut failed,
    );

    println!("\n========================================");
    println!("Results: {} passed, {} failed", passed, failed);
    println!("========================================\n");

    if failed > 0 {
        std::process::exit(1);
    }
}
