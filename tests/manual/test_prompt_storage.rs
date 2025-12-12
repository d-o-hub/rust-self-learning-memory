#!/usr/bin/env cargo script

//! This script tests memory-mcp prompt storage and retrieval from both Turso and redb backends
//! 
//! Usage: cargo script test_prompt_storage.rs

use std::sync::Arc;
use memory_core::{SelfLearningMemory, TaskContext, ComplexityLevel};
use memory_storage_turso::{TursoConfig, TursoStorage};
use memory_storage_redb::{CacheConfig, RedbStorage};
use serde_json::json;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("🧪 Testing Memory-MCP Prompt Storage and Retrieval");
    println!("==================================================");

    // Test data
    let test_prompt = "use the memory-mcp and verify in the turso and redb that the current prompt is write and read correctly";
    let test_domain = "testing";
    let test_task_type = "verification";

    // Test 1: Turso backend
    println!("\n📊 Testing Turso Backend...");
    match test_turso_backend(test_prompt, test_domain, test_task_type).await {
        Ok(_) => println!("✅ Turso backend test passed"),
        Err(e) => println!("❌ Turso backend test failed: {}", e),
    }

    // Test 2: redb backend
    println!("\n🔴 Testing redb Backend...");
    match test_redb_backend(test_prompt, test_domain, test_task_type).await {
        Ok(_) => println!("✅ redb backend test passed"),
        Err(e) => println!("❌ redb backend test failed: {}", e),
    }

    // Test 3: Dual storage (Turso + redb)
    println!("\n🔄 Testing Dual Storage (Turso + redb)...");
    match test_dual_storage(test_prompt, test_domain, test_task_type).await {
        Ok(_) => println!("✅ Dual storage test passed"),
        Err(e) => println!("❌ Dual storage test failed: {}", e),
    }

    println!("\n🎯 All tests completed!");
    Ok(())
}

