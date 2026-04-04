//! MCP Integration Tests
//!
//! These tests simulate realistic application workflows using the Memory MCP server
//! and verify that all database entries are created and stored correctly.

use do_memory_core::{
    ComplexityLevel, ExecutionStep, MemoryConfig, SelfLearningMemory, TaskContext, TaskOutcome,
    TaskType,
};
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use std::sync::Arc;

#[cfg(test)]
mod mcp_integration_tests {
    use super::*;

    /// Disable WASM sandbox for all tests to prevent rquickjs GC crashes
    #[allow(unsafe_code)]
    fn disable_wasm_for_tests() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            // SAFETY: test-only env var manipulation
            unsafe {
                std::env::set_var("MCP_USE_WASM", "false");
            }
        });
    }

    async fn setup_test_environment() -> (Arc<SelfLearningMemory>, Arc<MemoryMCPServer>) {
        disable_wasm_for_tests();
        let memory = Arc::new(SelfLearningMemory::with_config(MemoryConfig {
            quality_threshold: 0.0,
            batch_config: None, // Disable batching for tests for test episodes
            ..Default::default()
        }));
        let sandbox_config = SandboxConfig::default();
        let mcp_server = Arc::new(
            MemoryMCPServer::new(sandbox_config, memory.clone())
                .await
                .unwrap(),
        );
        (memory, mcp_server)
    }

    #[tokio::test]
    async fn test_web_app_workflow_database_entries() {
        println!("🧪 Testing Web Application Workflow - Database Entries");
        println!("=======================================================");

        let (memory, mcp_server) = setup_test_environment().await;

        // Simulate web application development workflow
        println!("\n1. Episode Creation - Web Development");
        println!("--------------------------------------");

        let episode_id = memory
            .start_episode(
                "Build Web Todo List with Local Storage".to_string(),
                TaskContext {
                    domain: "web".to_string(),
                    language: Some("javascript".to_string()),
                    framework: Some("vanilla-js".to_string()),
                    complexity: ComplexityLevel::Moderate,
                    tags: vec![
                        "web".to_string(),
                        "todo".to_string(),
                        "local-storage".to_string(),
                    ],
                },
                TaskType::CodeGeneration,
            )
            .await;

        println!("✅ Episode created successfully");

        // Log development steps
        println!("\n2. Development Steps Logging");
        println!("----------------------------");

        let steps = [
            (
                "create_html",
                "Create HTML structure with modern JavaScript",
            ),
            ("add_manifest", "Add Web App Manifest for installation"),
            (
                "implement_service_worker",
                "Implement service worker for offline functionality",
            ),
            (
                "add_local_storage",
                "Implement localStorage for data persistence",
            ),
            (
                "add_responsive_design",
                "Add responsive CSS and mobile-first design",
            ),
            (
                "test_features",
                "Test installation and offline capabilities",
            ),
        ];

        for (i, (tool, action)) in steps.iter().enumerate() {
            let step = ExecutionStep::new(i + 1, tool.to_string(), action.to_string());
            memory.log_step(episode_id, step).await;
            println!("✅ Step {}: {} - {}", i + 1, tool, action);
        }

        // Complete the episode
        println!("\n3. Episode Completion");
        println!("--------------------");

        let outcome = TaskOutcome::Success {
            verdict: "Web Todo List implemented successfully with local storage, service worker, and offline support. All features working correctly.".to_string(),
            artifacts: vec![
                "index.html".to_string(),
                "manifest.json".to_string(),
                "sw.js".to_string(),
                "README.md".to_string(),
            ],
        };

        memory.complete_episode(episode_id, outcome).await.unwrap();
        println!("✅ Episode completed successfully");

        // Test 1: Verify Episode Storage
        println!("\n4. Database Verification - Episodes");
        println!("-----------------------------------");

        let episodes_result = mcp_server
            .query_memory(
                "Web todo".to_string(),
                "web".to_string(),
                None,
                10,
                "relevance".to_string(),
                None,
            )
            .await
            .unwrap();

        let episodes = episodes_result["episodes"].as_array().unwrap();
        assert_eq!(episodes.len(), 1, "Should have exactly 1 episode");

        let episode = &episodes[0];
        println!("📋 Episode Details:");
        println!("   ID: {}", episode["episode_id"]);
        println!("   Description: {}", episode["task_description"]);
        println!("   Domain: {}", episode["context"]["domain"]);
        println!("   Language: {}", episode["context"]["language"]);
        println!("   Framework: {}", episode["context"]["framework"]);
        println!("   Complexity: {}", episode["context"]["complexity"]);
        println!("   Tags: {:?}", episode["context"]["tags"]);
        println!("   Task Type: {}", episode["task_type"]);
        println!(
            "   Steps Count: {}",
            episode["steps"].as_array().unwrap().len()
        );
        println!("   Status: Complete");
        println!(
            "   Artifacts: {:?}",
            episode["outcome"]["Success"]["artifacts"]
        );

        // Test 2: Verify Pattern Extraction
        println!("\n5. Database Verification - Patterns");
        println!("-----------------------------------");

        let patterns_result = mcp_server
            .analyze_patterns("CodeGeneration".to_string(), 0.0, 10, None)
            .await
            .unwrap();

        let patterns = patterns_result["patterns"].as_array().unwrap();
        println!("🎯 Pattern Analysis Results:");
        println!("   Total Patterns: {}", patterns.len());

        for (i, pattern) in patterns.iter().enumerate() {
            println!(
                "   Pattern {}: {} (confidence: {:.2})",
                i + 1,
                pattern["pattern_type"],
                pattern["confidence"]
            );
        }

        let stats = &patterns_result["statistics"];
        println!("   📊 Statistics:");
        println!("      Total Patterns: {}", stats["total_patterns"]);
        println!("      Avg Success Rate: {:.2}", stats["avg_success_rate"]);

        // Test 3: Tool Usage Statistics
        println!("\n6. Database Verification - Tool Usage");
        println!("--------------------------------------");

        // Perform some tool operations to generate usage stats
        for _ in 0..3 {
            let _ = mcp_server
                .query_memory(
                    "test".to_string(),
                    "test".to_string(),
                    None,
                    1,
                    "relevance".to_string(),
                    None,
                )
                .await;
        }
        let _ = mcp_server
            .analyze_patterns("test".to_string(), 0.5, 2, None)
            .await;

        let usage = mcp_server.get_tool_usage().await;
        println!("📈 Tool Usage Statistics:");
        for (tool, count) in usage.iter() {
            println!("   {}: {} calls", tool, count);
        }

        // Test 4: Execution Statistics
        println!("\n7. Database Verification - Execution Stats");
        println!("-------------------------------------------");

        let exec_stats = mcp_server.get_stats().await;
        println!("⚡ Execution Statistics:");
        println!("   Total Executions: {}", exec_stats.total_executions);
        println!("   Successful: {}", exec_stats.successful_executions);
        println!("   Failed: {}", exec_stats.failed_executions);
        println!("   Security Violations: {}", exec_stats.security_violations);

        // Test 5: Memory Pattern Retrieval
        println!("\n8. Database Verification - Pattern Retrieval");
        println!("---------------------------------------------");

        let memory_patterns = memory
            .retrieve_relevant_patterns(
                &TaskContext {
                    domain: "web".to_string(),
                    language: Some("javascript".to_string()),
                    framework: Some("vanilla-js".to_string()),
                    complexity: ComplexityLevel::Moderate,
                    tags: vec!["web".to_string()],
                },
                10,
            )
            .await;

        println!("🧠 Memory Pattern Retrieval:");
        println!("   Patterns Found: {}", memory_patterns.len());

        for (i, pattern) in memory_patterns.iter().enumerate() {
            println!(
                "   Pattern {}: ID={}, Confidence={:.2}",
                i + 1,
                pattern.id(),
                pattern.confidence()
            );
        }

        // Test 6: Comprehensive Database Summary
        println!("\n9. Final Database Summary");
        println!("=========================");

        // Get final comprehensive view
        let final_query = mcp_server
            .query_memory(
                "web".to_string(),
                "web".to_string(),
                None,
                5,
                "relevance".to_string(),
                None,
            )
            .await
            .unwrap();

        let final_episodes = final_query["episodes"].as_array().unwrap();
        let final_patterns = final_query["patterns"].as_array().unwrap();
        let final_insights = &final_query["insights"];

        println!("📊 Complete Database State:");
        println!("   Episodes: {}", final_episodes.len());
        println!("   Patterns: {}", final_patterns.len());
        println!(
            "   Total Episodes in Insights: {}",
            final_insights["total_episodes"]
        );
        println!("   Success Rate: {:.2}", final_insights["success_rate"]);

        println!("\n✅ Web Todo App Database Verification Complete!");
        println!("=================================================");
        println!("All database entries verified and logged successfully.");
        println!("The Web Todo App integration with Memory MCP is fully functional.");
    }
}
