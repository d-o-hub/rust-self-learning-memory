//! Tests for DAG-based state management (WG-134).

use super::assembler::AssemblyFormat;
use super::*;
use crate::episode::Episode;
use crate::types::{ComplexityLevel, TaskContext, TaskType};
use std::sync::Arc;
use uuid::Uuid;

fn create_test_episode(desc: &str, task_type: TaskType, context: TaskContext) -> Episode {
    Episode::new(desc.to_string(), context, task_type)
}

fn rust_web_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("axum".to_string()),
        complexity: ComplexityLevel::Moderate,
        domain: "web-api".to_string(),
        tags: vec!["async".to_string(), "rest".to_string()],
    }
}

fn rust_cli_context() -> TaskContext {
    TaskContext {
        language: Some("rust".to_string()),
        framework: Some("clap".to_string()),
        complexity: ComplexityLevel::Simple,
        domain: "cli".to_string(),
        tags: vec!["async".to_string()],
    }
}

fn python_data_context() -> TaskContext {
    TaskContext {
        language: Some("python".to_string()),
        framework: Some("pandas".to_string()),
        complexity: ComplexityLevel::Complex,
        domain: "data-science".to_string(),
        tags: vec!["etl".to_string()],
    }
}

// ============================================================================
// StateNode Tests
// ============================================================================

#[test]
fn test_state_node_creation() {
    let node = StateNode::new(StateNodeType::Language, "rust".to_string());
    assert_eq!(node.node_type, StateNodeType::Language);
    assert_eq!(node.value, "rust");
    assert!(node.episode_refs.is_empty());
    assert_eq!(node.ref_count(), 0);
}

#[test]
fn test_state_node_add_ref() {
    let mut node = StateNode::new(StateNodeType::Domain, "web-api".to_string());
    let ep_id = uuid::Uuid::new_v4();

    node.add_episode_ref(ep_id);
    assert_eq!(node.ref_count(), 1);
    assert!(node.episode_refs.contains(&ep_id));
    assert!(node.has_refs());
}

#[test]
fn test_state_node_multiple_refs() {
    let mut node = StateNode::new(StateNodeType::Tag, "async".to_string());
    let ep1 = uuid::Uuid::new_v4();
    let ep2 = uuid::Uuid::new_v4();
    let ep3 = uuid::Uuid::new_v4();

    node.add_episode_ref(ep1);
    node.add_episode_ref(ep2);
    node.add_episode_ref(ep3);

    assert_eq!(node.ref_count(), 3);
}

#[test]
fn test_state_node_remove_ref() {
    let mut node = StateNode::new(StateNodeType::TaskType, "Debugging".to_string());
    let ep1 = uuid::Uuid::new_v4();
    let ep2 = uuid::Uuid::new_v4();

    node.add_episode_ref(ep1);
    node.add_episode_ref(ep2);
    assert_eq!(node.ref_count(), 2);

    node.remove_episode_ref(&ep1);
    assert_eq!(node.ref_count(), 1);
    assert!(!node.episode_refs.contains(&ep1));
}

#[test]
fn test_state_node_token_savings() {
    let mut node = StateNode::new(StateNodeType::Domain, "web-api".to_string());
    // No savings with 0 or 1 refs
    assert_eq!(node.token_savings(), 0);

    // Add multiple refs for savings
    for _ in 0..5 {
        node.add_episode_ref(uuid::Uuid::new_v4());
    }

    // Should have savings now
    assert!(node.token_savings() > 0);
}

#[test]
fn test_state_node_estimated_tokens() {
    let node = StateNode::new(StateNodeType::Language, "rust".to_string());
    // "rust" = 4 chars, ~1 token + 2 overhead = ~3 tokens
    assert!(node.estimated_tokens() >= 1);
}

// ============================================================================
// StateEdge Tests
// ============================================================================

