//! MCP attribution behavior tests (ADR-080 §1–3, ADR-081).
//!
//! Verifies the MCP surface parity contract for recommendation attribution:
//! strict `episode_id` handling (no silent degrade into the legacy path),
//! attributed response envelopes carrying a truthful `PersistenceReceipt`,
//! and the preserved legacy unattributed response shape.

#![allow(clippy::doc_markdown)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unwrap_used)]
// The `#[path]` include of the full bin `feature_handlers.rs` pulls in every
// handler function; only a subset is exercised here. Keep the included file
// warn-free under CI's `RUSTFLAGS="-D warnings"` and the workspace's
// `clippy::all = deny`.
#![allow(dead_code)]
// The included bin module is lint-clean only because the `server_impl` bin
// crate root carries this exact clippy allow-list (workspace sets
// `clippy::all = deny`). Replicate that leeway for this test crate so the
// real handler compiles verbatim and stays -D-warnings clean. `panic` follows
// the repo's test-file convention.
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::single_match_else)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::similar_names)]
#![allow(clippy::unused_async)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::inefficient_to_string)]
#![allow(clippy::manual_string_new)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::if_not_else)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::panic)]

use do_memory_core::{ComplexityLevel, SelfLearningMemory, TaskContext, TaskType};
use do_memory_mcp::mcp::tools::pattern_search::RecommendPatternsInput;
use do_memory_mcp::{MemoryMCPServer, SandboxConfig};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

// ────────────────────────────────────────────────────────────────────────
// Bin-side shims
//
// `feature_handlers.rs` lives in the binary crate (`server_impl`), which the
// library does not re-export. To exercise the real handler we compile that
// file into this test crate via `#[path]`; its `super::` imports resolve
// against this crate root. `get_client_id` and `json_value_len` are
// byte-for-byte the private helpers from `server_impl/tools/mod.rs`, and
// `Content`/`MemoryMCPServer`/`Value` are the same types the handler uses in
// the binary build.
// ────────────────────────────────────────────────────────────────────────

#[path = "../src/bin/server_impl/types.rs"]
mod server_impl_types;

use server_impl_types::Content;

fn get_client_id(args: &Value) -> String {
    args.get("client_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "anonymous".to_string())
}

fn json_value_len(value: &Value) -> usize {
    value.as_array().map(|a| a.len()).unwrap_or(0)
}

#[path = "../src/bin/server_impl/tools/feature_handlers.rs"]
mod feature_handlers;

// ────────────────────────────────────────────────────────────────────────
// Test helpers
// ────────────────────────────────────────────────────────────────────────

async fn make_server(memory: Arc<SelfLearningMemory>) -> MemoryMCPServer {
    MemoryMCPServer::new(SandboxConfig::restrictive(), memory)
        .await
        .expect("server creation should succeed")
}

async fn create_episode(memory: &SelfLearningMemory) -> Uuid {
    memory
        .start_episode(
            "Attribution test episode".to_string(),
            TaskContext {
                domain: "web-api".to_string(),
                language: Some("rust".to_string()),
                framework: None,
                complexity: ComplexityLevel::Simple,
                tags: vec!["attribution".to_string()],
            },
            TaskType::CodeGeneration,
        )
        .await
}

fn content_text(contents: &[Content]) -> &str {
    match contents {
        [Content::Text { text }] => text,
        other => panic!("expected a single text content, got {other:?}"),
    }
}

fn playbook_args(episode_id: Option<Uuid>) -> Value {
    let mut args = json!({
        "task_description": "Implement JWT authentication",
        "domain": "web-api",
        "task_type": "code_generation",
        "max_steps": 5
    });
    if let Some(id) = episode_id {
        args["episode_id"] = json!(id.to_string());
    }
    args
}

// ────────────────────────────────────────────────────────────────────────
// recommend_playbook: strict episode_id handling (plan section E)
// ────────────────────────────────────────────────────────────────────────

/// A present but malformed `episode_id` must be a tool error, never a silent
/// fallback to the unattributed legacy path.
#[tokio::test]
async fn malformed_playbook_episode_id_is_tool_error() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = make_server(memory).await;

    let args = playbook_args(None);
    let args = {
        let mut a = args;
        a["episode_id"] = json!("not-a-uuid");
        a
    };

    let result = feature_handlers::handle_recommend_playbook(&mut server, Some(args)).await;
    assert!(
        result.is_err(),
        "malformed episode_id must not degrade into an unattributed success"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Invalid episode UUID") || err.contains("episode_id"),
        "unexpected error message: {err}"
    );
}

/// A present non-string `episode_id` must be a tool error.
#[tokio::test]
async fn playbook_non_string_episode_id_is_tool_error() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = make_server(memory).await;

    let mut args = playbook_args(None);
    args["episode_id"] = json!(12345);

    let result = feature_handlers::handle_recommend_playbook(&mut server, Some(args)).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("episode_id"),
        "unexpected error message: {err}"
    );
}

/// A valid but nonexistent episode must surface the core invalid-input error.
#[tokio::test]
async fn playbook_nonexistent_episode_returns_error() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = make_server(memory).await;

    let args = playbook_args(Some(Uuid::new_v4()));

    let result = feature_handlers::handle_recommend_playbook(&mut server, Some(args)).await;
    assert!(
        result.is_err(),
        "nonexistent episode must not create a recommendation session"
    );
}

