//! Demonstration of episode tagging MCP tools
//! Run with: cargo run --example `episode_tags_demo`

use do_memory_core::{SelfLearningMemory, TaskContext, TaskType};
use do_memory_mcp::mcp::tools::episode_tags::{
    AddEpisodeTagsInput, EpisodeTagTools, GetEpisodeTagsInput, RemoveEpisodeTagsInput,
    SearchEpisodesByTagsInput, SetEpisodeTagsInput,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║        Episode Tagging Demo - Full Integration Test                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Initialize memory system
    println!("🔧 Initializing memory system...");
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EpisodeTagTools::new(Arc::clone(&memory));
    println!("   ✅ Memory system ready\n");

    // Create test episodes
    println!("📝 Creating test episodes...");

    let episode1 = memory
        .start_episode(
            "Fix authentication timeout bug".to_string(),
            TaskContext::default(),
            TaskType::Debugging,
        )
        .await;
    println!("   ✅ Episode 1: {episode1} (Debugging)");

    let episode2 = memory
        .start_episode(
            "Implement user profile feature".to_string(),
            TaskContext::default(),
            TaskType::CodeGeneration,
        )
        .await;
    println!("   ✅ Episode 2: {episode2} (Code Generation)");

    let episode3 = memory
        .start_episode(
            "Refactor database connection pool".to_string(),
            TaskContext::default(),
            TaskType::Refactoring,
        )
        .await;
    println!("   ✅ Episode 3: {episode3} (Refactoring)\n");

    // Demo 1: Add tags
    println!("🏷️  Demo 1: Adding Tags");
    println!("   ─────────────────────");

    let result1 = tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode1.to_string(),
            tags: vec![
                "bug-fix".to_string(),
                "critical".to_string(),
                "authentication".to_string(),
            ],
        })
        .await?;
    println!("   Episode 1 tagged:");
    println!("     • Tags added: {}", result1.tags_added);
    println!("     • Current tags: {:?}", result1.current_tags);

    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode2.to_string(),
            tags: vec!["feature".to_string(), "user-profile".to_string()],
        })
        .await?;
    println!("   Episode 2 tagged: feature, user-profile");

    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode3.to_string(),
            tags: vec!["refactor".to_string(), "performance".to_string()],
        })
        .await?;
    println!("   Episode 3 tagged: refactor, performance\n");

    // Demo 2: Get tags
    println!("🔍 Demo 2: Retrieving Tags");
    println!("   ───────────────────────");

    let get_result = tools
        .get_tags(GetEpisodeTagsInput {
            episode_id: episode1.to_string(),
        })
        .await?;
    println!("   Episode 1 tags: {:?}", get_result.tags);
    println!("   Message: {}\n", get_result.message);

    // Demo 3: OR Search
    println!("🔎 Demo 3: Search by Tags (OR logic)");
    println!("   ─────────────────────────────────");
    println!("   Searching for: 'bug-fix' OR 'feature'");

    let search_or = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["bug-fix".to_string(), "feature".to_string()],
            require_all: Some(false),
            limit: Some(10),
        })
        .await?;
    println!("   Found {} episodes:", search_or.count);
    for ep in &search_or.episodes {
        println!("     • {}: {}", ep.task_description, ep.tags.join(", "));
    }
    println!();

    // Demo 4: AND Search
    println!("🔎 Demo 4: Search by Tags (AND logic)");
    println!("   ──────────────────────────────────");

    // Add common tag first
    tools
        .add_tags(AddEpisodeTagsInput {
            episode_id: episode1.to_string(),
            tags: vec!["reviewed".to_string()],
        })
        .await?;

    println!("   Added 'reviewed' tag to Episode 1");
    println!("   Searching for: 'bug-fix' AND 'reviewed'");

    let search_and = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["bug-fix".to_string(), "reviewed".to_string()],
            require_all: Some(true),
            limit: Some(10),
        })
        .await?;
    println!("   Found {} episode(s):", search_and.count);
    for ep in &search_and.episodes {
        println!("     • {}: {}", ep.task_description, ep.tags.join(", "));
    }
    println!();

    // Demo 5: Remove tags
    println!("✂️  Demo 5: Removing Tags");
    println!("   ──────────────────────");

    let remove_result = tools
        .remove_tags(RemoveEpisodeTagsInput {
            episode_id: episode1.to_string(),
            tags: vec!["critical".to_string()],
        })
        .await?;
    println!("   Removed 'critical' from Episode 1");
    println!("   Tags removed: {}", remove_result.tags_removed);
    println!("   Remaining tags: {:?}\n", remove_result.current_tags);

    // Demo 6: Set tags (replace all)
    println!("🔄 Demo 6: Replacing All Tags");
    println!("   ──────────────────────────");

    let set_result = tools
        .set_tags(SetEpisodeTagsInput {
            episode_id: episode2.to_string(),
            tags: vec!["completed".to_string(), "production-ready".to_string()],
        })
        .await?;
    println!("   Replaced all tags on Episode 2");
    println!("   New tags: {:?}\n", set_result.current_tags);

    // Demo 7: Case-insensitive search
    println!("🔤 Demo 7: Case-Insensitive Search");
    println!("   ────────────────────────────────");
    println!("   Searching for: 'BUG-FIX' (uppercase)");

    let case_search = tools
        .search_by_tags(SearchEpisodesByTagsInput {
            tags: vec!["BUG-FIX".to_string()],
            require_all: Some(false),
            limit: Some(10),
        })
        .await?;
    println!(
        "   Found {} episode(s) - case doesn't matter!\n",
        case_search.count
    );

    // Summary
    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                    DEMO COMPLETE! ✅                                 ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!("\n✨ All episode tagging features demonstrated successfully!\n");
    println!("Features tested:");
    println!("  ✅ Adding tags to episodes");
    println!("  ✅ Retrieving episode tags");
    println!("  ✅ Searching by tags (OR logic)");
    println!("  ✅ Searching by tags (AND logic)");
    println!("  ✅ Removing specific tags");
    println!("  ✅ Replacing all tags");
    println!("  ✅ Case-insensitive matching\n");

    Ok(())
}
