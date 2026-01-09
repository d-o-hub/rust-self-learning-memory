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
//!           |
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

mod sandbox;
mod tests;
mod types;

pub use sandbox::UnifiedSandbox;
pub use types::{BackendChoice, BackendHealth, SandboxBackend, UnifiedMetrics};