#[test]
fn test_state_edge_creation() {
    let ep_id = uuid::Uuid::new_v4();
    let node_id = uuid::Uuid::new_v4();

    let edge = StateEdge::new(ep_id, node_id, EdgeType::HasAttribute);
    assert_eq!(edge.source_episode, ep_id);
    assert_eq!(edge.target_node, node_id);
    assert_eq!(edge.edge_type, EdgeType::HasAttribute);
    assert_eq!(edge.strength, 1.0);
}

#[test]
fn test_state_edge_attribute() {
    let ep_id = uuid::Uuid::new_v4();
    let node_id = uuid::Uuid::new_v4();

    let edge = StateEdge::attribute(ep_id, node_id, "language".to_string());
    assert_eq!(edge.edge_type, EdgeType::HasAttribute);
    assert_eq!(edge.source_field(), Some("language"));
    assert!(edge.is_primary());
}

#[test]
fn test_state_edge_strength() {
    let ep_id = uuid::Uuid::new_v4();
    let node_id = uuid::Uuid::new_v4();
    let mut edge = StateEdge::new(ep_id, node_id, EdgeType::SimilarTo);

    edge.set_strength(0.5);
    assert_eq!(edge.strength, 0.5);

    // Clamp to 0-1
    edge.set_strength(2.0);
    assert_eq!(edge.strength, 1.0);

    edge.set_strength(-0.5);
    assert_eq!(edge.strength, 0.0);
}

// ============================================================================
// StateDag Tests
// ============================================================================

#[test]
fn test_state_dag_creation() {
    let dag = StateDag::new();
    assert_eq!(dag.node_count(), 0);
    assert_eq!(dag.edge_count(), 0);
    assert_eq!(dag.stats().total_episodes, 0);
}

#[test]
fn test_state_dag_register_episode() {
    let mut dag = StateDag::new();
    let episode = create_test_episode("Fix auth bug", TaskType::Debugging, rust_web_context());

    let edges = dag.register_episode(&episode);
    assert!(edges > 0); // Should create edges for language, domain, task_type, etc.
    assert!(
        dag.node_count() >= 5,
        "Should create at least 5 nodes (language, domain, task_type, complexity, tags)"
    );
}

#[test]
fn test_state_dag_shared_nodes() {
    let mut dag = StateDag::new();

    // Register two episodes with same language/domain
    let ep1 = create_test_episode("Task 1", TaskType::Debugging, rust_web_context());
    let ep2 = create_test_episode("Task 2", TaskType::Refactoring, rust_web_context());

    dag.register_episode(&ep1);
    dag.register_episode(&ep2);

    // Should reuse language and domain nodes
    assert!(dag.node_count() < 10); // Less than if each episode had unique nodes

    // Check language node has 2 refs
    let lang_nodes = dag.nodes_by_type(StateNodeType::Language);
    assert_eq!(lang_nodes.len(), 1);
    assert_eq!(lang_nodes[0].ref_count(), 2);
}

#[test]
fn test_state_dag_different_contexts() {
    let mut dag = StateDag::new();

    // Register episodes with different contexts
    let ep1 = create_test_episode("Task 1", TaskType::Debugging, rust_web_context());
    let ep2 = create_test_episode("Task 2", TaskType::CodeGeneration, python_data_context());

    dag.register_episode(&ep1);
    dag.register_episode(&ep2);

    // Should have separate language nodes
    let lang_nodes = dag.nodes_by_type(StateNodeType::Language);
    assert_eq!(lang_nodes.len(), 2); // rust and python
}

#[test]
fn test_state_dag_get_episode_nodes() {
    let mut dag = StateDag::new();
    let episode = create_test_episode("Task", TaskType::Debugging, rust_web_context());

    dag.register_episode(&episode);

    let nodes = dag.get_episode_nodes(&episode.episode_id);
    assert!(!nodes.is_empty());

    // Should have language, domain, etc.
    let types: Vec<_> = nodes.iter().map(|n| n.node_type).collect();
    assert!(types.contains(&StateNodeType::Language));
    assert!(types.contains(&StateNodeType::Domain));
}

