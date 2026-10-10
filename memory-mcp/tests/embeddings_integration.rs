//! Integration tests for embedding MCP tools
#![allow(clippy::expect_used)]

use do_memory_core::embeddings::{
    EPISODE_NAMESPACE, EmbeddingConfig, EmbeddingStorageAdapter, EmbeddingStorageBackend,
    EmbeddingStorageScope, EphemeralEmbeddingStorage, MockLocalModel, ProviderConfig,
    SemanticService,
};
use do_memory_core::episode::PatternId;
use do_memory_core::{
    Episode, Heuristic, MemoryConfig, Pattern, SelfLearningMemory, StorageBackend,
    StorageBackendCapabilities,
};
use do_memory_mcp::mcp::tools::embeddings::{
    ConfigureEmbeddingsInput, ConfigureEmbeddingsOutput, EmbeddingProviderStatusInput,
    EmbeddingTools, QuerySemanticMemoryInput, configure_embeddings_tool,
    configured_embedding_storage, query_semantic_memory_tool, test_embeddings_tool,
};
use do_memory_mcp::server::MemoryMCPServer;
use do_memory_mcp::types::SandboxConfig;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Disable WASM sandbox for all tests to prevent rquickjs GC crashes
#[allow(unsafe_code)]
fn disable_wasm_for_tests() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MCP_USE_WASM", "false");
            std::env::set_var("MCP_CACHE_WARMING_ENABLED", "false");
        }
    });
}

/// Create a test MCP server
async fn create_test_server() -> MemoryMCPServer {
    disable_wasm_for_tests();

    let memory = Arc::new(SelfLearningMemory::new());
    MemoryMCPServer::new(SandboxConfig::default(), memory)
        .await
        .expect("Failed to create test server")
}

#[tokio::test]
async fn test_embedding_tools_registered() {
    let server = create_test_server().await;

    // Load embedding extended tools (they're lazy-loaded)
    let _ = server.get_tool("configure_embeddings").await;
    let _ = server.get_tool("query_semantic_memory").await;
    let _ = server.get_tool("test_embeddings").await;

    let tools = server.list_tools().await;

    assert!(
        tools.iter().any(|t| t.name == "configure_embeddings"),
        "configure_embeddings tool should be registered"
    );
    assert!(
        tools.iter().any(|t| t.name == "query_semantic_memory"),
        "query_semantic_memory tool should be registered"
    );
    assert!(
        tools.iter().any(|t| t.name == "test_embeddings"),
        "test_embeddings tool should be registered"
    );
}

/// REA-2026-07-26 A5: Local provider activation attempt.
///
/// In CI, `LocalEmbeddingProvider` uses a mock/degraded model because the
/// sentence-transformers model file is not available.  `build_exact` rejects
/// degraded-mock providers, so the call returns an error.  This test covers
/// both possible outcomes:
///
/// - **If a real local model IS available** (feature = `local-embeddings` and
///   model downloaded): `configure_embeddings` succeeds and `activation_revision`
///   is `Some(1)`, `provider_health` is `"active"`.
/// - **If no real model is available** (CI default): the call returns an error
///   because `build_exact` refuses to install a mock provider.
#[tokio::test]
async fn test_configure_embeddings_local_provider() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
        api_key_env: None,
        similarity_threshold: Some(0.75),
        batch_size: Some(16),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;

    match result {
        Ok(output) => {
            // Real model available — activation must be live.
            assert!(output.success, "success flag must be true");
            assert_eq!(output.provider, "local");
            assert_eq!(output.model, "sentence-transformers/all-MiniLM-L6-v2");
            assert_eq!(output.dimension, 384);
            assert_eq!(
                output.provider_health, "active",
                "provider_health must be 'active' after real activation"
            );
            assert!(
                output.activation_revision.is_some(),
                "activation_revision must be set after activation"
            );
        }
        Err(e) => {
            // No real model in this environment — build_exact correctly rejected.
            let msg = e.to_string();
            assert!(
                msg.contains("activation failed")
                    || msg.contains("not production-ready")
                    || msg.contains("Local embedding model unavailable"),
                "Unexpected error for missing local model: {msg}"
            );
        }
    }
}

