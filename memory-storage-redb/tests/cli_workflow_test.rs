//! Test CLI-like behavior where each operation is a separate storage instance

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::float_cmp)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::panic)]

use chrono::TimeZone;
use do_memory_core::{Episode, ExecutionStep, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_cli_like_episode_workflow() {
    // Create a persistent directory (not temp, so we can re-open)
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("cli_test.redb");

    println!("Database path: {}", db_path.display());

    // Session 1: Create episode
    {
        let storage1 = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage 1");
        let episode = Episode::new(
            "CLI workflow test".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        );
        let episode_id = episode.episode_id;

        println!("Session 1: Storing episode {}", episode_id);
        storage1
            .store_episode(&episode)
            .await
            .expect("Failed to store episode");
        println!("Session 1: Episode stored");
        // storage1 is dropped here, releasing the lock
    }

    // Get the episode_id from a fresh storage instance
    let episode_id = {
        let storage_check = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage check");
        let episodes = storage_check
            .query_episodes_since(
                chrono::Utc
                    .timestamp_millis_opt(0)
                    .single()
                    .unwrap_or_else(chrono::Utc::now),
                Some(100),
            )
            .await
            .expect("Failed to query episodes");
        episodes
            .first()
            .map(|ep| ep.episode_id)
            .expect("Should have episode")
    };

    // Session 2: View episode (simulate separate CLI process)
    {
        let storage2 = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage 2");
        println!("Session 2: Retrieving episode {}", episode_id);
        let retrieved = storage2
            .get_episode(episode_id)
            .await
            .expect("Failed to get episode");
        match retrieved {
            Some(ep) => {
                println!("Session 2: Found episode with {} steps", ep.steps.len());
                assert_eq!(ep.steps.len(), 0, "Episode should have 0 steps initially");
            }
            None => {
                panic!("Session 2: Episode not found!");
            }
        }
        // storage2 is dropped here
    }

    // Session 3: Add step to episode
    {
        let storage3 = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage 3");
        println!("Session 3: Retrieving episode for modification");
        let episode_for_modification = storage3
            .get_episode(episode_id)
            .await
            .expect("Failed to get episode for modification")
            .expect("Episode should exist");

        println!("Session 3: Episode retrieved, adding step");
        let mut modified_episode = episode_for_modification.clone();
        let step = ExecutionStep::new(1, "test-tool".to_string(), "test-action".to_string());
        modified_episode.steps.push(step);
        // Add metadata to simulate a meaningful modification
        modified_episode
            .metadata
            .insert("modified".to_string(), "true".to_string());

        println!("Session 3: Storing modified episode");
        // Simulate double-write (both storage backends)
        storage3
            .store_episode(&modified_episode)
            .await
            .expect("Failed first write");
        storage3
            .store_episode(&modified_episode)
            .await
            .expect("Failed second write");
        println!("Session 3: Episode stored (double-write)");
        // storage3 is dropped here
    }

    // Session 4: Verify episode persists with step
    {
        let storage4 = RedbStorage::new(&db_path)
            .await
            .expect("Failed to create storage 4");
        println!("Session 4: Retrieving final episode");
        let final_episode = storage4
            .get_episode(episode_id)
            .await
            .expect("Failed to get final episode");

        match final_episode {
            Some(ep) => {
                println!("Session 4: Found episode with {} steps", ep.steps.len());
                assert_eq!(
                    ep.steps.len(),
                    1,
                    "Episode should have 1 step after modification"
                );
                println!("SUCCESS: Episode persisted with step!");
            }
            None => {
                panic!("Session 4: Episode disappeared after log_step! This matches CLI bug.");
            }
        }
    }
}
