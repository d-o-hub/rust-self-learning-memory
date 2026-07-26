//! Integration tests for embedding MCP tools
#![allow(clippy::expect_used)]

use do_memory_core::SelfLearningMemory;
use do_memory_mcp::mcp::tools::embeddings::{
    ConfigureEmbeddingsInput, EmbeddingProviderStatusInput, EmbeddingTools,
    QuerySemanticMemoryInput, configure_embeddings_tool, query_semantic_memory_tool,
    test_embeddings_tool,
};
use do_memory_mcp::server::MemoryMCPServer;
use do_memory_mcp::types::SandboxConfig;
use std::sync::Arc;

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
            "Azure provider '{}' should be rejected",
            provider_name
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Azure provider is not supported"),
            "Expected 'Azure provider is not supported' in error for '{}', got: {}",
            provider_name,
            err_msg
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
        "Expected 'Custom provider is not supported', got: {}",
        err_msg
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
    assert!(!output.message.is_empty());
    assert!(!output.errors.is_empty());
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
    assert!(!configure_tool.description.is_empty());

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
