//! Property tests for playbook serialization roundtrips

use do_memory_core::memory::playbook::{
    PlaybookPitfall, PlaybookStep, PlaybookSynthesisSource, RecommendedPlaybook,
};
use proptest::prelude::*;
use uuid::Uuid;

proptest! {
    #[test]
    fn playbook_step_serialization_roundtrip(order in 0usize..100usize, action in ".*", tool_hint in prop::option::of(".*"), expected_result in prop::option::of(".*")) {
        let step = PlaybookStep {
            order,
            action,
            tool_hint,
            expected_result,
        };
        let encoded = serde_json::to_string(&step).expect("serialize");
        let decoded: PlaybookStep = serde_json::from_str(&encoded).expect("deserialize");
        prop_assert_eq!(step, decoded);
    }

    #[test]
    fn playbook_pitfall_serialization_roundtrip(warning in ".*", reason in ".*", mitigation in prop::option::of(".*")) {
        let pitfall = PlaybookPitfall {
            warning,
            reason,
            mitigation,
        };
        let encoded = serde_json::to_string(&pitfall).expect("serialize");
        let decoded: PlaybookPitfall = serde_json::from_str(&encoded).expect("deserialize");
        prop_assert_eq!(pitfall, decoded);
    }

    #[test]
    fn synthesis_source_count_invariant(pattern_count in 0usize..20usize, episode_count in 0usize..20usize, summary_count in 0usize..20usize) {
        let mut source = PlaybookSynthesisSource::new();
        for _ in 0..pattern_count { source.add_pattern(Uuid::new_v4()); }
        for _ in 0..episode_count { source.add_episode(Uuid::new_v4()); }
        for _ in 0..summary_count { source.add_summary(Uuid::new_v4()); }

        prop_assert_eq!(source.total_sources(), pattern_count + episode_count + summary_count);
    }

    #[test]
    fn recommended_playbook_quality_score_bounds(task_match in 0.0f32..1.0f32, confidence in 0.0f32..1.0f32, source_count in 0usize..100usize) {
        let mut playbook = RecommendedPlaybook::new(Uuid::new_v4(), task_match);
        playbook.confidence = confidence;
        for _ in 0..source_count {
            playbook.supporting_pattern_ids.push(Uuid::new_v4());
        }

        let score = playbook.quality_score();
        prop_assert!(score >= 0.0);
        prop_assert!(score <= 1.0);
    }
}