#[tokio::test]
async fn test_configure_embeddings_openai_models() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    // Test text-embedding-3-small — will fail without a real API key but must
    // not panic and must fail with a specific error (missing cred or probe fail).
    let input_small = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result_small = tools.execute_configure_embeddings(input_small).await;
    match result_small {
        Ok(output) => {
            assert_eq!(output.model, "text-embedding-3-small");
            assert_eq!(output.dimension, 1536);
            assert_eq!(output.provider_health, "active");
        }
        Err(e) => {
            // Expected without a real key set.
            let msg = e.to_string();
            assert!(
                msg.contains("not set")
                    || msg.contains("activation failed")
                    || msg.contains("probe failed"),
                "Unexpected error: {msg}"
            );
        }
    }

    // Test text-embedding-3-large
    let input_large = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-large".to_string()),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result_large = tools.execute_configure_embeddings(input_large).await;
    if let Ok(output) = result_large {
        assert_eq!(output.model, "text-embedding-3-large");
        assert_eq!(output.dimension, 3072);
    }
}

#[tokio::test]
async fn test_configure_embeddings_mistral() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "mistral".to_string(),
        model: Some("mistral-embed".to_string()),
        api_key_env: Some("MISTRAL_API_KEY".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    match result {
        Ok(output) => {
            assert_eq!(output.model, "mistral-embed");
            assert_eq!(output.dimension, 1024);
            assert_eq!(output.provider_health, "active");
        }
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("not set")
                    || msg.contains("activation failed")
                    || msg.contains("not enabled"),
                "Unexpected error: {msg}"
            );
        }
    }
}

#[tokio::test]
async fn test_configure_embeddings_azure_rejected() {
    // REA-2026-07-26 A1: Azure provider is no longer selectable.
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    for provider_name in &["azure", "azure_openai"] {
        let input = ConfigureEmbeddingsInput {
            provider: (*provider_name).to_string(),
            model: None,
            api_key_env: Some("AZURE_OPENAI_API_KEY".to_string()),
            similarity_threshold: None,
            batch_size: None,
            base_url: None,
            api_version: Some("2023-05-15".to_string()),
            resource_name: Some("my-resource".to_string()),
            deployment_name: Some("my-deployment".to_string()),
        };

        let result = tools.execute_configure_embeddings(input).await;
        assert!(
            result.is_err(),
            "Azure provider '{provider_name}' should be rejected"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Azure provider is not supported"),
            "Expected 'Azure provider is not supported' in error for '{provider_name}', got: {err_msg}"
        );
    }
}

#[tokio::test]
async fn test_configure_embeddings_custom_rejected() {
    // REA-2026-07-26 A1: Custom provider is no longer selectable.
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "custom".to_string(),
        model: Some("my-model".to_string()),
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: Some("https://my-endpoint.example.com".to_string()),
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err(), "Custom provider should be rejected");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Custom provider is not supported"),
        "Expected 'Custom provider is not supported', got: {err_msg}"
    );
}

// ── REA-2026-07-26 A5: True-activation regression tests ──────────────────────
//
// After A5, configure_embeddings either:
//   a) succeeds and immediately activates the provider (activation_revision=Some,
//      provider_health="active", and subsequent status/generate reflect it), or
//   b) fails (no state change to the prior activation).
//
// The false-success behavior (configure returns Ok + activation_revision=None)
// is no longer possible.

/// When `configure_embeddings` returns Ok, the provider must be immediately live:
/// status must show configured=true and `activation_revision` must be Some.
///
/// This test uses a cloud provider without a real key, so it will fail at
/// `build_exact` and demonstrate failure preservation (status stays unconfigured).
#[tokio::test]
async fn test_configure_failure_preserves_unconfigured_status() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(Arc::clone(&memory));

    // Attempt to configure OpenAI without providing a key env var that exists.
    // This will fail at the credential check or probe stage.
    let configure_input = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some("__NONEXISTENT_KEY_RSLM_TEST__".to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let configure_result = tools.execute_configure_embeddings(configure_input).await;
    // Must fail — the env var does not exist.
    assert!(
        configure_result.is_err(),
        "configure must fail when the credential env var is not set"
    );

    // After the failed configure, the status must still report unconfigured —
    // the failure must not have corrupted the prior (None) activation.
    let status_result = tools
        .execute_embedding_provider_status(EmbeddingProviderStatusInput {
            test_connectivity: false,
        })
        .await;
    assert!(status_result.is_ok());
    let status_output = status_result.unwrap();

    assert!(
        !status_output.configured,
        "status must remain unconfigured after a failed configure attempt"
    );
}