#[test]
fn test_state_dag_get_shared_context() {
    let mut dag = StateDag::new();

    let ep1 = create_test_episode("Task 1", TaskType::Debugging, rust_web_context());
    let ep2 = create_test_episode("Task 2", TaskType::Refactoring, rust_web_context());

    dag.register_episode(&ep1);
    dag.register_episode(&ep2);

    // Both share language, domain, framework, tags
    let shared = dag.get_shared_context(&[ep1.episode_id, ep2.episode_id]);
    assert!(!shared.is_empty());

    // Language should be shared
    assert!(shared
        .iter()
        .any(|n| n.node_type == StateNodeType::Language));
}

#[test]
fn test_state_dag_remove_episode() {
    let mut dag = StateDag::new();

    let ep1 = create_test_episode("Task 1", TaskType::Debugging, rust_web_context());
    let ep2 = create_test_episode("Task 2", TaskType::Refactoring, rust_web_context());

    dag.register_episode(&ep1);
    dag.register_episode(&ep2);

    // Remove one episode
    let removed = dag.remove_episode(&ep1.episode_id);
    assert!(
        removed,
        "remove_episode should return true for registered episode"
    );

    // Language node should still exist (referenced by ep2)
    let lang_nodes = dag.nodes_by_type(StateNodeType::Language);
    assert_eq!(lang_nodes.len(), 1);
    assert_eq!(lang_nodes[0].ref_count(), 1);
}

#[test]
fn test_state_dag_token_reduction() {
    let mut dag = StateDag::new();

    // Register 10 episodes with same context for clear reduction
    for i in 0..10 {
        let ep = create_test_episode(
            &format!("Task {i}"),
            TaskType::Debugging,
            rust_web_context(),
        );
        dag.register_episode(&ep);
    }

    // Should have significant token savings (>50% reduction expected)
    let reduction = dag.reduction_percentage();
    assert!(reduction > 50.0);
}

#[test]
fn test_state_dag_stats() {
    let mut dag = StateDag::new();

    for i in 0..3 {
        let ep = create_test_episode(
            &format!("Task {i}"),
            TaskType::Debugging,
            rust_web_context(),
        );
        dag.register_episode(&ep);
    }

    let stats = dag.stats();
    assert_eq!(stats.total_episodes, 3);
    assert!(stats.total_nodes > 0);
    assert!(stats.total_edges > 0);
    assert!(stats.token_savings > 0);
}

// ============================================================================
// DagContextAssembler Tests
// ============================================================================

#[test]
fn test_assembler_creation() {
    let assembler = DagContextAssembler::empty();
    assert_eq!(assembler.dag().node_count(), 0);
}

#[test]
fn test_assembler_register_and_assemble() {
    let mut assembler = DagContextAssembler::empty();

    let ep1 = Arc::new(create_test_episode(
        "Task 1",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "Task 2",
        TaskType::Refactoring,
        rust_web_context(),
    ));

    assembler.register_episodes(&[ep1.clone(), ep2.clone()]);

    let assembled = assembler.assemble(&[ep1, ep2]);

    assert!(!assembled.shared_context.is_empty());
    assert!(assembled.token_savings > 0);
}

#[test]
fn test_assembler_token_efficient_config() {
    let config = DagAssemblyConfig::token_efficient();
    assert_eq!(config.max_unique_items, 10);
    assert!(config.deduplicate_shared);
}

#[test]
fn test_assembler_shared_context_extraction() {
    let mut assembler = DagContextAssembler::empty();

    let ep1 = Arc::new(create_test_episode(
        "Task 1",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "Task 2",
        TaskType::Refactoring,
        rust_web_context(),
    ));

    assembler.register_episodes(&[ep1.clone(), ep2.clone()]);
    let assembled = assembler.assemble(&[ep1, ep2]);

    // Should extract shared language, domain, etc.
    let shared_types: Vec<_> = assembled
        .shared_context
        .iter()
        .map(|s| s.node_type)
        .collect();
    assert!(shared_types.contains(&StateNodeType::Language));
    assert!(shared_types.contains(&StateNodeType::Domain));
}

