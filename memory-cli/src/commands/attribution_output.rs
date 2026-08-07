//! Shared human-readable rendering of ADR-080 attribution state.
//!
//! The pattern and playbook recommendation commands both print an
//! `--- Attribution Tracking (ADR-080) ---` block. Sharing it here guarantees
//! the attributed and unattributed surfaces cannot drift apart and keeps the
//! receipt wording identical across commands.

use do_memory_core::{PersistenceReceipt, RecommendationSession};

/// Print the attribution block for a recommendation session and its
/// persistence receipt.
pub(crate) fn print_attribution_block(
    session: &RecommendationSession,
    receipt: &PersistenceReceipt,
) {
    println!("--- Attribution Tracking (ADR-080) ---");
    println!("Session ID: {}", session.session_id);
    println!("Episode ID: {}", session.episode_id);

    match receipt {
        PersistenceReceipt::Persisted { .. } => {
            println!("Durability: Persisted (durable across restarts)");
        }
        PersistenceReceipt::PartiallyPersisted {
            failed_backends, ..
        } => {
            println!(
                "⚠️ Durability: Partially Persisted (failed backends: {})",
                failed_backends.join(", ")
            );
        }
        PersistenceReceipt::MemoryOnly { .. } => {
            println!("⚠️ Durability: Memory-only (process-local, will be lost on restart)");
        }
        PersistenceReceipt::PersistenceFailed {
            failed_backends, ..
        } => {
            println!(
                "❌ Durability: Persistence Failed (failed backends: {})",
                failed_backends.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn session() -> RecommendationSession {
        RecommendationSession {
            session_id: Uuid::new_v4(),
            episode_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            recommended_pattern_ids: vec!["p1".to_string()],
            recommended_playbook_ids: vec![],
        }
    }

    #[test]
    fn attribution_block_covers_all_receipt_states() {
        let s = session();

        let persisted = PersistenceReceipt::Persisted {
            session_id: s.session_id,
            episode_id: s.episode_id,
        };
        assert!(persisted.is_durable());
        assert!(persisted.is_restart_safe());

        let partial = PersistenceReceipt::PartiallyPersisted {
            session_id: s.session_id,
            episode_id: s.episode_id,
            failed_backends: vec!["redb".to_string()],
        };
        assert!(partial.is_durable());
        assert!(!partial.is_restart_safe());

        let memory_only = PersistenceReceipt::MemoryOnly {
            session_id: s.session_id,
            episode_id: s.episode_id,
        };
        assert!(!memory_only.is_durable());
        assert!(!memory_only.is_restart_safe());

        let failed = PersistenceReceipt::PersistenceFailed {
            session_id: s.session_id,
            episode_id: s.episode_id,
            failed_backends: vec!["turso".to_string(), "redb".to_string()],
        };
        assert!(!failed.is_durable());
        assert!(!failed.is_restart_safe());

        // The renderer must accept every state without panicking.
        for receipt in [persisted, partial, memory_only, failed] {
            print_attribution_block(&s, &receipt);
        }
    }
}
