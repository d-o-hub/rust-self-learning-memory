//! OAuth 2.1 security functions for MCP server
//!
//! This module provides OAuth 2.1 authorization support including:
//! - Configuration loading from environment
//! - Bearer token validation (simplified JWT parsing)
//! - Scope checking
//! - WWW-Authenticate header generation
//!
//! All functions in this module are gated behind the `oauth` feature flag
//! and reserved for future OAuth 2.1 implementation.

#![cfg(feature = "oauth")]

use super::types::AuthorizationResult;
use tracing::debug;

/// Load OAuth configuration from environment variables
///
/// Environment variables:
/// - `MCP_OAUTH_ENABLED`: Enable OAuth (true/1/yes)
/// - `MCP_OAUTH_AUDIENCE`: Expected audience claim
/// - `MCP_OAUTH_ISSUER`: Expected issuer claim
/// - `MCP_OAUTH_SCOPES`: Comma-separated list of supported scopes
/// - `MCP_OAUTH_JWKS_URI`: JWKS URI for token validation
pub fn load_oauth_config() -> crate::protocol::OAuthConfig {
    let enabled = std::env::var("MCP_OAUTH_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase();

    OAuthConfig {
        enabled: enabled == "true" || enabled == "1" || enabled == "yes",
        audience: std::env::var("MCP_OAUTH_AUDIENCE").ok(),
        issuer: std::env::var("MCP_OAUTH_ISSUER").ok(),
        scopes: std::env::var("MCP_OAUTH_SCOPES")
            .unwrap_or_else(|_| "mcp:read,mcp:write".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        jwks_uri: std::env::var("MCP_OAUTH_JWKS_URI").ok(),
    }
}

/// ⚠️ SECURITY WARNING: This is simplified JWT validation for stdio mode only.
/// It does NOT verify signatures. For production HTTP mode, use a proper JWT library.
///
/// Validate Bearer token (simplified JWT parsing)
///
/// This performs basic JWT validation including:
/// - Format validation (3 parts separated by dots)
/// - Base64url decoding of payload
/// - JSON parsing of claims
/// - Issuer validation (if configured)
/// - Audience validation (if configured)
/// - Expiration check (if present)
/// - Subject claim presence
///
/// Note: This is a simplified implementation. Production systems should use
/// a proper JWT library with signature verification against JWKS.
#[allow(dead_code)]
pub fn validate_bearer_token(token: &str, config: &OAuthConfig) -> AuthorizationResult {
    // Split JWT into parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return AuthorizationResult::InvalidToken("Invalid token format".to_string());
    }

    // Decode payload (base64url)
    let payload = match base64url_decode(parts[1]) {
        Ok(p) => p,
        Err(e) => {
            return AuthorizationResult::InvalidToken(format!("Invalid token payload: {}", e))
        }
    };

    // Parse JSON payload - convert bytes to string first
    let payload_str = match String::from_utf8(payload) {
        Ok(s) => s,
        Err(e) => {
            return AuthorizationResult::InvalidToken(format!("Invalid token encoding: {}", e))
        }
    };

    let claims: serde_json::Value = match serde_json::from_str(&payload_str) {
        Ok(c) => c,
        Err(e) => return AuthorizationResult::InvalidToken(format!("Invalid token JSON: {}", e)),
    };

    // Validate issuer if configured
    if let Some(expected_iss) = &config.issuer {
        let token_iss = claims.get("iss").and_then(|v| v.as_str()).unwrap_or("");
        if !token_iss.is_empty() && token_iss != expected_iss {
            return AuthorizationResult::InvalidToken(format!(
                "Invalid token issuer: expected {}, got {}",
                expected_iss, token_iss
            ));
        }
    }

    // Validate audience if configured
    if let Some(expected_aud) = &config.audience {
        let token_aud = claims.get("aud").and_then(|v| v.as_str()).unwrap_or("");
        if !token_aud.is_empty() && token_aud != expected_aud {
            return AuthorizationResult::InvalidToken(format!(
                "Invalid token audience: expected {}, got {}",
                expected_aud, token_aud
            ));
        }
    }

    // Check expiration if present
    if let Some(exp) = claims.get("exp").and_then(|v| v.as_u64()) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if exp < now {
            return AuthorizationResult::InvalidToken("Token expired".to_string());
        }
    }

    // Validate required subject claim
    let sub = claims.get("sub").and_then(|v| v.as_str()).unwrap_or("");
    if sub.is_empty() {
        return AuthorizationResult::InvalidToken("Token missing subject claim".to_string());
    }

    debug!("Token validated for subject: {}", sub);
    AuthorizationResult::Authorized
}

/// Base64url decode (RFC 4648)
///
/// Decodes base64url-encoded data. Base64url is a URL-safe variant of base64
/// that uses `-` and `_` instead of `+` and `/`, and omits padding.
#[allow(dead_code)]
pub fn base64url_decode(input: &str) -> Result<Vec<u8>, base64::DecodeError> {
    // For simplicity, we'll do basic base64 decoding
    // In production, use a proper base64url crate
    let filtered: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    // Pad if necessary
    let padded = match filtered.len() % 4 {
        2 => filtered + "==",
        3 => filtered + "=",
        _ => filtered,
    };

    base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &padded)
}

/// Check if token has required scopes
///
/// Validates that the token contains all required scopes for the requested operation.
/// Scopes in the token are expected to be space-separated as per RFC 6749.
#[allow(dead_code)]
pub fn check_scopes(token_scope: Option<&str>, required_scopes: &[String]) -> AuthorizationResult {
    let token_scopes: Vec<String> = match token_scope {
        Some(s) => s
            .split(' ')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => vec![],
    };

    // If no required scopes, allow access
    if required_scopes.is_empty() {
        return AuthorizationResult::Authorized;
    }

    // If token has no scopes and required scopes exist, deny
    if token_scopes.is_empty() {
        return AuthorizationResult::InsufficientScope(required_scopes.to_vec());
    }

    // Check if token has all required scopes
    let missing: Vec<String> = required_scopes
        .iter()
        .filter(|r| !token_scopes.contains(r))
        .cloned()
        .collect();

    if missing.is_empty() {
        AuthorizationResult::Authorized
    } else {
        AuthorizationResult::InsufficientScope(missing)
    }
}

/// Extract Bearer token from Authorization header
///
/// Note: For stdio mode, we can't access HTTP headers directly.
/// This function is provided for future HTTP transport mode support.
#[allow(dead_code)]
pub fn extract_bearer_token(_headers: &str) -> Option<String> {
    // For stdio mode, we can't access headers directly
    // This would be used for HTTP transport mode
    None
}

/// Create WWW-Authenticate challenge header value (RFC 6750)
///
/// Generates a WWW-Authenticate header value for OAuth 2.1 Bearer token authentication.
/// Used when returning 401 Unauthorized responses to inform clients how to authenticate.
///
/// # Arguments
/// * `error` - OAuth error code (e.g., "invalid_token", "insufficient_scope")
/// * `error_description` - Human-readable error description
/// * `realm` - Optional realm value
#[allow(dead_code)]
pub fn create_www_authenticate_header(
    error: &str,
    error_description: Option<&str>,
    realm: Option<&str>,
) -> String {
    let mut parts = vec![format!("error=\"{}\"", error)];

    if let Some(desc) = error_description {
        parts.push(format!("error_description=\"{}\"", desc));
    }

    if let Some(r) = realm {
        parts.push(format!("realm=\"{}\"", r));
    }

    format!("Bearer {}", parts.join(", "))
}
