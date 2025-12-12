use memory_core::{SelfLearningMemory, TaskContext, TaskType, ExecutionStep, ExecutionResult};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the memory system
    let memory = SelfLearningMemory::new();
    
    // Create task context for debugging MCP server configuration
    let task_context = TaskContext {
        language: Some("rust".to_string()),
        framework: None,
        complexity: memory_core::ComplexityLevel::Moderate,
        domain: "debugging".to_string(),
        tags: vec!["mcp".to_string(), "configuration".to_string(), "opencode".to_string()],
    };
    
    // Start the debugging episode
    let episode_id = memory.start_episode(
        "Debug memory-mcp server configuration issue where it works in Claude Code but fails in OpenCode CLI".to_string(),
        task_context,
        TaskType::Debugging,
    ).await;
    
    println!("Started debugging episode with ID: {}", episode_id);
    
    // Log the initial problem statement step
    let mut problem_step = ExecutionStep::new(
        1,
        "debugger".to_string(),
        "Log problem statement and start investigation".to_string(),
    );
    
    // Add details about the problem
    problem_step.parameters = serde_json::json!({
        "environment": "OpenCode CLI vs Claude Code",
        "issue": "memory-mcp server configuration works in Claude Code but fails in OpenCode CLI",
        "status": "investigation_started"
    });
    
    problem_step.result = Some(ExecutionResult::Success {
        output: "Problem statement logged successfully. Starting investigation into MCP server configuration differences between environments.".to_string(),
    });
    
    problem_step.latency_ms = 5; // Minimal time for logging
    problem_step.metadata = {
        let mut meta = HashMap::new();
        meta.insert("phase".to_string(), "initial_investigation".to_string());
        meta.insert("priority".to_string(), "high".to_string());
        meta
    };
    
    // Log the step
    memory.log_step(episode_id, problem_step).await;
    
    println!("Logged initial problem statement step");
    
    // Get current stats to verify episode was created
    let (total_episodes, completed_episodes, total_patterns) = memory.get_stats().await;
    println!("Memory stats - Total episodes: {}, Completed: {}, Patterns: {}", 
             total_episodes, completed_episodes, total_patterns);
    
    // Retrieve the episode to verify it was stored correctly
    let episode = memory.get_episode(episode_id).await?;
    println!("Retrieved episode: {}", episode.task_description);
    println!("Episode has {} steps logged", episode.steps.len());
    println!("Episode domain: {}", episode.context.domain);
    println!("Episode tags: {:?}", episode.context.tags);
    
    Ok(())
}