#[test]
fn test_assembler_unique_context() {
    let mut assembler = DagContextAssembler::empty();

    let ep1 = Arc::new(create_test_episode(
        "Fix auth bug",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "Add logging",
        TaskType::CodeGeneration,
        rust_cli_context(),
    ));

    assembler.register_episodes(&[ep1.clone(), ep2.clone()]);
    let assembled = assembler.assemble(&[ep1, ep2]);

    // Each episode should have unique context for its unique description
    assert_eq!(assembled.unique_context.len(), 2);
}

#[test]
fn test_assembler_reduction_calculation() {
    let mut assembler = DagContextAssembler::empty();

    // Create episodes with same shared context
    let episodes: Vec<Arc<Episode>> = (0..5)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    assembler.register_episodes(&episodes);
    let assembled = assembler.assemble(&episodes);

    let reduction = assembler.reduction_percentage(&assembled);
    // With 5 episodes sharing same context, should have high reduction
    assert!(reduction > 50.0);
}

#[test]
fn test_assembler_format_compact() {
    let config = DagAssemblyConfig {
        format: AssemblyFormat::Compact,
        ..DagAssemblyConfig::default()
    };
    let mut assembler = DagContextAssembler::with_config(StateDag::new(), config);

    let ep = Arc::new(create_test_episode(
        "Task",
        TaskType::Debugging,
        rust_web_context(),
    ));
    assembler.register_episodes(std::slice::from_ref(&ep));
    let assembled = assembler.assemble(&[ep]);

    let formatted = assembler.format_for_prompt(&assembled);
    // Compact format should be short
    assert!(formatted.len() < 500);
}

#[test]
fn test_assembler_format_full() {
    let config = DagAssemblyConfig {
        format: AssemblyFormat::Full,
        ..DagAssemblyConfig::default()
    };
    let mut assembler = DagContextAssembler::with_config(StateDag::new(), config);

    let ep = Arc::new(create_test_episode(
        "Task",
        TaskType::Debugging,
        rust_web_context(),
    ));
    assembler.register_episodes(std::slice::from_ref(&ep));
    let assembled = assembler.assemble(&[ep]);

    let formatted = assembler.format_for_prompt(&assembled);
    // Full format should have sections
    assert!(formatted.contains("Shared Context"));
    assert!(formatted.contains("Episode Context"));
}

#[test]
fn test_assembler_format_optimized() {
    let config = DagAssemblyConfig {
        format: AssemblyFormat::TokenOptimized,
        ..DagAssemblyConfig::default()
    };
    let mut assembler = DagContextAssembler::with_config(StateDag::new(), config);

    let ep1 = Arc::new(create_test_episode(
        "Task 1",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "Task 2",
        TaskType::Refactoring,
        rust_web_context(),
    ));

    assembler.register_episodes(&[ep1.clone(), ep2.clone()]);
    let assembled = assembler.assemble(&[ep1, ep2]);

    let formatted = assembler.format_for_prompt(&assembled);
    // Optimized format should be minimal
    assert!(formatted.contains("SHARED:"));
    assert!(formatted.contains("EP:"));
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_dag_integration_with_bundle_accumulator() {
    use crate::context::{BundleAccumulator, ContextItem};

    // Create episodes
    let ep1 = Arc::new(create_test_episode(
        "Task 1",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "Task 2",
        TaskType::Refactoring,
        rust_web_context(),
    ));
    let ep3 = Arc::new(create_test_episode(
        "Task 3",
        TaskType::CodeGeneration,
        python_data_context(),
    ));

    // Create bundle from episodes
    let mut accumulator = BundleAccumulator::default_config();
    for ep in [&ep1, &ep2, &ep3] {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.8));
    }
    let bundle = accumulator.to_bundle();

    // Now assemble with DAG
    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(
        &bundle
            .iter()
            .filter_map(|i| i.as_episode())
            .cloned()
            .collect::<Vec<_>>(),
    );

    let assembled = assembler.assemble(
        &bundle
            .iter()
            .filter_map(|i| i.as_episode())
            .cloned()
            .collect::<Vec<_>>(),
    );

    // Should have token savings
    assert!(assembled.token_savings > 0);
}

