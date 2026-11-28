//! Behaviour tests for [`DiscoveryMemory`](super::DiscoveryMemory).
//!
//! Every test opens a real local Turso plus redb pair in a temporary directory,
//! because the guarantee under test is what survives a process boundary: a
//! replay has to find the episode another process wrote.

use super::*;
use crate::adaptive_discovery::dataset::{Plan, Truth};
use do_memory_core::{TaskOutcome, TaskType};

fn plan() -> Plan {
    Plan {
        strategy: "keyword".to_string(),
        provider: "alpha".to_string(),
    }
}

fn family() -> QueryFamily {
    QueryFamily {
        id: "nba".to_string(),
        query: "latest basketball scores and streaming sources".to_string(),
        domain: "sports-research".to_string(),
        baseline: plan(),
        history: Vec::new(),
        attempts: Vec::new(),
        truth: Truth {
            verified_urls: Vec::new(),
        },
    }
}

fn attempt(outcome: OutcomeClass) -> HistoryAttempt {
    HistoryAttempt {
        id: None,
        plan: plan(),
        outcome,
        latency_ms: 120,
        cost_usd: 0.25,
        endpoint: "https://news.example/nba?utm=feed".to_string(),
    }
}

async fn open(dir: &tempfile::TempDir) -> DiscoveryMemory {
    DiscoveryMemory::open(&dir.path().join("memory.db"))
        .await
        .expect("the local store should open")
}

/// Write an episode by hand, the way a caller bypassing [`DiscoveryMemory::record`] would.
async fn write_episode(
    memory: &DiscoveryMemory,
    family: &QueryFamily,
    attempt_id: Option<&str>,
    complete: bool,
) -> Uuid {
    let episode_id = memory
        .memory
        .start_episode(
            family.query.clone(),
            task_context(family),
            TaskType::Analysis,
        )
        .await;
    let tags = match attempt_id {
        Some(attempt_id) => tags_for(family, &family.baseline, OutcomeClass::Verified, attempt_id),
        None => vec![
            family_tag(&family.id),
            "strategy_keyword".to_string(),
            "provider_alpha".to_string(),
            "outcome_verified".to_string(),
        ],
    };
    memory
        .memory
        .add_episode_tags(episode_id, tags)
        .await
        .expect("tags should be accepted");
    if complete {
        memory
            .memory
            .complete_episode(
                episode_id,
                TaskOutcome::Success {
                    verdict: "written by hand".to_string(),
                    artifacts: Vec::new(),
                },
            )
            .await
            .expect("the episode should complete");
    }
    episode_id
}

#[test]
fn candidate_window_is_wider_than_the_requested_limit() {
    assert_eq!(candidate_window(1), CANDIDATE_WINDOW_FLOOR);
    assert_eq!(candidate_window(16), CANDIDATE_WINDOW_FLOOR);
    assert_eq!(candidate_window(17), 17 * CANDIDATE_WINDOW_MULTIPLIER);
    assert_eq!(candidate_window(0), CANDIDATE_WINDOW_FLOOR);
    assert!(candidate_window(usize::MAX) >= usize::MAX / 2);
}

#[tokio::test]
async fn replaying_the_same_attempt_keeps_one_episode() {
    let dir = tempfile::tempdir().expect("temp dir");
    let family = family();

    let first = {
        let memory = open(&dir).await;
        memory
            .record(&family, &attempt(OutcomeClass::Verified), "nba-h0")
            .await
            .expect("the first seed should record")
    };
    let replay = {
        let memory = open(&dir).await;
        memory
            .record(&family, &attempt(OutcomeClass::Verified), "nba-h0")
            .await
            .expect("the replay should resolve to the recorded episode")
    };

    assert_eq!(first.state, RecordState::Created);
    assert_eq!(replay.state, RecordState::Replayed);
    assert_eq!(first.episode_id, replay.episode_id);

    let memory = open(&dir).await;
    let (observations, ignored) = memory
        .observations(&family, 16)
        .await
        .expect("observations");
    assert_eq!(observations.len(), 1, "a replay must not add evidence");
    assert_eq!(ignored, 0);
}

#[tokio::test]
async fn separately_executed_attempts_stay_distinct() {
    let dir = tempfile::tempdir().expect("temp dir");
    let family = family();
    let memory = open(&dir).await;

    // Same plan, outcome, and endpoint: only the identity tells them apart, and
    // collapsing them would hide a genuine second observation.
    let first = memory
        .record(&family, &attempt(OutcomeClass::Verified), "nba-h0")
        .await
        .expect("the first attempt should record");
    let second = memory
        .record(&family, &attempt(OutcomeClass::Verified), "nba-h1")
        .await
        .expect("the second attempt should record");

    assert_eq!(second.state, RecordState::Created);
    assert_ne!(first.episode_id, second.episode_id);

    let (observations, _) = memory
        .observations(&family, 16)
        .await
        .expect("observations");
    assert_eq!(observations.len(), 2);
}

#[tokio::test]
async fn an_interrupted_attempt_is_retried_not_duplicated() {
    let dir = tempfile::tempdir().expect("temp dir");
    let family = family();
    let memory = open(&dir).await;

    let interrupted = write_episode(&memory, &family, Some("nba-h0"), false).await;

    let recorded = memory
        .record(&family, &attempt(OutcomeClass::Verified), "nba-h0")
        .await
        .expect("the retry should record the attempt");

    assert_eq!(recorded.state, RecordState::Retried);
    assert_ne!(
        recorded.episode_id, interrupted,
        "the partial episode must be replaced, not reused"
    );
    assert!(
        memory.memory.get_episode(interrupted).await.is_err(),
        "the partial episode must be gone"
    );

    let (observations, _) = memory
        .observations(&family, 16)
        .await
        .expect("observations");
    assert_eq!(observations.len(), 1);
}

#[tokio::test]
async fn a_duplicate_episode_for_one_attempt_is_counted_once() {
    let dir = tempfile::tempdir().expect("temp dir");
    let family = family();
    let memory = open(&dir).await;

    // Two completed episodes for one identity: what two seeds racing on the same
    // database can leave behind.
    write_episode(&memory, &family, Some("nba-h0"), true).await;
    write_episode(&memory, &family, Some("nba-h0"), true).await;

    let (observations, ignored) = memory
        .observations(&family, 16)
        .await
        .expect("observations");
    assert_eq!(observations.len(), 1, "one attempt is one observation");
    assert_eq!(ignored, 1, "the replica is reported, not silently dropped");
}

#[tokio::test]
async fn episodes_without_an_identity_are_reported_as_legacy() {
    let dir = tempfile::tempdir().expect("temp dir");
    let family = family();
    let memory = open(&dir).await;

    write_episode(&memory, &family, None, true).await;
    assert_eq!(
        memory.legacy_episodes(&family, 16).await.expect("legacy"),
        1,
        "an episode written without an identity cannot be replayed"
    );

    memory
        .record(&family, &attempt(OutcomeClass::Verified), "nba-h0")
        .await
        .expect("the identified attempt should record");
    assert_eq!(
        memory.legacy_episodes(&family, 16).await.expect("legacy"),
        1,
        "an identified attempt is not legacy"
    );
}
