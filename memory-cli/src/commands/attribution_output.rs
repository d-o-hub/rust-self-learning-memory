//! Shared human-readable rendering of ADR-080 attribution state.
//!
//! The pattern and playbook recommendation commands both print an
//! `--- Attribution Tracking (ADR-080) ---` block. Sharing it here guarantees
//! the attributed and unattributed surfaces cannot drift apart and keeps the
//! receipt wording identical across commands.

use std::io::Write;

use do_memory_core::{PersistenceReceipt, RecommendationSession};

/// Print the attribution block for a recommendation session and its
/// persistence receipt to stdout.
pub(crate) fn print_attribution_block(
    session: &RecommendationSession,
    receipt: &PersistenceReceipt,
) {
    let mut out = std::io::stdout();
    // stdout is line-buffered; a failed write is not actionable at this layer.
    let _ = write_attribution_block(&mut out, session, receipt);
}

/// Write the attribution block into any `Write` sink (unit-testable).
fn write_attribution_block<W: Write>(
    out: &mut W,
    session: &RecommendationSession,
    receipt: &PersistenceReceipt,
) -> std::io::Result<()> {
    writeln!(out, "--- Attribution Tracking (ADR-080) ---")?;
    writeln!(out, "Session ID: {}", session.session_id)?;
    writeln!(out, "Episode ID: {}", session.episode_id)?;

    match receipt {
        PersistenceReceipt::Persisted { .. } => {
            writeln!(out, "Durability: Persisted (durable across restarts)")?;
        }
        PersistenceReceipt::PartiallyPersisted {
            failed_backends, ..
        } => {
            writeln!(
                out,
                "⚠️ Durability: Partially Persisted (failed backends: {})",
                failed_backends.join(", ")
            )?;
        }
        PersistenceReceipt::MemoryOnly { .. } => {
            writeln!(
                out,
                "⚠️ Durability: Memory-only (process-local, will be lost on restart)"
            )?;
        }
        PersistenceReceipt::PersistenceFailed {
            failed_backends, ..
        } => {
            writeln!(
                out,
                "❌ Durability: Persistence Failed (failed backends: {})",
                failed_backends.join(", ")
            )?;
        }
    }

    Ok(())
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
    fn attribution_block_renders_every_receipt_state() {
        let s = session();

        let cases = [
            (
                PersistenceReceipt::Persisted {
                    session_id: s.session_id,
                    episode_id: s.episode_id,
                },
                "Durability: Persisted (durable across restarts)",
            ),
            (
                PersistenceReceipt::PartiallyPersisted {
                    session_id: s.session_id,
                    episode_id: s.episode_id,
                    failed_backends: vec!["redb".to_string()],
                },
                "⚠️ Durability: Partially Persisted (failed backends: redb)",
            ),
            (
                PersistenceReceipt::MemoryOnly {
                    session_id: s.session_id,
                    episode_id: s.episode_id,
                },
                "⚠️ Durability: Memory-only (process-local, will be lost on restart)",
            ),
            (
                PersistenceReceipt::PersistenceFailed {
                    session_id: s.session_id,
                    episode_id: s.episode_id,
                    failed_backends: vec!["turso".to_string(), "redb".to_string()],
                },
                "❌ Durability: Persistence Failed (failed backends: turso, redb)",
            ),
        ];

        for (receipt, expected) in cases {
            let mut buf = Vec::new();
            write_attribution_block(&mut buf, &s, &receipt).unwrap();
            let out = String::from_utf8(buf).unwrap();
            assert!(out.contains("--- Attribution Tracking (ADR-080) ---"));
            assert!(out.contains(&format!("Session ID: {}", s.session_id)));
            assert!(out.contains(&format!("Episode ID: {}", s.episode_id)));
            assert!(
                out.contains(expected),
                "expected {expected:?} in rendered block, got: {out:?}"
            );
        }
    }
}