#[test]
fn test_dag_clear() {
    let mut dag = StateDag::new();

    for i in 0..3 {
        let ep = create_test_episode(
            &format!("Task {i}"),
            TaskType::Debugging,
            rust_web_context(),
        );
        dag.register_episode(&ep);
    }

    assert!(dag.node_count() > 0);

    dag.clear();

    assert_eq!(dag.node_count(), 0);
    assert_eq!(dag.edge_count(), 0);
}

// ============================================================================
// Full Pipeline Integration Tests (Accumulator → DAG → Formatted Prompt)
// ============================================================================

/// Test the full pipeline: episodes → accumulator → DAG → token-optimized prompt.
#[test]
fn test_full_pipeline_token_optimized() {
    use crate::context::{BundleAccumulator, ContextItem};

    // Step 1: Create episodes with shared context
    let eps: Vec<Arc<Episode>> = (0..8)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    // Step 2: Feed through BundleAccumulator (simulates retrieval + accumulation)
    let mut accumulator = BundleAccumulator::default_config();
    for ep in &eps {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.8));
    }
    let bundle = accumulator.to_bundle();

    // Step 3: Extract episodes from bundle and register in DAG
    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(&bundle_eps);

    // Step 4: Assemble context
    let assembled = assembler.assemble(&bundle_eps);

    // Step 5: Format for downstream prompt
    let prompt = assembler.format_for_prompt(&assembled);

    // Assertions
    assert!(!prompt.is_empty(), "Formatted prompt should not be empty");
    assert!(
        prompt.contains("SHARED:"),
        "Token-optimized format should have SHARED section"
    );
    assert!(
        prompt.contains("EP:"),
        "Token-optimized format should have EP entries"
    );
    assert!(
        assembled.token_savings > 0,
        "Shared context should produce token savings"
    );
    assert!(
        assembler.reduction_percentage(&assembled) > 40.0,
        "8 episodes sharing context should yield >40% reduction"
    );
}

/// Test the full pipeline with mixed contexts (some shared, some unique).
#[test]
fn test_full_pipeline_mixed_contexts() {
    use crate::context::{BundleAccumulator, ContextItem};

    // Create episodes with mixed contexts
    let rust_eps: Vec<Arc<Episode>> = (0..5)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Rust task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    let python_eps: Vec<Arc<Episode>> = (0..3)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Python task {i}"),
                TaskType::CodeGeneration,
                python_data_context(),
            ))
        })
        .collect();

    // Feed through accumulator
    let mut accumulator = BundleAccumulator::default_config();
    for ep in rust_eps.iter().chain(python_eps.iter()) {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.75));
    }
    let bundle = accumulator.to_bundle();

    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    // Assemble
    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    // Verify shared context exists within the rust subgroup
    let rust_ids: Vec<Uuid> = rust_eps.iter().map(|e| e.episode_id).collect();
    let shared_rust = assembler.dag().get_shared_context(&rust_ids);
    let rust_shared_languages: Vec<_> = shared_rust
        .iter()
        .filter(|n| n.node_type == StateNodeType::Language)
        .collect();
    assert!(
        !rust_shared_languages.is_empty(),
        "Rust episodes should share language node"
    );

    // Verify shared context exists within the python subgroup
    let python_ids: Vec<Uuid> = python_eps.iter().map(|e| e.episode_id).collect();
    let shared_python = assembler.dag().get_shared_context(&python_ids);
    let python_shared_languages: Vec<_> = shared_python
        .iter()
        .filter(|n| n.node_type == StateNodeType::Language)
        .collect();
    assert!(
        !python_shared_languages.is_empty(),
        "Python episodes should share language node"
    );

    // Each episode should have unique context entry
    assert_eq!(
        assembled.unique_context.len(),
        8,
        "All 8 episodes should have unique context"
    );

    // Verify token savings exist
    assert!(
        assembled.token_savings > 0,
        "Mixed contexts should still produce savings"
    );
}

