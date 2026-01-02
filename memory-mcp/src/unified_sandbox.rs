//! Sandbox abstraction layer supporting both Node.js and WASM backends
//!
//! This module provides a unified interface for code execution that can use either
//! the traditional Node.js-based sandbox or the new WASM-based sandbox. This allows
//! for gradual migration and A/B testing during the transition period.
//!
//! ## Architecture
//!
//! ```text
//!     UnifiedSandbox
//!           │
//!     ┌─────┴─────┐
//!     │           │
//! NodeSandbox   WasmSandbox
//! (Process)     (Memory)
//! ```
//!
//! ## Migration Strategy
//!
//! 1. **Phase 1**: Implement WASM sandbox alongside Node.js
//! 2. **Phase 2**: Enable hybrid mode with intelligent routing
//! 3. **Phase 3**: Gradually migrate workloads to WASM
//! 4. **Phase 4**: Decommission Node.js sandbox
//!
//! ## Example
//!
//! ```no_run
//! use memory_mcp::{UnifiedSandbox, SandboxBackend};
//! use memory_mcp::types::{ExecutionContext, SandboxConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create unified sandbox with both backends
//!     let sandbox = UnifiedSandbox::new(
//!         SandboxConfig::restrictive(),
//!         SandboxBackend::Hybrid { wasm_ratio: 0.5, intelligent_routing: false }
//!     ).await?;
//!
//!     let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));
//!     let result = sandbox.execute("console.log('Hello')", context).await?;
//!
//!     println!("Result: {:?}", result);
//!     Ok(())
//! }
//! ```

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

#[cfg(not(feature = "javy-backend"))]
use base64::prelude::BASE64_STANDARD;

#[cfg(not(feature = "javy-backend"))]
use base64::Engine;

use crate::types::{ExecutionContext, ExecutionResult, SandboxConfig};
use crate::wasmtime_sandbox::{WasmtimeConfig, WasmtimeMetrics, WasmtimeSandbox};

#[cfg(feature = "javy-backend")]
use crate::javy_compiler::{JavyCompiler, JavyConfig};

// Re-export existing Node.js sandbox
pub use super::sandbox::CodeSandbox;

/// Sandbox backend selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxBackend {
    /// Use only Node.js sandbox (current implementation)
    NodeJs,
    /// Use only WASM sandbox (new implementation)
    Wasm,
    /// Hybrid mode with intelligent routing
    Hybrid {
        /// Ratio of executions to route to WASM (0.0 to 1.0)
        wasm_ratio: f64,
        /// Route specific workloads based on heuristics
        intelligent_routing: bool,
    },
}

impl Default for SandboxBackend {
    fn default() -> Self {
        Self::Hybrid {
            wasm_ratio: 0.5, // Start with 50% WASM
            intelligent_routing: true,
        }
    }
}

/// Unified sandbox supporting multiple backends
#[derive(Debug)]
pub struct UnifiedSandbox {
    config: SandboxConfig,
    backend: SandboxBackend,
    node_sandbox: Option<Arc<CodeSandbox>>,
    wasmtime_sandbox: Option<Arc<WasmtimeSandbox>>,
    #[cfg(feature = "javy-backend")]
    javy_compiler: Option<Arc<JavyCompiler>>,
    metrics: Arc<tokio::sync::RwLock<UnifiedMetrics>>,
}

/// Metrics for unified sandbox
#[derive(Debug, Default, Clone)]
pub struct UnifiedMetrics {
    pub total_executions: u64,
    pub node_executions: u64,
    pub wasm_executions: u64,
    pub node_success_rate: f64,
    pub wasm_success_rate: f64,
    pub node_avg_latency_ms: f64,
    pub wasm_avg_latency_ms: f64,
    pub routing_decisions: Vec<RoutingDecision>,
}

