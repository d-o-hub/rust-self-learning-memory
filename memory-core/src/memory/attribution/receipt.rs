//! Persistence receipt and attributed recommendation result types.
//!
//! These types implement the ADR-080 truthful persistence state:
//! every attributed operation returns a machine-stable receipt that
//! distinguishes durable, partial, process-only, and failed persistence.
//!
//! # Serialization contract
//!
//! `PersistenceReceipt` is a JSON-only (`serde_json`) receipt. It MUST NOT be
//! embedded in postcard-persisted types: `RecommendationSession` and
//! `RecommendationFeedback` are postcard-encoded by the redb backend and carry
//! no receipt field. Adding one would change their wire shape and break
//! existing persisted data.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::RecommendationSession;

// ============================================================================
// Persistence Receipt
// ============================================================================

/// Tagged receipt for attribution persistence outcomes (ADR-080 §3).
///
/// The attributed recommendation operation always succeeds when recommendation
/// generation succeeds. Its durability state is separate and captured here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum PersistenceReceipt {
    /// All configured capable backends wrote the session.
    Persisted {
        /// The session that was persisted.
        session_id: Uuid,
        /// The episode this session belongs to.
        episode_id: Uuid,
    },
    /// At least one configured capable backend wrote it and at least one failed.
    PartiallyPersisted {
        /// The session that was partially persisted.
        session_id: Uuid,
        /// The episode this session belongs to.
        episode_id: Uuid,
        /// Stable backend identifiers that failed (no raw errors or credentials).
        failed_backends: Vec<String>,
    },
    /// No configured backend advertises recommendation-attribution capability;
    /// the session exists only in this process.
    ///
    /// This includes the case where backends are configured but advertise
    /// capability `false` (ADR-081 §2): non-advertising backends are never
    /// counted as durable, so no write is attempted and the receipt can never
    /// claim a write the backend cannot honor.
    MemoryOnly {
        /// The session that is memory-only.
        session_id: Uuid,
        /// The episode this session belongs to.
        episode_id: Uuid,
    },
    /// All configured capable backends failed; the session remains process-local.
    PersistenceFailed {
        /// The session that failed to persist.
        session_id: Uuid,
        /// The episode this session belongs to.
        episode_id: Uuid,
        /// Stable backend identifiers that failed (no raw errors or credentials).
        failed_backends: Vec<String>,
    },
}

impl PersistenceReceipt {
    /// Extract the session ID from any receipt state.
    pub fn session_id(&self) -> Uuid {
        match self {
            Self::Persisted { session_id, .. }
            | Self::PartiallyPersisted { session_id, .. }
            | Self::MemoryOnly { session_id, .. }
            | Self::PersistenceFailed { session_id, .. } => *session_id,
        }
    }

    /// Extract the episode ID from any receipt state.
    pub fn episode_id(&self) -> Uuid {
        match self {
            Self::Persisted { episode_id, .. }
            | Self::PartiallyPersisted { episode_id, .. }
            | Self::MemoryOnly { episode_id, .. }
            | Self::PersistenceFailed { episode_id, .. } => *episode_id,
        }
    }

    /// Returns true if the session is durably stored in at least one backend.
    pub fn is_durable(&self) -> bool {
        matches!(
            self,
            Self::Persisted { .. } | Self::PartiallyPersisted { .. }
        )
    }

    /// Returns true if feedback submitted after restart will find this session.
    pub fn is_restart_safe(&self) -> bool {
        matches!(self, Self::Persisted { .. })
    }
}

// ============================================================================
// Attributed Recommendation Result
// ============================================================================

/// Result of an attributed pattern recommendation (ADR-080 §1–3).
///
/// Wraps the recommendation results with an attribution receipt so callers
/// can distinguish durable, partial, process-only, and failed persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedPatternResult<R> {
    /// The recommendations returned to the caller.
    pub recommendations: Vec<R>,
    /// The recommendation session created from the exact returned IDs.
    pub session: RecommendationSession,
    /// Persistence outcome for the session.
    pub receipt: PersistenceReceipt,
}

/// Result of an attributed playbook recommendation (ADR-080 §1–3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributedPlaybookResult<P> {
    /// The playbooks returned to the caller.
    pub playbooks: Vec<P>,
    /// The recommendation session created from the exact returned IDs.
    pub session: RecommendationSession,
    /// Persistence outcome for the session.
    pub receipt: PersistenceReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ids() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    #[test]
    fn test_persisted_receipt_fields() {
        let (sid, eid) = make_ids();
        let r = PersistenceReceipt::Persisted {
            session_id: sid,
            episode_id: eid,
        };
        assert_eq!(r.session_id(), sid);
        assert_eq!(r.episode_id(), eid);
        assert!(r.is_durable());
        assert!(r.is_restart_safe());
    }

    #[test]
    fn test_memory_only_receipt() {
        let (sid, eid) = make_ids();
        let r = PersistenceReceipt::MemoryOnly {
            session_id: sid,
            episode_id: eid,
        };
        assert!(!r.is_durable());
        assert!(!r.is_restart_safe());
    }

    #[test]
    fn test_partially_persisted_receipt() {
        let (sid, eid) = make_ids();
        let r = PersistenceReceipt::PartiallyPersisted {
            session_id: sid,
            episode_id: eid,
            failed_backends: vec!["redb".to_string()],
        };
        assert!(r.is_durable());
        assert!(!r.is_restart_safe());
    }

    #[test]
    fn test_persistence_failed_receipt() {
        let (sid, eid) = make_ids();
        let r = PersistenceReceipt::PersistenceFailed {
            session_id: sid,
            episode_id: eid,
            failed_backends: vec!["turso".to_string(), "redb".to_string()],
        };
        assert!(!r.is_durable());
        assert!(!r.is_restart_safe());
    }

    #[test]
    fn test_receipt_serialization_roundtrip() {
        let (sid, eid) = make_ids();
        let r = PersistenceReceipt::Persisted {
            session_id: sid,
            episode_id: eid,
        };
        let json = serde_json::to_string(&r).unwrap();
        let r2: PersistenceReceipt = serde_json::from_str(&json).unwrap();
        assert_eq!(r, r2);
    }
}