/// A successful configure must produce `activation_revision` = `Some` and
/// `provider_health` = `"active"`.  Because the Local provider requires a real
/// model that may not be present in CI, we only assert the invariants when
/// configure returns Ok.
#[tokio::test]
async fn test_successful_configure_reports_active_and_revision() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;

    if let Ok(output) = result {
        // If configure succeeded, these invariants must hold:
        assert!(output.success, "success flag must be true on Ok result");
        assert_eq!(
            output.provider_health, "active",
            "provider_health must be 'active' after real activation (was '{}')",
            output.provider_health
        );
        assert!(
            output.activation_revision.is_some(),
            "activation_revision must be Some after successful activation"
        );
        assert_eq!(
            output.activation_revision,
            Some(1),
            "first activation must have revision=1"
        );
    }
    // If configure failed (no real model in CI), the test is vacuously passing —
    // the failure path is covered by test_configure_failure_preserves_unconfigured_status.
}

/// Cohere provider must still be rejected (regression from before A1).
#[tokio::test]
async fn test_configure_embeddings_cohere_rejected() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "cohere".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err(), "Cohere provider should be rejected");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cohere provider is not implemented")
    );
}

#[tokio::test]
async fn test_configure_embeddings_invalid_provider() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "invalid-provider".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    assert!(result.is_err(), "Invalid provider should fail");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Unsupported provider")
    );
}

#[tokio::test]
async fn test_query_semantic_memory_basic() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = QuerySemanticMemoryInput {
        query: "implement REST API".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.8),
        domain: Some("web-api".to_string()),
        task_type: Some("code_generation".to_string()),
    };

    let result = tools.execute_query_semantic_memory(input).await;
    assert!(result.is_ok(), "Query should succeed");

    let output = result.unwrap();
    assert!(
        output.query_time_ms > 0.0,
        "Query should have measurable time"
    );
    assert_eq!(output.embedding_dimension, 384);
}

#[tokio::test]
async fn test_query_semantic_memory_with_filters() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input_domain = QuerySemanticMemoryInput {
        query: "parse JSON data".to_string(),
        limit: Some(10),
        similarity_threshold: Some(0.7),
        domain: Some("data-processing".to_string()),
        task_type: None,
    };

    let result = tools.execute_query_semantic_memory(input_domain).await;
    assert!(result.is_ok());

    let input_task = QuerySemanticMemoryInput {
        query: "debug performance issue".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.75),
        domain: None,
        task_type: Some("debugging".to_string()),
    };

    let result = tools.execute_query_semantic_memory(input_task).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_query_semantic_memory_default_params() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = QuerySemanticMemoryInput {
        query: "test query".to_string(),
        limit: None,
        similarity_threshold: None,
        domain: None,
        task_type: None,
    };

    let result = tools.execute_query_semantic_memory(input).await;
    assert!(result.is_ok());

    let output = result.unwrap();
    assert!(output.results_found <= 10);
}

#[tokio::test]
async fn test_test_embeddings_tool() {
    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let result = tools.execute_test_embeddings().await;
    assert!(result.is_ok(), "Test embeddings should succeed");

    let output = result.unwrap();
    assert!(!output.available, "Should not be available by default");
    assert_eq!(output.provider, "not-configured");
    assert_eq!(output.dimension, 384);
    assert_eq!(output.sample_embedding.len(), 0);
    assert_ne!(output.message.len(), 0);
    assert_ne!(output.errors.len(), 0);
}

#[tokio::test]
async fn test_server_execute_configure_embeddings() {
    let server = create_test_server().await;

    let input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: Some(0.8),
        batch_size: Some(32),
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    // configure_embeddings now either succeeds (real model) or fails (CI).
    // The server call should never panic regardless.
    let result = server.execute_configure_embeddings(input).await;
    // Result may be Ok or Err — we only assert it doesn't panic.
    drop(result);
}