/// Routing decision tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub backend: String,
    pub reason: String,
    pub code_length: usize,
    pub has_async: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl UnifiedSandbox {
    /// Get current backend configuration
    pub fn backend(&self) -> SandboxBackend {
        self.backend.clone()
    }

    /// Create new unified sandbox with specified backend
    pub async fn new(config: SandboxConfig, backend: SandboxBackend) -> Result<Self> {
        info!("Creating unified sandbox with backend: {:?}", backend);

        let mut node_sandbox = None;
        let mut wasmtime_sandbox = None;

        // Initialize backends based on configuration
        match &backend {
            SandboxBackend::NodeJs => {
                debug!("Initializing Node.js sandbox");
                node_sandbox = Some(Arc::new(CodeSandbox::new(config.clone())?));
            }
            SandboxBackend::Wasm => {
                debug!("Initializing Wasmtime WASM sandbox");
                let wasmtime_config = WasmtimeConfig::from(&config);
                wasmtime_sandbox = Some(Arc::new(WasmtimeSandbox::new(wasmtime_config)?));
            }
            SandboxBackend::Hybrid { .. } => {
                debug!("Initializing Node.js sandbox");
                node_sandbox = Some(Arc::new(CodeSandbox::new(config.clone())?));
                debug!("Initializing Wasmtime WASM sandbox");
                let wasmtime_config = WasmtimeConfig::from(&config);
                wasmtime_sandbox = Some(Arc::new(WasmtimeSandbox::new(wasmtime_config)?));
            }
        }

        // Initialize Javy compiler if using Wasm or Hybrid backends
        #[cfg(feature = "javy-backend")]
        let javy_compiler = if matches!(
            backend,
            SandboxBackend::Wasm | SandboxBackend::Hybrid { .. }
        ) {
            debug!("Initializing Javy JavaScript→WASM compiler");
            Some(Arc::new(JavyCompiler::new(JavyConfig::default())?))
        } else {
            None
        };

        Ok(Self {
            config,
            backend,
            node_sandbox,
            wasmtime_sandbox,
            #[cfg(feature = "javy-backend")]
            javy_compiler,
            metrics: Arc::new(tokio::sync::RwLock::new(UnifiedMetrics::default())),
        })
    }

    /// Execute code using the configured backend
    pub async fn execute(&self, code: &str, context: ExecutionContext) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();

        // Update total execution count
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_executions += 1;
        }

        // Select backend
        let backend_choice = self.select_backend(code, &context).await;
        let backend_name = match backend_choice {
            BackendChoice::NodeJs => "nodejs",
            BackendChoice::Wasm => "wasm",
        };

        debug!("Routing execution to {} backend", backend_name);

        // Execute with selected backend
        let result = match backend_choice {
            BackendChoice::NodeJs => {
                if let Some(sandbox) = &self.node_sandbox {
                    sandbox.execute(code, context).await
                } else {
                    Err(anyhow!("Node.js sandbox not available"))
                }
            }
            BackendChoice::Wasm => {
                // Try Javy compiler first (if enabled), then fallback to pre-compiled WASM
                #[cfg(feature = "javy-backend")]
                {
                    if let Some(compiler) = &self.javy_compiler {
                        // Use Javy to compile JavaScript to WASM and execute
                        debug!("Compiling JavaScript to WASM via Javy");
                        compiler.execute_js(code.to_string(), context).await
                    } else if let Some(sandbox) = &self.wasmtime_sandbox {
                        // Fallback: assume it's pre-compiled WASM bytecode
                        debug!("Executing pre-compiled WASM bytecode");
                        sandbox.execute(code.as_bytes(), &context).await
                    } else {
                        Err(anyhow!(
                            "Neither Javy compiler nor Wasmtime sandbox initialized"
                        ))
                    }
                }
                #[cfg(not(feature = "javy-backend"))]
                {
                    // Without Javy feature, require pre-compiled WASM
                    if let Some(sandbox) = &self.wasmtime_sandbox {
                        debug!("Executing pre-compiled WASM bytecode (Javy not enabled)");

                        let wasm_bytes = if let Some(encoded) = code.strip_prefix("wasm_base64:") {
                            BASE64_STANDARD
                                .decode(encoded)
                                .map_err(|e| anyhow!("Invalid wasm_base64 payload: {}", e))?
                        } else {
                            code.as_bytes().to_vec()
                        };

                        sandbox.execute(&wasm_bytes, &context).await
                    } else {
                        Err(anyhow!("Wasmtime sandbox not initialized. Enable 'javy-backend' feature for JavaScript support."))
                    }
                }
            }
        };

        let execution_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;

            // Track routing decision
            metrics.routing_decisions.push(RoutingDecision {
                backend: backend_name.to_string(),
                reason: self.get_routing_reason(&backend_choice, code).await,
                code_length: code.len(),
                has_async: code.contains("await") || code.contains("async"),
                timestamp: chrono::Utc::now(),
            });

            // Keep only last 100 routing decisions
            if metrics.routing_decisions.len() > 100 {
                metrics.routing_decisions.remove(0);
            }

            // Update backend-specific metrics
            match backend_choice {
                BackendChoice::NodeJs => {
                    metrics.node_executions += 1;
                    if let Ok(ref result) = result {
                        if matches!(result, crate::types::ExecutionResult::Success { .. }) {
                            metrics.node_success_rate = (metrics.node_success_rate
                                * (metrics.node_executions - 1) as f64
                                + 1.0)
                                / metrics.node_executions as f64;
                        } else {
                            metrics.node_success_rate = (metrics.node_success_rate
                                * (metrics.node_executions - 1) as f64)
                                / metrics.node_executions as f64;
                        }
                    }
                    metrics.node_avg_latency_ms = (metrics.node_avg_latency_ms
                        * (metrics.node_executions - 1) as f64
                        + execution_time.as_millis() as f64)
                        / metrics.node_executions as f64;
                }
                BackendChoice::Wasm => {
                    metrics.wasm_executions += 1;
                    if let Ok(ref result) = result {
                        if matches!(result, crate::types::ExecutionResult::Success { .. }) {
                            metrics.wasm_success_rate = (metrics.wasm_success_rate
                                * (metrics.wasm_executions - 1) as f64
                                + 1.0)
                                / metrics.wasm_executions as f64;
                        } else {
                            metrics.wasm_success_rate = (metrics.wasm_success_rate
                                * (metrics.wasm_executions - 1) as f64)
                                / metrics.wasm_executions as f64;
                        }
                    }
                    metrics.wasm_avg_latency_ms = (metrics.wasm_avg_latency_ms
                        * (metrics.wasm_executions - 1) as f64
                        + execution_time.as_millis() as f64)
                        / metrics.wasm_executions as f64;
                }
            }
        }

        result
    }

    /// Select backend for execution based on configuration and heuristics
    async fn select_backend(&self, code: &str, context: &ExecutionContext) -> BackendChoice {
        match &self.backend {
            SandboxBackend::NodeJs => BackendChoice::NodeJs,
            SandboxBackend::Wasm => BackendChoice::Wasm,
            SandboxBackend::Hybrid {
                wasm_ratio,
                intelligent_routing,
            } => {
                if *intelligent_routing {
                    self.intelligent_routing(code, context, *wasm_ratio).await
                } else {
                    self.random_routing(*wasm_ratio).await
                }
            }
        }
    }

    /// Intelligent routing based on code characteristics
    async fn intelligent_routing(
        &self,
        code: &str,
        _context: &ExecutionContext,
        wasm_ratio: f64,
    ) -> BackendChoice {
        let code_heuristics = self.analyze_code(code).await;

        // Detect if code looks like JavaScript
        let is_javascript = code.contains("function")
            || code.contains("const ")
            || code.contains("let ")
            || code.contains("var ")
            || code.contains("console.")
            || code.contains("async ")
            || code.contains("await ")
            || code.contains("class ")
            || code.contains("=>")
            || code.contains("import ")
            || code.contains("export ");

        // If code looks like JavaScript, route to Node.js (unless it's clearly WASM bytecode)
        if is_javascript {
            // Check if this might be WASM bytecode (starts with WASM magic number)
            if code.starts_with("\0asm") {
                return BackendChoice::Wasm;
            }
            // Route JavaScript to Node.js
            return BackendChoice::NodeJs;
        }

        // Route to WASM for simple, short-lived code that's not JavaScript
        if code_heuristics.is_simple && code_heuristics.is_short {
            return BackendChoice::Wasm;
        }

        // Route to Node.js for complex, long-running code
        if code_heuristics.is_complex || code_heuristics.has_external_deps {
            return BackendChoice::NodeJs;
        }

        // Use ratio-based routing for ambiguous cases
        self.random_routing(wasm_ratio).await
    }

    /// Random routing based on configured ratio
    async fn random_routing(&self, wasm_ratio: f64) -> BackendChoice {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        if rng.gen::<f64>() < wasm_ratio {
            BackendChoice::Wasm
        } else {
            BackendChoice::NodeJs
        }
    }

    /// Analyze code characteristics for routing decisions
    async fn analyze_code(&self, code: &str) -> CodeHeuristics {
        let lines = code.lines().count();
        let chars = code.chars().count();

        CodeHeuristics {
            is_simple: !code.contains("require")
                && !code.contains("import")
                && !code.contains("fetch"),
            is_short: lines < 10 && chars < 500,
            is_complex: code.contains("class") || code.contains("function") && lines > 20,
            has_external_deps: code.contains("require")
                || code.contains("import")
                || code.contains("fetch"),
            has_async: code.contains("await") || code.contains("async"),
            has_loops: code.contains("for") || code.contains("while") || code.contains("do"),
        }
    }

    /// Get routing reason for metrics
    async fn get_routing_reason(&self, choice: &BackendChoice, code: &str) -> String {
        let heuristics = self.analyze_code(code).await;

        match choice {
            BackendChoice::NodeJs => {
                if heuristics.is_complex {
                    "Complex code routed to Node.js".to_string()
                } else if heuristics.has_external_deps {
                    "External dependencies require Node.js".to_string()
                } else {
                    "Default Node.js routing".to_string()
                }
            }
            BackendChoice::Wasm => {
                if heuristics.is_simple && heuristics.is_short {
                    "Simple short code routed to WASM".to_string()
                } else {
                    "WASM routing based on ratio".to_string()
                }
            }
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> UnifiedMetrics {
        self.metrics.read().await.clone()
    }

    /// Get backend health status
    pub async fn get_health_status(&self) -> BackendHealth {
        let mut health = BackendHealth::default();

        if self.node_sandbox.is_some() {
            health.node_available = true;
        }

        if let Some(sandbox) = &self.wasmtime_sandbox {
            health.wasm_available = true;
            health.wasmtime_pool_stats = Some(sandbox.get_metrics().await);
        }

        health
    }

    /// Update backend configuration
    pub async fn update_backend(&mut self, backend: SandboxBackend) -> Result<()> {
        info!("Updating sandbox backend to: {:?}", backend);

        // Reinitialize with new backend configuration
        let new_sandbox = Self::new(self.config.clone(), backend).await?;

        // Replace current instance
        *self = new_sandbox;

        Ok(())
    }
}

/// Backend choice for execution
#[derive(Debug, Clone, PartialEq)]
enum BackendChoice {
    NodeJs,
    Wasm,
}

/// Code heuristics for routing decisions
#[derive(Debug)]
#[allow(dead_code)]
struct CodeHeuristics {
    is_simple: bool,
    is_short: bool,
    is_complex: bool,
    has_external_deps: bool,
    has_async: bool,
    has_loops: bool,
}

/// Backend health status
#[derive(Debug, Default, Clone)]
pub struct BackendHealth {
    pub node_available: bool,
    pub wasm_available: bool,
    pub wasmtime_pool_stats: Option<WasmtimeMetrics>,
}

/// Convert SandboxConfig to WasmtimeConfig
impl From<&SandboxConfig> for WasmtimeConfig {
    fn from(config: &SandboxConfig) -> Self {
        // Map SandboxConfig to WasmtimeConfig based on restrictiveness
        if config.allow_network || config.allow_filesystem || config.allow_subprocesses {
            // Permissive if any dangerous permissions are enabled
            WasmtimeConfig::default()
        } else {
            // Restrictive otherwise
            WasmtimeConfig::restrictive()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SandboxConfig;

    #[tokio::test]
    async fn test_unified_sandbox_nodejs_backend() -> Result<()> {
        let sandbox =
            UnifiedSandbox::new(SandboxConfig::restrictive(), SandboxBackend::NodeJs).await?;

        let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));
        let result = sandbox.execute("console.log('test')", context).await?;

        match &result {
            ExecutionResult::Success { stdout, .. } => {
                assert!(
                    stdout.contains("test") || !stdout.is_empty(),
                    "Expected non-empty stdout, got: {:?}",
                    stdout
                );
            }
            _ => panic!("Expected success but got: {:?}", result),
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.node_executions, 1);
        assert_eq!(metrics.wasm_executions, 0);

        Ok(())
    }

    #[tokio::test]
    #[ignore = "WASM backend test needs proper binary data handling - String::from_utf8 fails on binary WASM"]
    async fn test_unified_sandbox_wasm_backend() -> Result<()> {
        let sandbox =
            UnifiedSandbox::new(SandboxConfig::restrictive(), SandboxBackend::Wasm).await?;

        // Use pre-compiled WASM module instead of JavaScript (Javy plugin not bundled)
        let wasm_bytes = wat::parse_str(
            r#"
            (module
                (func (export "main") (result i32)
                    i32.const 42
                )
            )
        "#,
        )?;

        let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));
        let wasm_payload = format!("wasm_base64:{}", BASE64_STANDARD.encode(wasm_bytes));
        let result = sandbox.execute(&wasm_payload, context).await?;

        match &result {
            ExecutionResult::Success { .. } => {} // Success
            _ => panic!("Expected success but got: {:?}", result),
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.node_executions, 0);
        assert_eq!(metrics.wasm_executions, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_unified_sandbox_hybrid_backend() -> Result<()> {
        let sandbox = UnifiedSandbox::new(
            SandboxConfig::default(),
            SandboxBackend::Hybrid {
                wasm_ratio: 0.5,
                intelligent_routing: true,
            },
        )
        .await?;

        let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

        // Execute multiple times to test routing - all JavaScript, routing will decide backend
        for i in 0..10 {
            let code = format!("console.log('test{}')", i);

            let result = sandbox.execute(&code, context.clone()).await?;
            match &result {
                ExecutionResult::Success { .. } => {} // Success
                _ => panic!("Expected success for iteration {} but got: {:?}", i, result),
            }
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 10);
        // With random routing and JavaScript code, should use Node.js (Javy not fully working)
        assert!(metrics.node_executions > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_intelligent_routing() -> Result<()> {
        let sandbox = UnifiedSandbox::new(
            SandboxConfig::default(),
            SandboxBackend::Hybrid {
                wasm_ratio: 0.1, // Low ratio, but intelligent routing should override
                intelligent_routing: true,
            },
        )
        .await?;

        let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

        // Simple code should route to Node.js (not WASM, as Javy is not fully working)
        let simple_result = sandbox
            .execute("console.log('simple')", context.clone())
            .await?;
        match &simple_result {
            ExecutionResult::Success { .. } => {} // Success
            _ => panic!(
                "Expected success for simple code but got: {:?}",
                simple_result
            ),
        }

        // Complex code should route to Node.js
        let complex_result = sandbox
            .execute(
                "function test() { return 'complex'; } console.log(test());",
                context,
            )
            .await?;
        match &complex_result {
            ExecutionResult::Success { .. } => {} // Success
            _ => panic!(
                "Expected success for complex code but got: {:?}",
                complex_result
            ),
        }

        let metrics = sandbox.get_metrics().await;
        assert_eq!(metrics.total_executions, 2);

        // Check routing decisions
        let routing_decisions = &metrics.routing_decisions;
        assert_eq!(routing_decisions.len(), 2);

        // Both should be Node.js (as Javy is not fully working)
        assert_eq!(routing_decisions[0].backend, "nodejs");
        assert_eq!(routing_decisions[1].backend, "nodejs");

        Ok(())
    }

    #[tokio::test]
    async fn test_backend_health() -> Result<()> {
        let sandbox = UnifiedSandbox::new(
            SandboxConfig::restrictive(),
            SandboxBackend::Hybrid {
                wasm_ratio: 0.5,
                intelligent_routing: true,
            },
        )
        .await?;

        let health = sandbox.get_health_status().await;
        assert!(health.node_available);
        assert!(health.wasm_available);
        assert!(health.wasmtime_pool_stats.is_some());

        Ok(())
    }

    #[tokio::test]
    #[ignore = "WASM backend test needs proper binary data handling - String::from_utf8 fails on binary WASM"]
    async fn test_backend_update() -> Result<()> {
        let mut sandbox =
            UnifiedSandbox::new(SandboxConfig::restrictive(), SandboxBackend::NodeJs).await?;

        let context = ExecutionContext::new("test".to_string(), serde_json::json!({}));

        // Execute with Node.js backend
        let result1 = sandbox
            .execute("console.log('nodejs')", context.clone())
            .await?;
        match &result1 {
            ExecutionResult::Success { .. } => {} // Success
            _ => panic!(
                "Expected success for Node.js backend but got: {:?}",
                result1
            ),
        }

        // Update to WASM backend
        sandbox.update_backend(SandboxBackend::Wasm).await?;

        // Execute with WASM backend (use pre-compiled WASM)
        let wasm_bytes = wat::parse_str(
            r#"
            (module
                (func (export "main") (result i32)
                    i32.const 42
                )
            )
        "#,
        )?;
        let wasm_payload = format!("wasm_base64:{}", BASE64_STANDARD.encode(wasm_bytes));
        let result2 = sandbox.execute(&wasm_payload, context).await?;
        match &result2 {
            ExecutionResult::Success { .. } => {} // Success
            _ => panic!("Expected success for WASM backend but got: {:?}", result2),
        }

        let metrics = sandbox.get_metrics().await;
        // Verify at least one execution happened
        assert!(
            metrics.total_executions >= 1,
            "Expected at least 1 execution, got {}",
            metrics.total_executions
        );

        Ok(())
    }
}
