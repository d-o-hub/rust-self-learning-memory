//! Postcard serialization safety for attribution types (ADR-081 §7 / AC-12).
//!
//! The redb backend postcard-encodes `RecommendationSession` and
//! `RecommendationFeedback` (`memory-storage-redb/src/recommendations.rs`), so
//! those types must keep round-tripping through postcard — that is their wire
//! shape and it must not change. `PersistenceReceipt` is JSON-only: it is never
//! embedded in the persisted types, and its internally-tagged JSON shape is
//! not postcard-safe. Empirically (postcard 1.1.3) the receipt *serializes*
//! through postcard (serde routes internally-tagged struct variants through
//! `serialize_struct`, which postcard treats as a no-op), but it can never be
//! *deserialized*: internally-tagged enums deserialize through
//! `deserialize_any`, which postcard rejects with `WontImplement`. The JSON
//! round-trip plus the structural non-embedding assertions below are the
//! authoritative contract.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use do_memory_core::memory::attribution::{RecommendationFeedback, RecommendationSession};
use do_memory_core::{PersistenceReceipt, TaskOutcome};
use uuid::Uuid;

fn test_session() -> RecommendationSession {
    RecommendationSession {
        session_id: Uuid::new_v4(),
        episode_id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        recommended_pattern_ids: vec!["pattern-a".to_string(), "pattern-b".to_string()],
        recommended_playbook_ids: vec![Uuid::new_v4()],
    }
}

fn test_feedback() -> RecommendationFeedback {
    RecommendationFeedback {
        session_id: Uuid::new_v4(),
        applied_pattern_ids: vec!["pattern-a".to_string()],
        consulted_episode_ids: vec![Uuid::new_v4()],
        outcome: TaskOutcome::Success {
            verdict: "Done".to_string(),
            artifacts: vec!["output.txt".to_string()],
        },
        agent_rating: Some(0.9),
    }
}

fn all_receipts() -> Vec<PersistenceReceipt> {
    let session_id = Uuid::new_v4();
    let episode_id = Uuid::new_v4();
    vec![
        PersistenceReceipt::Persisted {
            session_id,
            episode_id,
        },
        PersistenceReceipt::PartiallyPersisted {
            session_id,
            episode_id,
            failed_backends: vec!["redb".to_string()],
        },
        PersistenceReceipt::MemoryOnly {
            session_id,
            episode_id,
        },
        PersistenceReceipt::PersistenceFailed {
            session_id,
            episode_id,
            failed_backends: vec!["turso".to_string(), "redb".to_string()],
        },
    ]
}

#[test]
fn recommendation_session_postcard_round_trips() {
    let session = test_session();
    let bytes = postcard::to_allocvec(&session).expect("session must postcard-serialize");
    let decoded: RecommendationSession =
        postcard::from_bytes(&bytes).expect("session must postcard-deserialize");
    assert_eq!(decoded, session);
}

#[test]
fn recommendation_feedback_postcard_round_trips() {
    let feedback = test_feedback();
    let bytes = postcard::to_allocvec(&feedback).expect("feedback must postcard-serialize");
    let decoded: RecommendationFeedback =
        postcard::from_bytes(&bytes).expect("feedback must postcard-deserialize");
    assert_eq!(decoded, feedback);
}

#[test]
fn persistence_receipt_json_round_trips_with_exact_state_tags() {
    // JSON is the receipt's wire contract (MCP/CLI). Every state must
    // round-trip and carry the exact snake_case `state` discriminant.
    let expected_tags = [
        ("persisted", 0usize),
        ("partially_persisted", 1),
        ("memory_only", 2),
        ("persistence_failed", 3),
    ];
    for (receipt, (tag, _)) in all_receipts().into_iter().zip(expected_tags.iter()) {
        let json = serde_json::to_string(&receipt).expect("receipt must JSON-serialize");
        assert!(
            json.contains(&format!("\"state\":\"{tag}\"")),
            "receipt JSON must carry the state tag {tag:?}, got: {json}"
        );
        let decoded: PersistenceReceipt =
            serde_json::from_str(&json).expect("receipt must JSON-deserialize");
        assert_eq!(
            decoded, receipt,
            "JSON round-trip must preserve the receipt"
        );
    }
}

#[test]
fn persistence_receipt_is_not_a_postcard_wire_type() {
    // JSON-only contract: the internally-tagged receipt cannot be read back
    // through postcard even when serialization happens to produce bytes
    // (postcard's `deserialize_any` is `WontImplement` for internally-tagged
    // enums). Any code that persisted a receipt through postcard would be
    // storing bytes it could never decode — the hazard ADR-081 §7 forbids.
    for receipt in all_receipts() {
        let bytes = postcard::to_allocvec(&receipt)
            .expect("postcard serialization itself must not error (documented behavior)");
        let decoded = postcard::from_bytes::<PersistenceReceipt>(&bytes);
        assert!(
            decoded.is_err(),
            "an internally-tagged PersistenceReceipt must not postcard-deserialize"
        );
    }
}

#[test]
fn persisted_types_do_not_embed_a_receipt() {
    // Structural non-embedding: the postcard-persisted types expose exactly
    // their documented fields. If a `PersistenceReceipt` were ever embedded in
    // either type, its JSON serialization would expose a `receipt` key and the
    // postcard wire shape would change.
    let session_json = serde_json::to_value(test_session()).expect("session JSON");
    let session_obj = session_json
        .as_object()
        .expect("session JSON must be an object");
    for key in [
        "session_id",
        "episode_id",
        "timestamp",
        "recommended_pattern_ids",
        "recommended_playbook_ids",
    ] {
        assert!(
            session_obj.contains_key(key),
            "missing expected field {key}"
        );
    }
    assert!(
        !session_obj.contains_key("receipt"),
        "RecommendationSession must not embed a PersistenceReceipt"
    );

    let feedback_json = serde_json::to_value(test_feedback()).expect("feedback JSON");
    let feedback_obj = feedback_json
        .as_object()
        .expect("feedback JSON must be an object");
    for key in [
        "session_id",
        "applied_pattern_ids",
        "consulted_episode_ids",
        "outcome",
        "agent_rating",
    ] {
        assert!(
            feedback_obj.contains_key(key),
            "missing expected field {key}"
        );
    }
    assert!(
        !feedback_obj.contains_key("receipt"),
        "RecommendationFeedback must not embed a PersistenceReceipt"
    );
}