#[tokio::test]
async fn test_server_execute_query_semantic_memory() {
    let server = create_test_server().await;

    let input = QuerySemanticMemoryInput {
        query: "implement feature".to_string(),
        limit: Some(5),
        similarity_threshold: Some(0.7),
        domain: None,
        task_type: None,
    };

    let result = server.execute_query_semantic_memory(input).await;
    assert!(result.is_ok(), "Server execution should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");
    assert!(output.get("results_found").is_some());
    assert!(output.get("results").is_some());
    assert!(output.get("query_time_ms").is_some());
}

#[tokio::test]
async fn test_server_execute_test_embeddings() {
    let server = create_test_server().await;

    let result = server.execute_test_embeddings().await;
    assert!(result.is_ok(), "Server execution should succeed");

    let output = result.unwrap();
    assert!(output.is_object(), "Output should be JSON object");
    assert!(output.get("available").is_some());
    assert!(output.get("provider").is_some());
    assert!(output.get("test_time_ms").is_some());
    assert!(output.get("sample_embedding").is_some());
}

#[tokio::test]
async fn test_embeddings_tool_usage_tracking() {
    let server = create_test_server().await;

    let _ = server.execute_test_embeddings().await;

    let config_input = ConfigureEmbeddingsInput {
        provider: "local".to_string(),
        model: None,
        api_key_env: None,
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };
    let _ = server.execute_configure_embeddings(config_input).await;

    let query_input = QuerySemanticMemoryInput {
        query: "test".to_string(),
        limit: None,
        similarity_threshold: None,
        domain: None,
        task_type: None,
    };
    let _ = server.execute_query_semantic_memory(query_input).await;

    let usage = server.get_tool_usage().await;
    assert!(
        usage.contains_key("test_embeddings"),
        "test_embeddings usage should be tracked"
    );
    assert!(
        usage.contains_key("configure_embeddings"),
        "configure_embeddings usage should be tracked"
    );
    assert!(
        usage.contains_key("query_semantic_memory"),
        "query_semantic_memory usage should be tracked"
    );
}

#[tokio::test]
async fn test_tool_definitions_json_rpc_compliant() {
    let configure_tool = configure_embeddings_tool();
    assert_eq!(configure_tool.name, "configure_embeddings");
    assert_ne!(configure_tool.description.len(), 0);

    let schema = configure_tool.input_schema;
    assert!(schema.is_object());

    let obj = schema.as_object().unwrap();
    assert!(obj.contains_key("type"));
    assert!(obj.contains_key("properties"));
    assert!(obj.contains_key("required"));

    let required = obj.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&serde_json::json!("provider")));

    let query_tool = query_semantic_memory_tool();
    let schema = query_tool.input_schema.as_object().unwrap();
    let required = schema.get("required").unwrap().as_array().unwrap();
    assert!(required.contains(&serde_json::json!("query")));

    let test_tool = test_embeddings_tool();
    let schema = test_tool.input_schema.as_object().unwrap();
    let properties = schema.get("properties").unwrap().as_object().unwrap();
    assert!(properties.is_empty());
}

/// REA-2026-07-26-A6: credential redaction regression (structural).
///
/// The activation output type must have **no field capable of carrying a
/// credential**.  Assert the serialized field set is exactly the known
/// non-secret contract so that any future field that could hold a key (for
/// example `api_key`, `resolved_secret`) fails this test.  Per ADR-077 §6 the
/// resolved key is read at activation time only and is never stored in config,
/// output, warnings, or audit fields.
///
/// This test is deterministic and never mutates the process environment.
#[tokio::test]
async fn test_configure_embeddings_output_contract_has_no_credential_field() {
    let output = ConfigureEmbeddingsOutput {
        success: true,
        provider: "openai".to_string(),
        model: "text-embedding-3-small".to_string(),
        dimension: 1536,
        message: "Activated openai provider with model text-embedding-3-small (dimension: 1536, revision: 1)".to_string(),
        warnings: vec![],
        activation_revision: Some(1),
        reindex_required: false,
        provider_health: "active".to_string(),
        storage_mode: "durable".to_string(),
        storage_scope: "emb_v1:openai:text-embedding-3-small:1536#r1".to_string(),
    };

    let json = serde_json::to_value(&output).expect("output must serialize");
    let obj = json
        .as_object()
        .expect("output must serialize to an object");

    // Exactly the public, non-secret contract — nothing else may be exposed.
    let allowed: std::collections::BTreeSet<&str> = [
        "success",
        "provider",
        "model",
        "dimension",
        "message",
        "warnings",
        "activation_revision",
        "reindex_required",
        "provider_health",
        "storage_mode",
        "storage_scope",
    ]
    .into_iter()
    .collect();
    let actual: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
    assert_eq!(
        actual, allowed,
        "activation output must expose only the known non-secret fields"
    );

    // No field name may be credential-bearing.
    for key in obj.keys() {
        let lower = key.to_lowercase();
        for needle in ["key", "secret", "token", "credential", "password", "auth"] {
            assert!(
                !lower.contains(needle),
                "field name must not be credential-bearing: {key}"
            );
        }
    }
}

