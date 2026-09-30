//! OAuth 2.1 security functions for MCP server
//!
//! This module provides OAuth 2.1 authorization support including:
//! - Configuration loading from environment
//! - Bearer token validation (JWT signature verification)
//! - Scope checking
//! - WWW-Authenticate header generation
//! - Fail-closed request authorization (issue #1082)
//!
//! Configuration loading is always available to allow the MCP server to check
//! OAuth configuration regardless of feature flags.
//!
//! # Default stdio behavior
//!
//! The server transport is JSON-RPC over stdio, which cannot carry HTTP headers.
//! Therefore:
//!
//! - **OAuth disabled (default).** `MCP_OAUTH_ENABLED` is unset, so no
//!   authorization capability is advertised, no credentials are read and every
//!   request is rate limited as a single process-scoped principal.
//! - **OAuth enabled.** `MCP_OAUTH_ENABLED=true` requires `MCP_OAUTH_TOKEN_SECRET`
//!   (and the `oauth` feature compiled in). The secret is validated at startup
//!   and the server refuses to start otherwise, because advertising authorization
//!   that cannot be enforced is a security misrepresentation.
//!   Credentials are read, in order, from the request (`_meta.authorization`,
//!   `_meta.headers.authorization`, or a top-level `authorization` field) or, as
//!   the transport default for stdio, from the `MCP_OAUTH_TOKEN` /
//!   `MCP_OAUTH_BEARER_TOKEN` environment variables of the server process.
//!   `initialize` and `.well-known/oauth-protected-resource` remain callable
//!   without a token so clients can discover the authorization server; every
//!   other method requires a valid, unexpired token with the required scope.

use super::types::RequestAuthorization;
use do_memory_mcp::protocol::OAuthConfig;
use do_memory_mcp::server::rate_limiter::OperationType;
use serde_json::Value;

#[cfg(feature = "oauth")]
use {
    super::types::{AuthenticatedPrincipal, AuthorizationResult},
    jsonwebtoken::{DecodingKey, Validation, decode},
    serde::{Deserialize, Serialize},
    tracing::{debug, warn},
};

/// Scope required for write operations
#[cfg(feature = "oauth")]
pub const SCOPE_WRITE: &str = "mcp:write";
/// Scope required for read operations
#[cfg(feature = "oauth")]
pub const SCOPE_READ: &str = "mcp:read";
/// JSON-RPC error code for missing/invalid credentials
pub const UNAUTHORIZED_CODE: i32 = -32001;
/// JSON-RPC error code for valid credentials with insufficient scope
#[cfg(feature = "oauth")]
pub const INSUFFICIENT_SCOPE_CODE: i32 = -32003;

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

