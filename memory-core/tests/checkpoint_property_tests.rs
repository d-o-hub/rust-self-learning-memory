//! Property tests for checkpoint serialization roundtrips

// Integration tests are separate crate roots and don't inherit .clippy.toml settings
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::doc_markdown)]
#![allow(missing_docs)]

use do_memory_core::memory::checkpoint::{CheckpointMeta, HandoffPack};
// HandoffSummary is not public in checkpoint mod, but it's okay, we can test CheckpointMeta and HandoffPack
use chrono::{TimeZone, Utc};
use proptest::prelude::*;
use uuid::Uuid;

fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(Uuid::from_bytes)
}

proptest! {
    #[test]
    fn checkpoint_meta_serialization_roundtrip(
        checkpoint_id in uuid_strategy(),
        timestamp_ms in 0i64..2_000_000_000_000i64,
        reason in ".*",
        step_number in 0usize..1000usize,
        note in prop::option::of(".*")
    ) {
        let meta = CheckpointMeta {
            checkpoint_id,
            created_at: Utc.timestamp_millis_opt(timestamp_ms).unwrap(),
            reason,
            step_number,
            note,
        };
        let encoded = serde_json::to_string(&meta).expect("serialize");
        let decoded: CheckpointMeta = serde_json::from_str(&encoded).expect("deserialize");
        prop_assert_eq!(meta, decoded);
    }

    #[test]
    fn handoff_pack_summary_consistency(
        checkpoint_id in uuid_strategy(),
        episode_id in uuid_strategy(),
        current_goal in ".*",
        what_worked in prop::collection::vec(".*", 0..10),
        what_failed in prop::collection::vec(".*", 0..10),
        salient_facts in prop::collection::vec(".*", 0..10),
        suggested_next_steps in prop::collection::vec(".*", 0..10)
    ) {
        let pack = HandoffPack {
            checkpoint_id,
            episode_id,
            timestamp: Utc::now(),
            current_goal,
            steps_completed: vec![],
            what_worked,
            what_failed,
            salient_facts,
            suggested_next_steps,
            relevant_patterns: vec![],
            relevant_heuristics: vec![],
        };

        let summary = pack.summary();
        prop_assert_eq!(summary.checkpoint_id, pack.checkpoint_id);
        prop_assert_eq!(summary.episode_id, pack.episode_id);
        prop_assert_eq!(summary.what_worked_count, pack.what_worked.len());
        prop_assert_eq!(summary.what_failed_count, pack.what_failed.len());
        prop_assert_eq!(summary.suggested_steps_count, pack.suggested_next_steps.len());
    }
}
