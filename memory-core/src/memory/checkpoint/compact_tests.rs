//! Unit tests for compact handoff packs (issue #965).

use super::*;
use crate::episode::ExecutionStep;

fn step(number: usize, tool: &str, action: &str) -> ExecutionStep {
    let mut step = ExecutionStep::new(number, tool.to_string(), action.to_string());
    step.result = Some(ExecutionResult::Success {
        output: format!("output of step {number}"),
    });
    step
}

fn sample_inputs(steps: usize) -> CompactInputs {
    CompactInputs {
        checkpoint_id: Uuid::new_v4(),
        episode_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        current_goal: "Ship the thing".to_string(),
        status: "in_progress".to_string(),
        steps_done: steps,
        steps_total: steps + 2,
        steps: (1..=steps)
            .map(|i| step(i, "tool", &format!("action {i}")))
            .collect(),
        worked: vec!["worked A".to_string()],
        failed: vec!["failed B".to_string()],
        salient_facts: vec!["fact C".to_string()],
        decisions: vec!["decision D".to_string()],
        pending_actions: vec!["next E".to_string()],
        pattern_ids: vec!["pattern-1".to_string()],
        heuristic_ids: vec!["heuristic-1".to_string()],
    }
}

#[test]
fn compact_budget_defaults_are_sane() {
    let budget = HandoffBudget::default();

    assert_eq!(budget.max_bytes, DEFAULT_HANDOFF_MAX_BYTES);
    assert!(budget.max_bytes >= 1024);
    assert!(budget.max_findings > 0);
    assert!(budget.max_evidence_excerpts > 0);
}

#[test]
fn compact_payload_complies_with_tight_budget() {
    let inputs = sample_inputs(50);
    let budget = HandoffBudget {
        max_bytes: 2048,
        max_evidence_excerpts: 3,
        ..HandoffBudget::default()
    };

    let pack = assemble_compact(inputs, &budget);

    assert!(
        pack.payload_bytes() <= budget.max_bytes,
        "payload {} exceeds {}",
        pack.payload_bytes(),
        budget.max_bytes
    );
    assert_eq!(pack.evidence_excerpts.len(), 3);
    // Most recent steps win, chronological order kept.
    assert_eq!(pack.evidence_excerpts[0].step_number, 48);
    assert_eq!(pack.evidence_excerpts[2].step_number, 50);
    assert_eq!(pack.omitted.omitted_steps, 47);
    assert_eq!(
        pack.omitted.full_available_via,
        "get_handoff_pack(checkpoint_id)"
    );
    assert_eq!(pack.approx_tokens, pack.payload_bytes() / 4);
}

#[test]
fn compact_assembly_is_deterministic() {
    let budget = HandoffBudget::default();
    let inputs = sample_inputs(12);
    let first = assemble_compact(inputs.clone(), &budget);
    let second = assemble_compact(inputs, &budget);

    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
}

#[test]
fn compact_omission_counts_are_exact() {
    let mut inputs = sample_inputs(4);
    inputs.worked = (0..6).map(|i| format!("worked {i}")).collect();
    inputs.failed = (0..6).map(|i| format!("failed {i}")).collect();
    inputs.salient_facts = (0..6).map(|i| format!("fact {i}")).collect();
    let budget = HandoffBudget {
        max_bytes: usize::MAX,
        max_findings: 5,
        ..HandoffBudget::default()
    };

    let pack = assemble_compact(inputs, &budget);

    // Priority: worked, then failed, then facts.
    assert_eq!(pack.verified_findings.len(), 5);
    assert!(pack.verified_findings[0].starts_with("worked"));
    assert_eq!(pack.omitted.omitted_findings, 18 - 5);
}

#[test]
fn compact_goal_truncation_is_utf8_safe() {
    let mut inputs = sample_inputs(1);
    inputs.current_goal = "ééééé done".to_string();
    let budget = HandoffBudget {
        max_bytes: usize::MAX,
        max_goal_chars: 4,
        ..HandoffBudget::default()
    };

    let pack = assemble_compact(inputs, &budget);

    assert_eq!(pack.current_goal, "éééé");
    assert_eq!(pack.omitted.truncated_fields, vec!["current_goal"]);
}

#[test]
fn compact_byte_pressure_cascades_into_knowledge_sections() {
    let mut inputs = sample_inputs(20);
    inputs.steps = (1..=20)
        .map(|i| {
            let mut excerpt = step(i, "tool", &format!("action {i}"));
            excerpt.result = Some(ExecutionResult::Success {
                output: "z".repeat(400),
            });
            excerpt
        })
        .collect();
    // Long knowledge sections so pressure cascades past excerpts into them.
    inputs.worked = vec!["w".repeat(300)];
    inputs.failed = vec!["f".repeat(300)];
    inputs.salient_facts = vec!["s".repeat(300)];
    inputs.decisions = vec!["d".repeat(300)];
    inputs.pending_actions = vec!["p".repeat(300)];
    let budget = HandoffBudget {
        max_bytes: 1100,
        ..HandoffBudget::default()
    };

    let pack = assemble_compact(inputs, &budget);

    assert!(
        pack.payload_bytes() <= budget.max_bytes,
        "payload {} exceeds {}",
        pack.payload_bytes(),
        budget.max_bytes
    );
    // Pressure passed refs and excerpts into pending/decisions/findings.
    let deep_omissions = pack.omitted.omitted_pending_actions
        + pack.omitted.omitted_decisions
        + pack.omitted.omitted_findings;
    assert!(
        deep_omissions > 0,
        "expected cascade past excerpts: {:?}",
        pack.omitted
    );
}

#[test]
fn compact_sub_floor_budget_returns_best_effort_skeleton() {
    let inputs = sample_inputs(20);
    let budget = HandoffBudget {
        max_bytes: 600,
        ..HandoffBudget::default()
    };

    let pack = assemble_compact(inputs, &budget);

    // Below the ~1 KB floor compliance is impossible (skeleton + receipts);
    // everything droppable is dropped and omissions stay exact.
    assert!(pack.verified_findings.is_empty());
    assert!(pack.decisions.is_empty());
    assert!(pack.pending_actions.is_empty());
    assert!(pack.evidence_excerpts.is_empty());
    assert!(pack.current_goal.is_empty());
    assert_eq!(pack.omitted.omitted_steps, 20);
    assert_eq!(pack.omitted.omitted_findings, 3);
    assert_eq!(pack.omitted.omitted_decisions, 1);
    assert_eq!(pack.omitted.omitted_pending_actions, 1);
    assert_eq!(pack.omitted.omitted_patterns, 1);
    assert_eq!(pack.omitted.omitted_heuristics, 1);
}