/// Load the transport-supplied bearer token for a stdio server process
///
/// stdio clients cannot send HTTP headers, so the credential is taken from the
/// server process environment instead.
pub fn load_transport_token() -> Option<String> {
    for var in ["MCP_OAUTH_TOKEN", "MCP_OAUTH_BEARER_TOKEN"] {
        if let Ok(value) = std::env::var(var) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
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

/// Parse a space-separated RFC 6749 scope claim
#[cfg(feature = "oauth")]
fn parse_scopes(scope: Option<&str>) -> Vec<String> {
    scope
        .map(|s| {
            s.split(' ')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Validate a Bearer token and return the authenticated principal
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
pub fn validate_access_token(
    token: &str,
    config: &OAuthConfig,
) -> Result<AuthenticatedPrincipal, AuthorizationResult> {
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
        return Err(AuthorizationResult::InvalidToken(
            "Server misconfiguration: OAUTH_TOKEN_SECRET is missing".to_string(),
        ));
    };

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            debug!("Token validated for subject: {}", token_data.claims.sub);
            Ok(AuthenticatedPrincipal {
                subject: token_data.claims.sub,
                scopes: parse_scopes(token_data.claims.scope.as_deref()),
            })
        }
        Err(e) => {
            // jsonwebtoken errors name the failure (e.g. ExpiredSignature) and
            // never echo the token itself.
            let err_msg = format!("JWT validation failed: {e}");
            warn!("{}", err_msg);
            Err(AuthorizationResult::InvalidToken(err_msg))
        }
    }
}

/// Validate Bearer token (JWT signature verification)
///
/// Thin wrapper over [`validate_access_token`] for callers that only need a
/// pass/fail answer.
#[cfg(feature = "oauth")]
pub fn validate_bearer_token(token: &str, config: &OAuthConfig) -> AuthorizationResult {
    match validate_access_token(token, config) {
        Ok(_) => AuthorizationResult::Authorized,
        Err(result) => result,
    }
}

/// Check if token has required scopes
///
/// Validates that the token contains all required scopes for the requested operation.
/// Scopes in the token are expected to be space-separated as per RFC 6749.
#[cfg(feature = "oauth")]
pub fn check_scopes(token_scope: Option<&str>, required_scopes: &[String]) -> AuthorizationResult {
    let token_scopes = parse_scopes(token_scope);

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

/// Extract a Bearer token from an `Authorization` header value
///
/// Accepts a raw header value (`Bearer <token>`), a prefixed header
/// (`Authorization: Bearer <token>`, case-insensitive) and multiple
/// comma/newline separated header values. Returns `None` for other schemes or
/// empty tokens. The scheme is matched case-insensitively as required by
/// RFC 7235.
pub fn extract_bearer_token(headers: &str) -> Option<String> {
    for segment in headers.split(['\n', ',']) {
        let segment = segment.trim();
        let value = match segment.split_once(':') {
            Some((name, rest)) if name.trim().eq_ignore_ascii_case("authorization") => rest.trim(),
            _ => segment,
        };

        if let Some((scheme, rest)) = value.split_once(' ') {
            if scheme.eq_ignore_ascii_case("bearer") {
                let token = rest.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

/// Extract the request's bearer token from MCP parameters or the transport
///
/// Request-provided credentials are acceptable because they are *verified*
/// before use; what matters is that no unverified identity drives authorization.
pub fn extract_request_bearer_token(
    params: Option<&Value>,
    transport_token: Option<&str>,
) -> Option<String> {
    if let Some(params) = params {
        let meta = params.get("_meta");
        let headers = meta.and_then(|m| m.get("headers"));
        let candidates = [
            meta.and_then(|m| m.get("authorization")),
            headers.and_then(|h| h.get("authorization")),
            headers.and_then(|h| h.get("Authorization")),
            params.get("authorization"),
        ];

        for candidate in candidates.into_iter().flatten() {
            if let Value::String(raw) = candidate {
                if let Some(token) = extract_bearer_token(raw) {
                    return Some(token);
                }
            }
        }

        // Headers supplied as a raw string blob
        if let Some(Value::String(raw)) = headers {
            if let Some(token) = extract_bearer_token(raw) {
                return Some(token);
            }
        }
    }

    let transport_token = transport_token?;
    extract_bearer_token(transport_token).or_else(|| {
        let raw = transport_token.trim();
        (!raw.is_empty()).then(|| raw.to_string())
    })
}

/// Required scope for a rate-limited operation type
#[cfg(feature = "oauth")]
pub fn required_scope(operation: OperationType) -> &'static str {
    match operation {
        OperationType::Read => SCOPE_READ,
        OperationType::Write => SCOPE_WRITE,
    }
}

/// Methods that stay callable without credentials when OAuth is enforced
///
/// Discovery methods reveal no memory data and are required before a client can
/// obtain a token.
pub fn is_public_method(method: &str) -> bool {
    matches!(
        method,
        "initialize" | ".well-known/oauth-protected-resource"
    )
}

/// Verify a token and derive the trusted principal (issue #1082)
#[cfg(feature = "oauth")]
fn verify(token: &str, operation: OperationType, config: &OAuthConfig) -> RequestAuthorization {
    let principal = match validate_access_token(token, config) {
        Ok(principal) => principal,
        Err(result) => {
            let description = match result {
                AuthorizationResult::InvalidToken(msg) => msg,
                AuthorizationResult::MissingToken => "Missing bearer token".to_string(),
                AuthorizationResult::InsufficientScope(missing) => {
                    format!("insufficient scope: {}", missing.join(", "))
                }
                AuthorizationResult::Authorized => "invalid token".to_string(),
            };
            return RequestAuthorization::Rejected {
                code: UNAUTHORIZED_CODE,
                error: "invalid_token".to_string(),
                description,
            };
        }
    };

    let required = required_scope(operation);
    if !principal.scopes.iter().any(|scope| scope == required) {
        return RequestAuthorization::Rejected {
            code: INSUFFICIENT_SCOPE_CODE,
            error: "insufficient_scope".to_string(),
            description: format!("token is missing the required scope '{required}'"),
        };
    }

    RequestAuthorization::Authenticated(principal)
}

/// Fail-closed stand-in used when the `oauth` feature is not compiled in
///
/// This branch is only reachable for an enabled-but-unenforceable configuration,
/// which [`authorize_request`] rejects before calling it.
#[cfg(not(feature = "oauth"))]
fn verify(_token: &str, _operation: OperationType, config: &OAuthConfig) -> RequestAuthorization {
    RequestAuthorization::Rejected {
        code: UNAUTHORIZED_CODE,
        error: "invalid_config".to_string(),
        description: do_memory_mcp::protocol::validate_oauth_config(config)
            .err()
            .unwrap_or("token verification is unavailable")
            .to_string(),
    }
}

/// Authorize a request before method dispatch (issue #1082)
///
/// - OAuth disabled: request is unauthenticated (process-scoped principal).
/// - OAuth enabled but unenforceable: rejected, never silently bypassed.
/// - OAuth enabled: a valid signature/issuer/audience/expiry/scope token is
///   required, read from the request or the stdio transport environment.
///
/// Rejection descriptions never contain credential material.
pub fn authorize_request(
    params: Option<&Value>,
    operation: OperationType,
    config: &OAuthConfig,
    transport_token: Option<&str>,
) -> RequestAuthorization {
    if !config.enabled {
        return RequestAuthorization::Unauthenticated;
    }

    if let Some(reason) = config.enforcement_error() {
        return RequestAuthorization::Rejected {
            code: UNAUTHORIZED_CODE,
            error: "invalid_config".to_string(),
            description: format!("authorization is enabled but cannot be enforced: {reason}"),
        };
    }

    let Some(token) = extract_request_bearer_token(params, transport_token) else {
        return RequestAuthorization::Rejected {
            code: UNAUTHORIZED_CODE,
            error: "unauthorized".to_string(),
            description: "missing bearer token for an authenticated operation".to_string(),
        };
    };

    verify(&token, operation, config)
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
    let mut parts = vec![format!("error=\"{error}\"")];

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
    use super::super::types::{AuthorizationResult, RequestAuthorization};
    use do_memory_mcp::protocol::OAuthConfig;

    #[test]
    fn test_validate_bearer_token_missing_secret() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        };

        let result = super::validate_bearer_token("some.token.here", &config);
        assert!(
            matches!(&result, AuthorizationResult::InvalidToken(msg) if msg.contains("OAUTH_TOKEN_SECRET is missing")),
            "Expected InvalidToken error about missing secret, got {result:?}"
        );
    }

    #[test]
    fn test_extract_bearer_token_variants() {
        assert_eq!(
            super::extract_bearer_token("Bearer abc.def.ghi"),
            Some("abc.def.ghi".to_string())
        );
        assert_eq!(
            super::extract_bearer_token("authorization: bearer abc"),
            Some("abc".to_string())
        );
        assert_eq!(
            super::extract_bearer_token("X-Trace: 1\nAuthorization: Bearer tok\n"),
            Some("tok".to_string())
        );
        assert_eq!(super::extract_bearer_token("Basic dXNlcjpwYXNz"), None);
        assert_eq!(super::extract_bearer_token("Bearer "), None);
        assert_eq!(super::extract_bearer_token(""), None);
    }

    #[test]
    fn test_authorize_request_missing_secret_fails_closed() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        };

        let outcome = super::authorize_request(
            None,
            do_memory_mcp::server::rate_limiter::OperationType::Write,
            &config,
            None,
        );
        assert!(
            matches!(
                &outcome,
                RequestAuthorization::Rejected { error, description, .. }
                    if error == "invalid_config" && description.contains("MCP_OAUTH_TOKEN_SECRET")
            ),
            "Expected fail-closed invalid_config rejection, got {outcome:?}"
        );
    }
}