/// Test the full pipeline with many episodes to verify high reduction ratios.
#[test]
fn test_full_pipeline_high_reduction() {
    use crate::context::{BundleAccumulator, ContextItem};

    // 20 episodes all sharing the same context
    let eps: Vec<Arc<Episode>> = (0..20)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Shared task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    let mut accumulator = BundleAccumulator::comprehensive();
    for ep in &eps {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.8));
    }
    let bundle = accumulator.to_bundle();

    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    let reduction = assembler.reduction_percentage(&assembled);
    assert!(
        reduction > 50.0,
        "20 episodes sharing context should yield >50% reduction, got {reduction:.1}%"
    );

    // Verify the prompt is significantly smaller than full context would be
    let prompt = assembler.format_for_prompt(&assembled);
    let prompt_tokens_approx = prompt.len() / 4;
    let estimated_original_tokens = 20 * (20 + 20 + 10); // ~50 tokens per episode × 20
    assert!(
        prompt_tokens_approx < estimated_original_tokens,
        "Prompt tokens ({prompt_tokens_approx}) should be less than full context ({estimated_original_tokens})"
    );
}

/// Test the pipeline with a single episode (no deduplication possible).
#[test]
fn test_full_pipeline_single_episode() {
    use crate::context::{BundleAccumulator, ContextItem};

    let ep = Arc::new(create_test_episode(
        "Single task",
        TaskType::Debugging,
        rust_web_context(),
    ));

    let mut accumulator = BundleAccumulator::default_config();
    accumulator.add(ContextItem::from_episode(Arc::clone(&ep), 0.9));
    let bundle = accumulator.to_bundle();

    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    // Single episode should still produce valid output
    let prompt = assembler.format_for_prompt(&assembled);
    assert!(!prompt.is_empty());
    assert!(prompt.contains("EP:"));

    // Single episode has no sharing, so savings are minimal (approximation noise)
    assert!(
        assembled.token_savings <= 10,
        "Single episode should have negligible token savings, got {}",
        assembled.token_savings
    );
    // Reduction percentage is not meaningful without sharing
    assert!(
        assembler.reduction_percentage(&assembled) < 25.0,
        "Single episode should have low reduction, got {:.1}%",
        assembler.reduction_percentage(&assembled)
    );
}

/// Test the pipeline with an empty bundle (edge case).
#[test]
fn test_full_pipeline_empty_bundle() {
    let assembler = DagContextAssembler::empty();
    let eps: Vec<Arc<Episode>> = Vec::new();

    let assembled = assembler.assemble(&eps);

    // Should produce empty but valid output
    assert!(assembled.shared_context.is_empty());
    assert!(assembled.unique_context.is_empty());
    assert_eq!(assembled.estimated_tokens, 0);
    assert_eq!(assembled.token_savings, 0);

    // Formatting empty context should produce valid (empty) output
    let prompt = assembler.format_for_prompt(&assembled);
    assert!(prompt.is_empty());
}