/// A valid episode yields the attributed result: playbooks plus a session and
/// a truthful receipt. With no configured backends the receipt is memory_only
/// and still carries session_id + episode_id (ADR-080 §3).
#[tokio::test]
async fn playbook_attributed_valid_episode_returns_envelope() {
    let memory = Arc::new(SelfLearningMemory::new());
    let episode_id = create_episode(&memory).await;
    let mut server = make_server(memory).await;

    let args = playbook_args(Some(episode_id));
    let contents = feature_handlers::handle_recommend_playbook(&mut server, Some(args))
        .await
        .expect("attributed playbook call should succeed");
    let value: Value = serde_json::from_str(content_text(&contents)).unwrap();

    assert!(
        value.get("playbooks").is_some(),
        "attributed result must carry playbooks"
    );
    let session = value
        .get("session")
        .expect("attributed result must carry a session");
    assert!(
        session["session_id"].as_str().is_some(),
        "session must carry a session_id"
    );
    assert_eq!(session["episode_id"], episode_id.to_string());
    let receipt = value
        .get("receipt")
        .expect("attributed result must carry a receipt");
    assert_eq!(receipt["state"], "memory_only");
    assert_eq!(receipt["session_id"], session["session_id"]);
    assert_eq!(receipt["episode_id"], episode_id.to_string());
}

/// Without `episode_id` the legacy unattributed response shape is preserved:
/// a bare playbook array with no session/receipt/attribution envelope.
#[tokio::test]
async fn playbook_legacy_without_episode_id_has_no_envelope() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = make_server(memory).await;

    let args = playbook_args(None);
    let contents = feature_handlers::handle_recommend_playbook(&mut server, Some(args))
        .await
        .expect("legacy playbook call should succeed");
    let value: Value = serde_json::from_str(content_text(&contents)).unwrap();

    assert!(
        value.is_array(),
        "legacy response must be a bare playbook array"
    );
    assert!(
        value.get("session").is_none() && value.get("receipt").is_none(),
        "legacy response must not gain an attribution envelope"
    );
}

// ────────────────────────────────────────────────────────────────────────
// recommend_patterns: attribution envelope and legacy shape
// ────────────────────────────────────────────────────────────────────────

/// A valid episode produces the attribution envelope with session_id and a
/// memory_only receipt when no backends are configured.
#[tokio::test]
async fn recommend_patterns_attributed_envelope() {
    let memory = Arc::new(SelfLearningMemory::new());
    let episode_id = create_episode(&memory).await;
    let server = make_server(memory).await;

    let input = RecommendPatternsInput {
        task_description: "Implement JWT authentication".to_string(),
        domain: "web-api".to_string(),
        tags: vec![],
        limit: 3,
        episode_id: Some(episode_id),
    };
    let value = server
        .execute_recommend_patterns(input)
        .await
        .expect("attributed recommend_patterns should succeed");

    let attribution = value
        .get("attribution")
        .expect("attributed response must carry an attribution envelope");
    assert!(
        attribution["session_id"].as_str().is_some(),
        "attribution envelope must carry a session_id"
    );
    assert_eq!(attribution["episode_id"], episode_id.to_string());
    assert_eq!(attribution["receipt"]["state"], "memory_only");
    assert_eq!(attribution["receipt"]["episode_id"], episode_id.to_string());
}

/// Without `episode_id` the legacy response has no attribution envelope and no
/// receipt state.
#[tokio::test]
async fn recommend_patterns_legacy_shape_no_attribution() {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = make_server(memory).await;

    let input = RecommendPatternsInput {
        task_description: "Implement JWT authentication".to_string(),
        domain: "web-api".to_string(),
        tags: vec![],
        limit: 3,
        episode_id: None,
    };
    let value = server
        .execute_recommend_patterns(input)
        .await
        .expect("legacy recommend_patterns should succeed");

    assert!(
        value.get("attribution").is_none(),
        "legacy response must not carry an attribution envelope"
    );
    assert!(
        value.get("receipt").is_none() && value.get("session").is_none(),
        "legacy response must not carry receipt/session fields"
    );
}

/// A malformed `episode_id` on recommend_patterns is a hard error (the handler
/// deserializes into `Option<Uuid>`), not an unattributed success.
#[tokio::test]
async fn recommend_patterns_malformed_episode_id_is_tool_error() {
    let memory = Arc::new(SelfLearningMemory::new());
    let mut server = make_server(memory).await;

    let mut args = json!({
        "task_description": "Implement JWT authentication",
        "domain": "web-api"
    });
    args["episode_id"] = json!("not-a-uuid");

    let result = feature_handlers::handle_recommend_patterns(&mut server, Some(args)).await;
    assert!(
        result.is_err(),
        "malformed episode_id must not silently drop attribution"
    );
}

/// A valid but nonexistent episode surfaces the core invalid-input error for
/// the patterns entry point too.
#[tokio::test]
async fn recommend_patterns_nonexistent_episode_returns_error() {
    let memory = Arc::new(SelfLearningMemory::new());
    let server = make_server(memory).await;

    let input = RecommendPatternsInput {
        task_description: "Implement JWT authentication".to_string(),
        domain: "web-api".to_string(),
        tags: vec![],
        limit: 3,
        episode_id: Some(Uuid::new_v4()),
    };
    let result = server.execute_recommend_patterns(input).await;
    assert!(
        result.is_err(),
        "nonexistent episode must not create a recommendation session"
    );
}
