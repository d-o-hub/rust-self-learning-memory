//! Operation journal CLI (F4.2).

use crate::config::Config;
use crate::output::OutputFormat;

use super::types::{JournalEntryView, JournalStatus};

/// Show operation journal status and optionally run eviction repair (F4.2).
pub async fn journal_status(
    memory: &do_memory_core::SelfLearningMemory,
    _config: &Config,
    format: OutputFormat,
    pending_only: bool,
    repair: bool,
) -> anyhow::Result<()> {
    use do_memory_core::{JournalOpKind, JournalOutcome};

    let mut remaining_after_repair = None;
    if repair {
        let remaining = memory.reconcile_pending_evictions().await?;
        remaining_after_repair = Some(remaining.len());
    }

    let snapshot = memory.operation_journal_snapshot().await;
    let pending = memory.operation_journal_pending().await;
    let source = if pending_only {
        pending.as_slice()
    } else {
        snapshot.as_slice()
    };

    let entries: Vec<JournalEntryView> = source
        .iter()
        .take(50)
        .map(|e| {
            let outcome = match &e.outcome {
                JournalOutcome::Success => "success".to_string(),
                JournalOutcome::Pending => "pending".to_string(),
                JournalOutcome::Failed { error } => format!("failed: {error}"),
            };
            let kind = match e.kind {
                JournalOpKind::CapacityEviction => "capacity_eviction",
                JournalOpKind::EpisodeDelete => "episode_delete",
                JournalOpKind::EmbeddingDelete => "embedding_delete",
            };
            JournalEntryView {
                op_id: e.op_id.to_string(),
                episode_id: e.episode_id.to_string(),
                kind: kind.to_string(),
                backend: format!("{:?}", e.backend),
                outcome,
                recorded_at_ms: e.recorded_at_ms,
            }
        })
        .collect();

    let status = JournalStatus {
        total_entries: snapshot.len(),
        pending_repairs: pending.len(),
        remaining_after_repair,
        repair_attempted: repair,
        entries,
    };

    format.print_output(&status)?;
    Ok(())
}
