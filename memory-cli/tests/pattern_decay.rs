//! `pattern decay` must delete the patterns it reports as decayed.
//!
//! Runs the command implementation against a real redb store: the pre-fix code
//! printed "Successfully decayed N" while leaving every row in place.

#![cfg(feature = "redb")]

use std::sync::Arc;

use do_memory_cli::commands::decay_patterns;
use do_memory_cli::config::Config;
use do_memory_cli::output::OutputFormat;
use do_memory_core::patterns::{Pattern, PatternEffectiveness};
use do_memory_core::{MemoryConfig, SelfLearningMemory, StorageBackend, TaskContext, TaskType};
use do_memory_storage_redb::RedbStorage;
use uuid::Uuid;

#[tokio::test]
async fn decay_deletes_ineffective_patterns_from_storage() {
    let dir = tempfile::tempdir().expect("tempdir");
    let redb: Arc<dyn StorageBackend> = Arc::new(
        RedbStorage::new(&dir.path().join("decay.redb"))
            .await
            .expect("open redb"),
    );
    let memory = SelfLearningMemory::with_storage(MemoryConfig::default(), redb.clone(), redb);

    let episode_id = memory
        .start_episode(
            "Decay an ineffective pattern".to_string(),
            TaskContext::default(),
            TaskType::Testing,
        )
        .await;

    let mut effectiveness = PatternEffectiveness::new();
    effectiveness.times_applied = 4;
    effectiveness.failure_when_applied = 4;

    let pattern = Pattern::ContextPattern {
        id: Uuid::new_v4(),
        context_features: vec!["domain:testing".to_string()],
        recommended_approach: "Known to fail".to_string(),
        evidence: vec![],
        success_rate: 0.0,
        effectiveness,
    };
    let pattern_id = pattern.id();

    memory
        .store_patterns(episode_id, vec![pattern])
        .await
        .expect("store pattern");
    assert!(
        memory
            .get_pattern(pattern_id)
            .await
            .expect("get pattern")
            .is_some()
    );

    decay_patterns(&memory, &Config::default(), OutputFormat::Json, false, true)
        .await
        .expect("decay");

    assert!(
        memory
            .get_pattern(pattern_id)
            .await
            .expect("get after decay")
            .is_none(),
        "decay reported success without deleting the pattern"
    );
    assert_eq!(
        memory
            .get_all_patterns()
            .await
            .expect("list after decay")
            .len(),
        0
    );
    assert_eq!(
        memory
            .get_episode_patterns(episode_id)
            .await
            .expect("episode patterns")
            .len(),
        0
    );
}
