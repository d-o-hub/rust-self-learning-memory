//! OAuth 2.1 security functions for MCP server
//!
//! This module provides OAuth 2.1 authorization support including:
//! - Configuration loading from environment
//! - Bearer token validation (JWT HMAC signature verification)
//! - Scope checking
//! - WWW-Authenticate header generation
//!
//! Configuration loading is always available to allow the MCP server to check
//! OAuth configuration regardless of feature flags. When the `oauth` feature
//! is not enabled, the server will log that OAuth is disabled and continue.

// Import types needed for OAuth functionality
#[cfg(feature = "oauth")]
pub use super::types::AuthorizationResult;

// Import OAuthConfig from the library's protocol module
use do_memory_mcp::protocol::OAuthConfig;

#[cfg(feature = "oauth")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "oauth")]
use tracing::debug;

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
/// - `MCP_OAUTH_TOKEN_SECRET`: HMAC secret for token validation
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

/// JWT Claims structure for validation
#[cfg(feature = "oauth")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Issuer
    pub iss: Option<String>,
    /// Subject (required, non-empty)
    pub sub: String,
    /// Audience (can be string or array of strings)
    #[serde(deserialize_with = "deserialize_audience")]
    pub aud: Option<Vec<String>>,
    /// Expiration time
    pub exp: u64,
    /// Issued at
    pub iat: Option<u64>,
    /// OAuth scopes (space-separated string)
    pub scope: Option<String>,
}

/// Custom deserializer for audience claim which can be a string or an array
#[cfg(feature = "oauth")]
fn deserialize_audience<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(Some(vec![s])),
        serde_json::Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    result.push(s.to_string());
                } else {
                    return Err(D::Error::custom("Audience array must contain only strings"));
                }
            }
            Ok(Some(result))
        }
        serde_json::Value::Null => Ok(None),
        _ => Err(D::Error::custom("Audience must be a string or an array")),
    }
}

/// Validate Bearer token (JWT with HMAC signature verification)
///
/// This performs JWT validation including:
/// - HMAC signature verification (HS256/HS384/HS512)
/// - Issuer validation (if configured)
/// - Audience validation (if configured)
/// - Expiration check
/// - Subject claim presence (must be non-empty)
///
/// # Security
/// This function fails closed if `token_secret` is missing or empty when OAuth is enabled.
/// It never falls back to insecure decoding.
#[cfg(feature = "oauth")]
pub fn validate_bearer_token(token: &str, config: &OAuthConfig) -> AuthorizationResult {
    use jsonwebtoken::{DecodingKey, Validation, decode};

    // Fail closed if token_secret is missing or empty
    let secret = match &config.token_secret {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            return AuthorizationResult::InvalidToken(
                "OAuth configuration error: MCP_OAUTH_TOKEN_SECRET is not set or empty".to_string(),
            );
        }
    };

    // Configure validation
    let mut validation = Validation::default();

    // Set expected issuer
    if let Some(issuer) = &config.issuer {
        validation.set_issuer(&[issuer]);
    }

    // Set expected audience
    if let Some(audience) = &config.audience {
        validation.set_audience(&[audience]);
    } else {
        // Disable audience validation if not configured
        validation.validate_aud = false;
    }

    // Decode and verify token
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(c) => c,
        Err(e) => {
            return AuthorizationResult::InvalidToken(format!("JWT validation failed: {}", e));
        }
    };

    let claims = token_data.claims;

    // Validate required subject claim is non-empty
    if claims.sub.trim().is_empty() {
        return AuthorizationResult::InvalidToken(
            "Token missing or empty subject claim".to_string(),
        );
    }

    debug!("Token validated for subject: {}", claims.sub);
    AuthorizationResult::Authorized
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
        parts.push(format!("error_description=\"{}\"", desc));
    }

    if let Some(r) = realm {
        parts.push(format!("realm=\"{}\"", r));
    }

    format!("Bearer {}", parts.join(", "))
}

#[cfg(test)]
mod oauth_security_tests;