/// Test the pipeline with episodes-only extraction from a mixed bundle.
#[test]
fn test_full_pipeline_episodes_only_filter() {
    use crate::context::{BundleAccumulator, ContextItem};
    use crate::pattern::Pattern;
    use crate::pattern::PatternEffectiveness;

    // Create episodes
    let ep1 = Arc::new(create_test_episode(
        "E1",
        TaskType::Debugging,
        rust_web_context(),
    ));
    let ep2 = Arc::new(create_test_episode(
        "E2",
        TaskType::Refactoring,
        rust_web_context(),
    ));

    // Create a pattern
    let pattern = Arc::new(Pattern::ToolSequence {
        id: uuid::Uuid::new_v4(),
        tools: vec!["git".to_string(), "cargo".to_string()],
        context: TaskContext::default(),
        success_rate: 0.9,
        avg_latency: chrono::Duration::milliseconds(50),
        occurrence_count: 5,
        effectiveness: PatternEffectiveness {
            times_retrieved: 3,
            times_applied: 3,
            success_when_applied: 2,
            failure_when_applied: 1,
            avg_reward_delta: 0.1,
            last_used: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        },
    });

    // Feed mixed items through accumulator
    let mut accumulator = BundleAccumulator::default_config();
    accumulator.add(ContextItem::from_episode(ep1.clone(), 0.85));
    accumulator.add(ContextItem::from_pattern(pattern, 0.6));
    accumulator.add(ContextItem::from_episode(ep2.clone(), 0.75));
    let bundle = accumulator.to_bundle();

    // Extract only episodes (filter out patterns)
    let episode_count = bundle
        .iter()
        .filter(|item| item.as_episode().is_some())
        .count();
    assert_eq!(
        episode_count, 2,
        "Should have 2 episodes (1 pattern filtered)"
    );

    // Verify episodes_only() works
    let only_eps = accumulator.episodes_only();
    assert_eq!(only_eps.len(), 2);

    // DAG should only process episodes
    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    let mut assembler = DagContextAssembler::empty();
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    assert_eq!(assembled.unique_context.len(), 2);
    assert!(!assembled.shared_context.is_empty());
}

/// Test pipeline with full context config (no deduplication).
#[test]
fn test_full_pipeline_no_dedup_config() {
    use crate::context::{BundleAccumulator, ContextItem};

    let eps: Vec<Arc<Episode>> = (0..5)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    let mut accumulator = BundleAccumulator::default_config();
    for ep in &eps {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.8));
    }
    let bundle = accumulator.to_bundle();
    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    // Use full context config (deduplication disabled)
    let config = DagAssemblyConfig::full_context();
    let mut assembler = DagContextAssembler::with_config(StateDag::new(), config);
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    // Full context mode: no shared context extracted
    assert!(
        assembled.shared_context.is_empty(),
        "Full context mode should not deduplicate"
    );
    assert_eq!(assembled.unique_context.len(), 5);

    let prompt = assembler.format_for_prompt(&assembled);
    assert!(
        prompt.contains("Shared Context") || prompt.contains("Episode Context"),
        "Full format should contain context sections"
    );
}

/// Test pipeline with token-efficient config end-to-end.
#[test]
fn test_full_pipeline_token_efficient_end_to_end() {
    use crate::context::{BundleAccumulator, ContextItem};

    // Use token-efficient accumulator (max 10 items, higher threshold)
    let eps: Vec<Arc<Episode>> = (0..12)
        .map(|i| {
            Arc::new(create_test_episode(
                &format!("Task {i}"),
                TaskType::Debugging,
                rust_web_context(),
            ))
        })
        .collect();

    let mut accumulator = BundleAccumulator::token_efficient();
    for ep in &eps {
        accumulator.add(ContextItem::from_episode(Arc::clone(ep), 0.7));
    }
    let bundle = accumulator.to_bundle();
    // Token-efficient config limits to 10 items
    assert!(bundle.len() <= 10);

    let bundle_eps: Vec<Arc<Episode>> = bundle
        .iter()
        .filter_map(|item| item.as_episode().cloned())
        .collect();

    // Use token-efficient assembler config
    let config = DagAssemblyConfig::token_efficient();
    let mut assembler = DagContextAssembler::with_config(StateDag::new(), config);
    assembler.register_episodes(&bundle_eps);
    let assembled = assembler.assemble(&bundle_eps);

    let prompt = assembler.format_for_prompt(&assembled);
    assert!(prompt.contains("SHARED:"));
    assert!(prompt.contains("EP:"));

    // With token-efficient mode, unique items should be capped
    assert!(
        assembled.unique_context.len() <= 10,
        "Token-efficient should cap unique items"
    );
}