/// REA-2026-07-26-A6: credential redaction regression (behavioral).
///
/// When activation fails for a missing credential, the error must name the
/// environment variable to set (usability) but must never carry credential
/// material.  The variable simply does not exist, so this drives the real
/// `execute_configure_embeddings` error path **without mutating the process
/// environment** (no `unsafe`).
///
/// The structural guarantee that the output type cannot carry a credential is
/// covered by `test_configure_embeddings_output_contract_has_no_credential_field`;
/// this test guards the error-message / usability contract.
#[tokio::test]
async fn test_configure_embeddings_credential_error_names_var_not_value() {
    const UNSET_VAR: &str = "__RSLM_UNSET_CREDENTIAL_VAR__";

    let memory = Arc::new(SelfLearningMemory::new());
    let tools = EmbeddingTools::new(memory);

    let input = ConfigureEmbeddingsInput {
        provider: "openai".to_string(),
        model: Some("text-embedding-3-small".to_string()),
        api_key_env: Some(UNSET_VAR.to_string()),
        similarity_threshold: None,
        batch_size: None,
        base_url: None,
        api_version: None,
        resource_name: None,
        deployment_name: None,
    };

    let result = tools.execute_configure_embeddings(input).await;
    let err = result.expect_err("missing credential must fail activation");
    let msg = err.to_string();

    // The error tells the operator which variable to set...
    assert!(
        msg.contains(UNSET_VAR),
        "error should name the environment variable: {msg}"
    );

    // ...and contains no credential-value material.
    for sentinel in ["sk-", "Bearer ", "key=", "token=", "secret="] {
        assert!(
            !msg.contains(sentinel),
            "error must not contain credential material '{sentinel}': {msg}"
        );
    }
}

// ── REA-2026-07-26 / #1073: identity-scoped embedding storage ────────────────
//
// `configure_embeddings` selects its store through `configured_embedding_storage`.
// These tests exercise that exact selection with a recording fake backend so the
// durable path is covered without a live provider (CI has no model/key).

/// Fake `StorageBackend` that records namespaced embedding stores/reads.
#[derive(Default)]
struct RecordingStorageBackend {
    embeddings: Mutex<HashMap<String, Vec<f32>>>,
    stores: Mutex<Vec<String>>,
    reads: Mutex<Vec<String>>,
    fail_store: AtomicBool,
}

impl RecordingStorageBackend {
    fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn stored_keys(&self) -> Vec<String> {
        self.stores.lock().expect("stores lock").clone()
    }

    fn read_keys(&self) -> Vec<String> {
        self.reads.lock().expect("reads lock").clone()
    }

    fn set_fail_store(&self, fail: bool) {
        self.fail_store.store(fail, Ordering::SeqCst);
    }
}

impl StorageBackendCapabilities for RecordingStorageBackend {}

#[async_trait::async_trait]
impl StorageBackend for RecordingStorageBackend {
    async fn store_episode(&self, _episode: &Episode) -> Result<(), do_memory_core::Error> {
        Ok(())
    }

    async fn get_episode(&self, _id: uuid::Uuid) -> Result<Option<Episode>, do_memory_core::Error> {
        Ok(None)
    }

    async fn delete_episode(&self, _id: uuid::Uuid) -> Result<(), do_memory_core::Error> {
        Ok(())
    }

    async fn store_pattern(&self, _pattern: &Pattern) -> Result<(), do_memory_core::Error> {
        Ok(())
    }

    async fn get_pattern(&self, _id: PatternId) -> Result<Option<Pattern>, do_memory_core::Error> {
        Ok(None)
    }

