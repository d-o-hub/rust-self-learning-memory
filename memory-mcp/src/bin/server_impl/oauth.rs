//! OAuth 2.1 security functions for MCP server
//!
//! This module provides OAuth 2.1 authorization support including:
//! - Configuration loading from environment
//! - Bearer token validation (JWT signature verification)
//! - Scope checking
//! - WWW-Authenticate header generation
//!
//! Configuration loading is always available to allow the MCP server to check
//! OAuth configuration regardless of feature flags. When the `oauth` feature
//! is not enabled, the server will log that OAuth is disabled and continue.

use do_memory_mcp::protocol::OAuthConfig;

#[cfg(feature = "oauth")]
use {
    super::types::AuthorizationResult,
    jsonwebtoken::{DecodingKey, Validation, decode},
    serde::{Deserialize, Serialize},
    tracing::{debug, warn},
};

/// Load OAuth configuration from environment variables
///
/// This function is always available to allow the MCP server to check
/// OAuth configuration regardless of feature flags. When the `oauth` feature
/// is not enabled, the server will log that OAuth is disabled and continue.
///
/// Environment variables:
/// - `MCP_OAUTH_ENABLED`: Enable OAuth (true/1/yes)
/// - `MCP_OAUTH_AUDIENCE`: Expected audience claim
/// - `MCP_OAUTH_ISSUER`: Expected issuer claim
/// - `MCP_OAUTH_SCOPES`: Comma-separated list of supported scopes
/// - `MCP_OAUTH_JWKS_URI`: JWKS URI for token validation
/// - `MCP_OAUTH_TOKEN_SECRET`: Secret key for HMAC token validation
pub fn load_oauth_config() -> OAuthConfig {
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
        token_secret: std::env::var("MCP_OAUTH_TOKEN_SECRET").ok(),
    }
}

/// JWT Claims structure
#[cfg(feature = "oauth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    sub: String,
    scope: Option<String>,
}

/// Validate Bearer token (JWT signature verification)
///
/// This performs secure JWT validation including:
/// - Signature verification (using configured secret)
/// - Format validation
/// - Issuer validation (if configured)
/// - Audience validation (if configured)
/// - Expiration check
/// - Subject claim presence
///
/// Note: When `MCP_OAUTH_TOKEN_SECRET` is not provided, signature verification
/// fails and an error is returned. This ensures tokens are always verified.
#[cfg(feature = "oauth")]
pub fn validate_bearer_token(token: &str, config: &OAuthConfig) -> AuthorizationResult {
    let mut validation = Validation::default();

    // Configure expected issuer if present
    if let Some(expected_iss) = &config.issuer {
        validation.set_issuer(&[expected_iss]);
    }

    // Configure expected audience if present
    if let Some(expected_aud) = &config.audience {
        validation.set_audience(&[expected_aud]);
    }

    // Signature verification
    let decoding_key = if let Some(secret) = &config.token_secret {
        DecodingKey::from_secret(secret.as_bytes())
    } else {
        warn!("SECURITY ERROR: No OAUTH_TOKEN_SECRET configured. Rejecting token.");
        return AuthorizationResult::InvalidToken(
            "Server misconfiguration: OAUTH_TOKEN_SECRET is missing".to_string(),
        );
    };

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            debug!("Token validated for subject: {}", token_data.claims.sub);
            AuthorizationResult::Authorized
        }
        Err(e) => {
            let err_msg = format!("JWT validation failed: {e}");
            warn!("{}", err_msg);
            AuthorizationResult::InvalidToken(err_msg)
        }
    }
}

/// Check if token has required scopes
///
/// Validates that the token contains all required scopes for the requested operation.
/// Scopes in the token are expected to be space-separated as per RFC 6749.
#[cfg(feature = "oauth")]
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
#[cfg(feature = "oauth")]
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
#[cfg(feature = "oauth")]
pub fn create_www_authenticate_header(
    error: &str,
    error_description: Option<&str>,
    realm: Option<&str>,
) -> String {
    let mut parts = vec![format!("error=\"{}\"", error)];

    if let Some(desc) = error_description {
        parts.push(format!("error_description=\"{desc}\""));
    }

    if let Some(r) = realm {
        parts.push(format!("realm=\"{r}\""));
    }

    format!("Bearer {}", parts.join(", "))
}

#[cfg(all(test, feature = "oauth"))]
mod tests {
    use super::super::types::AuthorizationResult;
    use do_memory_mcp::protocol::OAuthConfig;

    #[test]
    fn test_validate_bearer_token_missing_secret() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        };

        let result = super::validate_bearer_token("some.token.here", &config);
        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("OAUTH_TOKEN_SECRET is missing"));
            }
            _ => unreachable!("Expected InvalidToken error, got {result:?}"),
        }
    }
}
