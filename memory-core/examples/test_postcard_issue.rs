//! Test to verify postcard serialization/deserialization issue with ExecutionStep

// Examples have relaxed clippy rules for demonstration patterns
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]

use do_memory_core::{Episode, ExecutionStep, TaskContext, TaskType};

fn main() {
    println!("Testing postcard serialization and deserialization of Episode and ExecutionStep...");

    // Create an episode with no steps
    let episode = Episode::new(
        "test task".to_string(),
        TaskContext::default(),
        TaskType::Testing,
    );

    // Serialize without steps
    match postcard::to_allocvec(&episode) {
        Ok(bytes) => {
            println!("✓ Episode without steps: {} bytes", bytes.len());
            // Deserialize
            match postcard::from_bytes::<Episode>(&bytes) {
                Ok(_) => println!("✓ Episode without steps deserialized successfully"),
                Err(e) => println!("✗ Episode without steps deserialization FAILED: {}", e),
            }
        }
        Err(e) => println!("✗ Episode without steps serialization FAILED: {}", e),
    }

    // Create a step and add to episode
    let mut episode_with_step = episode.clone();
    let step = ExecutionStep::new(1, "test-tool".to_string(), "test-action".to_string());
    episode_with_step.steps.push(step.clone());

    // Serialize with step
    match postcard::to_allocvec(&episode_with_step) {
        Ok(bytes) => {
            println!("✓ Episode with 1 step: {} bytes", bytes.len());
            // Deserialize - this should fail!
            match postcard::from_bytes::<Episode>(&bytes) {
                Ok(_) => println!("✓ Episode with 1 step deserialized successfully"),
                Err(e) => println!("✗ Episode with 1 step deserialization FAILED: {}", e),
            }
        }
        Err(e) => println!("✗ Episode with 1 step serialization FAILED: {}", e),
    }

    // Try to serialize ExecutionStep alone
    match postcard::to_allocvec(&step) {
        Ok(bytes) => {
            println!("✓ ExecutionStep alone: {} bytes", bytes.len());
            // Deserialize
            match postcard::from_bytes::<ExecutionStep>(&bytes) {
                Ok(_) => println!("✓ ExecutionStep deserialized successfully"),
                Err(e) => println!("✗ ExecutionStep deserialization FAILED: {}", e),
            }
        }
        Err(e) => println!("✗ ExecutionStep serialization FAILED: {}", e),
    }

    // Try to serialize and deserialize serde_json::Value
    let json_value = serde_json::json!({"key": "value"});
    match postcard::to_allocvec(&json_value) {
        Ok(bytes) => {
            println!("✓ serde_json::Value serialization: {} bytes", bytes.len());
            // Deserialize
            match postcard::from_bytes::<serde_json::Value>(&bytes) {
                Ok(_) => println!("✓ serde_json::Value deserialized successfully"),
                Err(e) => println!("✗ serde_json::Value deserialization FAILED: {}", e),
            }
        }
        Err(e) => println!("✗ serde_json::Value serialization FAILED: {}", e),
    }
}