    async fn store_heuristic(&self, _heuristic: &Heuristic) -> Result<(), do_memory_core::Error> {
        Ok(())
    }

    async fn get_heuristic(
        &self,
        _id: uuid::Uuid,
    ) -> Result<Option<Heuristic>, do_memory_core::Error> {
        Ok(None)
    }

    async fn query_episodes_since(
        &self,
        _since: chrono::DateTime<chrono::Utc>,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>, do_memory_core::Error> {
        Ok(Vec::new())
    }

    async fn query_episodes_by_metadata(
        &self,
        _key: &str,
        _value: &str,
        _limit: Option<usize>,
    ) -> Result<Vec<Episode>, do_memory_core::Error> {
        Ok(Vec::new())
    }

    async fn store_embedding(
        &self,
        id: &str,
        embedding: Vec<f32>,
    ) -> Result<(), do_memory_core::Error> {
        if self.fail_store.load(Ordering::SeqCst) {
            return Err(do_memory_core::Error::Storage(
                "injected store failure".into(),
            ));
        }
        self.stores
            .lock()
            .expect("stores lock")
            .push(id.to_string());
        self.embeddings
            .lock()
            .expect("embeddings lock")
            .insert(id.to_string(), embedding);
        Ok(())
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<Vec<f32>>, do_memory_core::Error> {
        self.reads.lock().expect("reads lock").push(id.to_string());
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .get(id)
            .cloned())
    }

    async fn delete_embedding(&self, id: &str) -> Result<bool, do_memory_core::Error> {
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .remove(id)
            .is_some())
    }

    async fn store_embeddings_batch(
        &self,
        embeddings: Vec<(String, Vec<f32>)>,
    ) -> Result<(), do_memory_core::Error> {
        for (id, embedding) in embeddings {
            self.store_embedding(&id, embedding).await?;
        }
        Ok(())
    }

    async fn get_embeddings_batch(
        &self,
        ids: &[String],
    ) -> Result<Vec<Option<Vec<f32>>>, do_memory_core::Error> {
        let guard = self.embeddings.lock().expect("embeddings lock");
        Ok(ids.iter().map(|id| guard.get(id).cloned()).collect())
    }

    async fn list_embedding_ids(&self) -> Result<Vec<String>, do_memory_core::Error> {
        Ok(self
            .embeddings
            .lock()
            .expect("embeddings lock")
            .keys()
            .cloned()
            .collect())
    }
}

fn test_scope(identity: &str, revision: u64) -> EmbeddingStorageScope {
    EmbeddingStorageScope::new(identity, revision)
}

/// With storage configured, the MCP selection must produce a durable adapter
/// that writes namespaced keys into the configured backend.
#[tokio::test]
async fn test_configured_embedding_storage_uses_configured_backend() {
    let backend = RecordingStorageBackend::new();
    let memory =
        SelfLearningMemory::with_storage(MemoryConfig::default(), backend.clone(), backend.clone());
    let provider = ProviderConfig::openai_3_small();

    let selected = configured_embedding_storage(&memory, &provider);

    assert!(
        selected.mode.is_durable(),
        "configured storage must be durable"
    );
    assert_eq!(selected.mode.label(), "durable");
    assert_eq!(
        selected.mode.scope().provider_identity(),
        provider.cache_identity()
    );
    assert_eq!(
        selected.mode.scope().config_revision(),
        provider.config_revision()
    );

    let episode_id = uuid::Uuid::new_v4();
    selected
        .storage
        .store_episode_embedding(episode_id, vec![1.0, 0.0])
        .await
        .expect("store through adapter");

    let expected = selected
        .mode
        .scope()
        .logical_key(EPISODE_NAMESPACE, &episode_id.to_string());
    assert_eq!(backend.stored_keys(), vec![expected.clone()]);
    assert!(
        expected.starts_with("emb_v1:"),
        "key must be schema-versioned"
    );
    assert!(
        expected.contains("openai:text-embedding-3-small:1536"),
        "key must carry the provider identity: {expected}"
    );

    assert_eq!(
        selected
            .storage
            .get_episode_embedding(episode_id)
            .await
            .unwrap(),
        Some(vec![1.0, 0.0])
    );
    assert!(backend.read_keys().contains(&expected));
}

