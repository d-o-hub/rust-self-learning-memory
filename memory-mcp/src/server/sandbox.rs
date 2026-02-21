//! Sandbox backend and validation utilities
//!
//! This module handles sandbox backend selection, WASM availability checks,
//! and Javy plugin validation for secure code execution.

use crate::types::SandboxConfig;
use crate::unified_sandbox::{SandboxBackend, UnifiedSandbox};
use anyhow::Result;
use std::sync::Arc;
#[cfg(feature = "javy-backend")]
use tracing::debug;

/// Determine the appropriate sandbox backend based on environment variables.
///
/// This function checks the `MCP_USE_WASM` and related environment variables
/// to decide whether to use WASM (Javy) or Node.js for code execution.
pub fn determine_sandbox_backend() -> SandboxBackend {
    match std::env::var("MCP_USE_WASM")
        .unwrap_or_else(|_| "auto".to_string())
        .as_str()
    {
        "true" | "wasm" => SandboxBackend::Wasm,
        "false" | "node" => SandboxBackend::NodeJs,
        _ => {
            let ratio = std::env::var("MCP_WASM_RATIO")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.25);
            let intelligent = std::env::var("MCP_INTELLIGENT_ROUTING")
                .map(|v| v.to_lowercase())
                .ok()
                .map(|v| v == "true" || v == "1" || v == "yes")
                .unwrap_or(true);
            SandboxBackend::Hybrid {
                wasm_ratio: ratio.clamp(0.0, 1.0),
                intelligent_routing: intelligent,
            }
        }
    }
}

/// Check if Javy plugin is valid (only when javy-backend feature is enabled)
#[cfg(feature = "javy-backend")]
pub fn is_javy_plugin_valid() -> bool {
    use std::path::Path;

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let plugin_path = Path::new(manifest_dir).join("javy-plugin.wasm");

    if let Ok(metadata) = std::fs::metadata(&plugin_path) {
        if metadata.len() > 100 {
            if let Ok(mut file) = std::fs::File::open(&plugin_path) {
                let mut magic = [0u8; 4];
                if std::io::Read::read_exact(&mut file, &mut magic).is_ok() {
                    if &magic == b"\0asm" {
                        debug!("Valid Javy plugin found ({} bytes)", metadata.len());
                        return true;
                    }
                }
            }
        }
    }

    const EMBEDDED_PLUGIN: &[u8] = include_bytes!("../../javy-plugin.wasm");
    if EMBEDDED_PLUGIN.len() > 100 && EMBEDDED_PLUGIN.starts_with(b"\0asm") {
        debug!(
            "Valid embedded Javy plugin ({} bytes)",
            EMBEDDED_PLUGIN.len()
        );
        return true;
    }

    // Invalid plugin is expected when javy-backend feature is used without proper setup
    // Javy compiler will handle graceful degradation
    debug!(
        "Javy plugin not valid ({} bytes, expected >100 bytes with WASM magic bytes)",
        EMBEDDED_PLUGIN.len()
    );
    false
}

#[cfg(not(feature = "javy-backend"))]
#[allow(dead_code)]
pub fn is_javy_plugin_valid() -> bool {
    false
}

/// Check if WASM sandbox is available for code execution
///
/// This function performs a lightweight check to avoid expensive initialization
/// during server startup. The actual sandbox creation is deferred to first use.
pub fn is_wasm_sandbox_available() -> bool {
    // Priority 1: Check environment variable for explicit override
    if let Ok(val) = std::env::var("MCP_USE_WASM") {
        match val.to_lowercase().as_str() {
            "true" | "wasm" => return true,
            "false" | "node" => return false,
            _ => {}
        }
    }

    // Priority 2: Lightweight check - only verify Javy plugin exists if needed
    // The actual sandbox initialization is deferred to first tool invocation
    #[cfg(feature = "javy-backend")]
    {
        if !is_javy_plugin_valid() {
            debug!("WASM sandbox may be limited due to invalid Javy plugin");
            // Continue - sandbox might still work with pre-compiled WASM
        }
    }

    // Default to available - actual initialization happens lazily
    // This avoids spawning threads with nested async runtimes during startup
    true
}

/// Create and initialize a new sandbox with the given configuration.
///
/// This is a convenience function that combines backend determination
/// and sandbox creation.
pub async fn create_sandbox(config: SandboxConfig) -> Result<Arc<UnifiedSandbox>> {
    let backend = determine_sandbox_backend();
    let sandbox = Arc::new(UnifiedSandbox::new(config, backend).await?);
    Ok(sandbox)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests that set environment variables are removed because they can
    // cause race conditions when run in parallel. The functionality is tested
    // indirectly through the server creation tests in tests.rs.

    #[test]
    #[allow(unsafe_code)]
    fn test_is_wasm_sandbox_available_explicit_false() {
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::set_var("MCP_USE_WASM", "false");
        }
        assert!(!is_wasm_sandbox_available());
        // SAFETY: test-only env var manipulation
        unsafe {
            std::env::remove_var("MCP_USE_WASM");
        }
    }
}