async fn test_turso_backend(prompt: &str, domain: &str, task_type: &str) -> anyhow::Result<()> {
    println!("  📝 Setting up Turso storage...");
    
    // Use local Turso database for testing
    let turso_url = "file:./test_turso_memory.db";
    let turso_token = "";
    
    // Clean up any existing test database
    if std::path::Path::new("./test_turso_memory.db").exists() {
        std::fs::remove_file("./test_turso_memory.db")?;
    }
    
    let turso_config = TursoConfig {
        max_retries: 1,
        retry_base_delay_ms: 50,
        retry_max_delay_ms: 1000,
        enable_pooling: false,
    };

    let turso_storage = TursoStorage::with_config(turso_url, turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    println!("  💾 Creating memory system with Turso backend...");
    let memory_config = memory_core::MemoryConfig::default();
    let turso_arc: Arc<dyn memory_core::StorageBackend> = Arc::new(turso_storage);
    let memory = SelfLearningMemory::with_storage(memory_config, Arc::clone(&turso_arc), Arc::clone(&turso_arc));

    println!("  🎯 Creating episode with test prompt...");
    let task_context = TaskContext {
        domain: domain.to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![task_type.to_string(), "prompt-storage".to_string()],
    };

    let episode_id = memory.start_episode(prompt.to_string(), task_context.clone()).await?;
    println!("    ✅ Episode created with ID: {}", episode_id);

    // Add some execution steps
    memory.log_step(episode_id, memory_core::ExecutionStep {
        tool: "memory-mcp".to_string(),
        action: "store-prompt".to_string(),
        input: json!({"prompt": prompt}),
        output: Some(json!({"status": "stored"})),
        latency_ms: 10,
        success: true,
        error: None,
    }).await?;

    // Complete the episode
    memory.complete_episode(episode_id, memory_core::TaskOutcome {
        verdict: memory_core::Verdict::Success,
        reward: Some(memory_core::Reward {
            total: 0.9,
            components: vec![("correctness".to_string(), 0.9)],
        }),
        summary: Some("Prompt storage test completed successfully".to_string()),
        lessons_learned: vec!["Turso backend stores prompts correctly".to_string()],
        next_steps: vec![],
    }).await?;

    println!("  🔍 Retrieving stored prompt...");
    sleep(Duration::from_millis(100)).await; // Allow for storage to settle

    let retrieved_episodes = memory.retrieve_relevant_context(
        prompt.to_string(),
        task_context.clone(),
        5,
    ).await?;

    if retrieved_episodes.is_empty() {
        anyhow::bail!("No episodes retrieved from Turso");
    }

    let found_prompt = retrieved_episodes.iter().any(|ep| {
        ep.description.contains(prompt) || 
        ep.steps.iter().any(|step| {
            step.input.as_ref()
                .and_then(|i| i.get("prompt"))
                .and_then(|p| p.as_str())
                .map(|p| p.contains(prompt))
                .unwrap_or(false)
        })
    });

    if !found_prompt {
        anyhow::bail!("Test prompt not found in retrieved episodes");
    }

    println!("    ✅ Prompt successfully retrieved from Turso");
    println!("    📊 Retrieved {} episodes", retrieved_episodes.len());

    // Cleanup
    drop(memory);
    std::fs::remove_file("./test_turso_memory.db")?;
    
    Ok(())
}

async fn test_redb_backend(prompt: &str, domain: &str, task_type: &str) -> anyhow::Result<()> {
    println!("  📝 Setting up redb storage...");
    
    let cache_path = "./test_redb_cache.redb";
    
    // Clean up any existing test database
    if std::path::Path::new(cache_path).exists() {
        std::fs::remove_file(cache_path)?;
    }
    
    let cache_config = CacheConfig {
        max_size: 100,
        default_ttl_secs: 1800,
        cleanup_interval_secs: 600,
        enable_background_cleanup: false, // Disable for testing
    };

    let redb_storage = RedbStorage::new_with_cache_config(std::path::Path::new(cache_path), cache_config).await?;

    println!("  💾 Creating memory system with redb backend...");
    let memory_config = memory_core::MemoryConfig::default();
    let redb_arc: Arc<dyn memory_core::StorageBackend> = Arc::new(redb_storage);
    let memory = SelfLearningMemory::with_storage(memory_config, Arc::clone(&redb_arc), redb_arc);

    println!("  🎯 Creating episode with test prompt...");
    let task_context = TaskContext {
        domain: domain.to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![task_type.to_string(), "prompt-storage".to_string()],
    };

    let episode_id = memory.start_episode(prompt.to_string(), task_context.clone()).await?;
    println!("    ✅ Episode created with ID: {}", episode_id);

    // Add some execution steps
    memory.log_step(episode_id, memory_core::ExecutionStep {
        tool: "memory-mcp".to_string(),
        action: "store-prompt".to_string(),
        input: json!({"prompt": prompt}),
        output: Some(json!({"status": "stored"})),
        latency_ms: 10,
        success: true,
        error: None,
    }).await?;

    // Complete the episode
    memory.complete_episode(episode_id, memory_core::TaskOutcome {
        verdict: memory_core::Verdict::Success,
        reward: Some(memory_core::Reward {
            total: 0.9,
            components: vec![("correctness".to_string(), 0.9)],
        }),
        summary: Some("Prompt storage test completed successfully".to_string()),
        lessons_learned: vec!["redb backend stores prompts correctly".to_string()],
        next_steps: vec![],
    }).await?;

    println!("  🔍 Retrieving stored prompt...");
    sleep(Duration::from_millis(100)).await; // Allow for storage to settle

    let retrieved_episodes = memory.retrieve_relevant_context(
        prompt.to_string(),
        task_context.clone(),
        5,
    ).await?;

    if retrieved_episodes.is_empty() {
        anyhow::bail!("No episodes retrieved from redb");
    }

    let found_prompt = retrieved_episodes.iter().any(|ep| {
        ep.description.contains(prompt) || 
        ep.steps.iter().any(|step| {
            step.input.as_ref()
                .and_then(|i| i.get("prompt"))
                .and_then(|p| p.as_str())
                .map(|p| p.contains(prompt))
                .unwrap_or(false)
        })
    });

    if !found_prompt {
        anyhow::bail!("Test prompt not found in retrieved episodes");
    }

    println!("    ✅ Prompt successfully retrieved from redb");
    println!("    📊 Retrieved {} episodes", retrieved_episodes.len());

    // Cleanup
    drop(memory);
    std::fs::remove_file(cache_path)?;
    
    Ok(())
}

async fn test_dual_storage(prompt: &str, domain: &str, task_type: &str) -> anyhow::Result<()> {
    println!("  📝 Setting up dual storage (Turso + redb)...");
    
    // Setup Turso
    let turso_url = "file:./test_dual_turso_memory.db";
    let turso_token = "";
    
    // Clean up any existing test databases
    if std::path::Path::new("./test_dual_turso_memory.db").exists() {
        std::fs::remove_file("./test_dual_turso_memory.db")?;
    }
    
    let turso_config = TursoConfig {
        max_retries: 1,
        retry_base_delay_ms: 50,
        retry_max_delay_ms: 1000,
        enable_pooling: false,
    };

    let turso_storage = TursoStorage::with_config(turso_url, turso_token, turso_config).await?;
    turso_storage.initialize_schema().await?;

    // Setup redb
    let cache_path = "./test_dual_redb_cache.redb";
    if std::path::Path::new(cache_path).exists() {
        std::fs::remove_file(cache_path)?;
    }
    
    let cache_config = CacheConfig {
        max_size: 100,
        default_ttl_secs: 1800,
        cleanup_interval_secs: 600,
        enable_background_cleanup: false,
    };

    let redb_storage = RedbStorage::new_with_cache_config(std::path::Path::new(cache_path), cache_config).await?;

    println!("  💾 Creating memory system with dual storage...");
    let memory_config = memory_core::MemoryConfig::default();
    let memory = SelfLearningMemory::with_storage(
        memory_config,
        Arc::new(turso_storage),
        Arc::new(redb_storage),
    );

    println!("  🎯 Creating episode with test prompt...");
    let task_context = TaskContext {
        domain: domain.to_string(),
        language: Some("rust".to_string()),
        framework: None,
        complexity: ComplexityLevel::Moderate,
        tags: vec![task_type.to_string(), "prompt-storage".to_string()],
    };

    let episode_id = memory.start_episode(prompt.to_string(), task_context.clone()).await?;
    println!("    ✅ Episode created with ID: {}", episode_id);

    // Add some execution steps
    memory.log_step(episode_id, memory_core::ExecutionStep {
        tool: "memory-mcp".to_string(),
        action: "store-prompt".to_string(),
        input: json!({"prompt": prompt}),
        output: Some(json!({"status": "stored", "backend": "dual"})),
        latency_ms: 10,
        success: true,
        error: None,
    }).await?;

    // Complete the episode
    memory.complete_episode(episode_id, memory_core::TaskOutcome {
        verdict: memory_core::Verdict::Success,
        reward: Some(memory_core::Reward {
            total: 0.95,
            components: vec![("correctness".to_string(), 0.9), ("persistence".to_string(), 0.05)],
        }),
        summary: Some("Dual storage prompt test completed successfully".to_string()),
        lessons_learned: vec!["Dual storage (Turso + redb) stores prompts correctly".to_string()],
        next_steps: vec![],
    }).await?;

    println!("  🔍 Retrieving stored prompt...");
    sleep(Duration::from_millis(100)).await; // Allow for storage to settle

    let retrieved_episodes = memory.retrieve_relevant_context(
        prompt.to_string(),
        task_context.clone(),
        5,
    ).await?;

    if retrieved_episodes.is_empty() {
        anyhow::bail!("No episodes retrieved from dual storage");
    }

    let found_prompt = retrieved_episodes.iter().any(|ep| {
        ep.description.contains(prompt) || 
        ep.steps.iter().any(|step| {
            step.input.as_ref()
                .and_then(|i| i.get("prompt"))
                .and_then(|p| p.as_str())
                .map(|p| p.contains(prompt))
                .unwrap_or(false)
        })
    });

    if !found_prompt {
        anyhow::bail!("Test prompt not found in retrieved episodes from dual storage");
    }

    println!("    ✅ Prompt successfully retrieved from dual storage");
    println!("    📊 Retrieved {} episodes", retrieved_episodes.len());

    // Test patterns retrieval as well
    println!("  🧠 Testing pattern retrieval...");
    let patterns = memory.retrieve_relevant_patterns(&task_context, 5).await;
    println!("    📈 Retrieved {} patterns", patterns.len());

    // Cleanup
    drop(memory);
    std::fs::remove_file("./test_dual_turso_memory.db")?;
    std::fs::remove_file(cache_path)?;
    
    Ok(())
}