/// Switching providers must produce a different scope, and the new provider
/// must not be able to read vectors written under the old identity.
#[tokio::test]
async fn test_reconfiguration_cannot_read_previous_provider_vectors() {
    let backend = RecordingStorageBackend::new();
    let memory =
        SelfLearningMemory::with_storage(MemoryConfig::default(), backend.clone(), backend.clone());

    let episode_id = uuid::Uuid::new_v4();
    let small = configured_embedding_storage(&memory, &ProviderConfig::openai_3_small());
    small
        .storage
        .store_episode_embedding(episode_id, vec![1.0])
        .await
        .expect("store under first provider");

    let large = configured_embedding_storage(&memory, &ProviderConfig::openai_3_large());
    assert_ne!(
        small.mode.scope().key_prefix(),
        large.mode.scope().key_prefix(),
        "provider switch must change the storage scope"
    );
    assert_eq!(
        large
            .storage
            .get_episode_embedding(episode_id)
            .await
            .unwrap(),
        None,
        "a reconfigured provider must not read the previous provider's vectors"
    );

    // A bumped configuration revision under the same identity is also isolated.
    let identity = "openai:text-embedding-3-small:1536";
    let revision_one = EmbeddingStorageAdapter::new(backend.clone(), None, test_scope(identity, 1));
    let revision_two = EmbeddingStorageAdapter::new(backend.clone(), None, test_scope(identity, 2));
    let other = uuid::Uuid::new_v4();
    revision_one
        .store_episode_embedding(other, vec![0.5])
        .await
        .expect("store revision one");
    assert_eq!(
        revision_two.get_episode_embedding(other).await.unwrap(),
        None,
        "a new config revision must not read older vectors"
    );
}

/// Without storage backends the MCP selection must be explicitly ephemeral.
#[tokio::test]
async fn test_configured_embedding_storage_without_backends_is_ephemeral() {
    let memory = SelfLearningMemory::new();
    let provider = ProviderConfig::mistral_embed();
    let selected = configured_embedding_storage(&memory, &provider);

    assert!(!selected.mode.is_durable());
    assert_eq!(selected.mode.label(), "ephemeral");
    assert!(!selected.storage.is_durable());
    assert_eq!(
        selected.storage.storage_scope(),
        Some(selected.mode.scope().clone())
    );
}

/// Status output must identify ephemeral storage and restart loss truthfully.
#[tokio::test]
async fn test_status_reports_ephemeral_scope_and_restart_loss() {
    let memory = Arc::new(SelfLearningMemory::new());
    let scope = test_scope("local:mini:384", 1);
    let config = EmbeddingConfig::default();
    let service = SemanticService::new(
        Box::new(MockLocalModel::new("mock-model".to_string(), 384)),
        Box::new(EphemeralEmbeddingStorage::new(scope.clone())),
        config,
    );
    memory
        .activate_semantic_service(Arc::new(service), scope.provider_identity().to_string())
        .await;

    let tools = EmbeddingTools::new(Arc::clone(&memory));
    let output = tools
        .execute_embedding_provider_status(EmbeddingProviderStatusInput {
            test_connectivity: false,
        })
        .await
        .expect("status");

    assert!(output.configured);
    assert_eq!(output.storage_mode, "ephemeral");
    assert_eq!(
        output.storage_scope.as_deref(),
        Some(scope.key_prefix().as_str())
    );
    assert!(
        output
            .warnings
            .iter()
            .any(|w| w.contains("lost on restart")),
        "ephemeral status must warn about restart loss: {:?}",
        output.warnings
    );
}

/// A durable failure on the primary backend must surface, not be swallowed.
#[tokio::test]
async fn test_configured_embedding_storage_surfaces_primary_failure() {
    let backend = RecordingStorageBackend::new();
    let memory =
        SelfLearningMemory::with_storage(MemoryConfig::default(), backend.clone(), backend.clone());
    let selected = configured_embedding_storage(&memory, &ProviderConfig::openai_3_small());

    backend.set_fail_store(true);
    let error = selected
        .storage
        .store_episode_embedding(uuid::Uuid::new_v4(), vec![1.0])
        .await
        .expect_err("primary store failure must surface");
    assert!(error.to_string().contains("injected store failure"));
}
