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
///
/// # Errors
///
/// Returns a diagnostic (naming the failed check, never echoing the token) when
/// the token is malformed, unverifiable, expired, or has the wrong
/// issuer/audience.
#[cfg(feature = "oauth")]
pub fn validate_access_token(
    token: &str,
    config: &OAuthConfig,
) -> Result<AuthenticatedPrincipal, String> {
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
        return Err("Server misconfiguration: OAUTH_TOKEN_SECRET is missing".to_string());
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
            Err(err_msg)
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
        Err(msg) => AuthorizationResult::InvalidToken(msg),
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
        Err(description) => {
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
#[allow(unsafe_code, clippy::undocumented_unsafe_blocks)]
mod tests {
    use super::super::types::{AuthorizationResult, RequestAuthorization};
    use do_memory_mcp::protocol::OAuthConfig;
    use do_memory_mcp::server::rate_limiter::OperationType;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tokio::sync::Mutex as AsyncMutex;

    const SECRET: &str = "unit-test-secret";
    const ISSUER: &str = "https://auth.example.com";
    const AUDIENCE: &str = "mcp-server";

    /// Serializes tests that mutate process environment variables.
    static ENV_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

    #[derive(Debug, Serialize, Deserialize)]
    struct TestClaims {
        iss: String,
        aud: String,
        exp: u64,
        sub: String,
        scope: Option<String>,
    }

    fn now() -> u64 {
        jsonwebtoken::get_current_timestamp()
    }

    fn claims(subject: &str, scope: Option<&str>, exp: u64, iss: &str, aud: &str) -> TestClaims {
        TestClaims {
            iss: iss.to_string(),
            aud: aud.to_string(),
            exp,
            sub: subject.to_string(),
            scope: scope.map(str::to_string),
        }
    }

    fn sign(claims: &TestClaims, secret: &str) -> anyhow::Result<String> {
        Ok(encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )?)
    }

    /// Valid, unexpired token signed with the configured secret
    fn valid_token(scope: &str) -> anyhow::Result<String> {
        sign(
            &claims("user-1", Some(scope), now() + 600, ISSUER, AUDIENCE),
            SECRET,
        )
    }

    fn enforced_config() -> OAuthConfig {
        OAuthConfig {
            enabled: true,
            issuer: Some(ISSUER.to_string()),
            audience: Some(AUDIENCE.to_string()),
            token_secret: Some(SECRET.to_string()),
            ..OAuthConfig::default()
        }
    }

    fn unenforceable_config() -> OAuthConfig {
        OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        }
    }

    fn bearer_params(token: &str) -> serde_json::Value {
        json!({"_meta": {"authorization": format!("Bearer {token}")}})
    }

    #[test]
    fn test_validate_access_token_authorized() -> anyhow::Result<()> {
        let token = valid_token("mcp:read mcp:write")?;
        let principal = super::validate_access_token(&token, &enforced_config())
            .map_err(|err| anyhow::anyhow!(err))?;

        assert_eq!(principal.subject, "user-1");
        assert_eq!(
            principal.scopes,
            vec!["mcp:read".to_string(), "mcp:write".to_string()]
        );
        Ok(())
    }

    #[test]
    fn test_validate_access_token_rejects_forged_signature() -> anyhow::Result<()> {
        let token = sign(
            &claims("user-1", Some("mcp:read"), now() + 600, ISSUER, AUDIENCE),
            "different-secret",
        )?;

        let err = super::validate_access_token(&token, &enforced_config())
            .err()
            .ok_or_else(|| anyhow::anyhow!("forged token must be rejected"))?;
        assert!(err.contains("JWT validation failed"), "unexpected: {err}");
        assert!(!err.contains(&token), "diagnostic must not echo the token");
        Ok(())
    }

    #[test]
    fn test_validate_access_token_rejects_wrong_issuer() -> anyhow::Result<()> {
        let token = sign(
            &claims(
                "user-1",
                Some("mcp:read"),
                now() + 600,
                "https://evil.example",
                AUDIENCE,
            ),
            SECRET,
        )?;
        assert!(
            super::validate_access_token(&token, &enforced_config()).is_err(),
            "token with the wrong issuer must be rejected"
        );
        Ok(())
    }

    #[test]
    fn test_validate_access_token_rejects_wrong_audience() -> anyhow::Result<()> {
        let token = sign(
            &claims(
                "user-1",
                Some("mcp:read"),
                now() + 600,
                ISSUER,
                "other-audience",
            ),
            SECRET,
        )?;
        assert!(
            super::validate_access_token(&token, &enforced_config()).is_err(),
            "token with the wrong audience must be rejected"
        );
        Ok(())
    }

    #[test]
    fn test_validate_access_token_rejects_expired() -> anyhow::Result<()> {
        // Expired well beyond the validation leeway
        let token = sign(
            &claims(
                "user-1",
                Some("mcp:read"),
                now().saturating_sub(3600),
                ISSUER,
                AUDIENCE,
            ),
            SECRET,
        )?;
        let err = super::validate_access_token(&token, &enforced_config())
            .err()
            .ok_or_else(|| anyhow::anyhow!("expired token must be rejected"))?;
        assert!(err.contains("JWT validation failed"), "unexpected: {err}");
        Ok(())
    }

    #[test]
    fn test_validate_access_token_missing_secret() -> anyhow::Result<()> {
        let token = valid_token("mcp:read")?;
        let err = super::validate_access_token(&token, &unenforceable_config())
            .err()
            .ok_or_else(|| anyhow::anyhow!("missing secret must reject the token"))?;
        assert!(
            err.contains("OAUTH_TOKEN_SECRET is missing"),
            "unexpected: {err}"
        );
        Ok(())
    }

    #[test]
    fn test_validate_access_token_rejects_malformed() -> anyhow::Result<()> {
        assert!(super::validate_access_token("not-a-jwt", &enforced_config()).is_err());
        assert!(super::validate_access_token("", &enforced_config()).is_err());
        Ok(())
    }

    #[test]
    fn test_validate_bearer_token_missing_secret() {
        let config = unenforceable_config();

        let result = super::validate_bearer_token("some.token.here", &config);
        assert!(
            matches!(&result, AuthorizationResult::InvalidToken(msg) if msg.contains("OAUTH_TOKEN_SECRET is missing")),
            "Expected InvalidToken error about missing secret, got {result:?}"
        );
    }

    #[test]
    fn test_validate_bearer_token_authorized_and_rejected() -> anyhow::Result<()> {
        let config = enforced_config();
        let token = valid_token("mcp:read")?;
        assert!(matches!(
            super::validate_bearer_token(&token, &config),
            AuthorizationResult::Authorized
        ));
        assert!(matches!(
            super::validate_bearer_token("broken", &config),
            AuthorizationResult::InvalidToken(_)
        ));
        Ok(())
    }

    #[test]
    fn test_check_scopes() {
        let read = vec!["mcp:read".to_string()];
        let read_write = vec!["mcp:read".to_string(), "mcp:write".to_string()];

        // No required scopes -> always authorized
        assert!(matches!(
            super::check_scopes(None, &[]),
            AuthorizationResult::Authorized
        ));
        // Missing scope claim -> insufficient
        assert!(matches!(
            super::check_scopes(None, &read),
            AuthorizationResult::InsufficientScope(missing) if missing == read
        ));
        // Granted scope -> authorized
        assert!(matches!(
            super::check_scopes(Some("mcp:read"), &read),
            AuthorizationResult::Authorized
        ));
        // Partially granted -> reports only the missing scopes
        assert!(matches!(
            super::check_scopes(Some("mcp:read"), &read_write),
            AuthorizationResult::InsufficientScope(missing) if missing == vec!["mcp:write".to_string()]
        ));
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
        // Scheme matching is case-insensitive (RFC 7235)
        assert_eq!(
            super::extract_bearer_token("BEARER Mixed.Case"),
            Some("Mixed.Case".to_string())
        );
        assert_eq!(
            super::extract_bearer_token("Authorization:BEARER tight"),
            Some("tight".to_string())
        );
        // A non-bearer scheme never yields a token, even when listed first
        assert_eq!(super::extract_bearer_token("Basic dXNlcjpwYXNz"), None);
        assert_eq!(
            super::extract_bearer_token("Basic dXNlcjpwYXNz, Bearer second"),
            Some("second".to_string())
        );
        assert_eq!(super::extract_bearer_token("Bearer "), None);
        assert_eq!(super::extract_bearer_token(""), None);
    }

    #[test]
    fn test_extract_request_bearer_token_sources() {
        let meta_direct = json!({"_meta": {"authorization": "Bearer meta-direct"}});
        assert_eq!(
            super::extract_request_bearer_token(Some(&meta_direct), None).as_deref(),
            Some("meta-direct")
        );

        let header_lower = json!({"_meta": {"headers": {"authorization": "Bearer header-lower"}}});
        assert_eq!(
            super::extract_request_bearer_token(Some(&header_lower), None).as_deref(),
            Some("header-lower")
        );

        let header_upper = json!({"_meta": {"headers": {"Authorization": "Bearer header-upper"}}});
        assert_eq!(
            super::extract_request_bearer_token(Some(&header_upper), None).as_deref(),
            Some("header-upper")
        );

        let header_blob = json!({"_meta": {"headers": "Authorization: Bearer raw-blob"}});
        assert_eq!(
            super::extract_request_bearer_token(Some(&header_blob), None).as_deref(),
            Some("raw-blob")
        );

        let direct = json!({"authorization": "Bearer direct-field"});
        assert_eq!(
            super::extract_request_bearer_token(Some(&direct), None).as_deref(),
            Some("direct-field")
        );

        // Non-string credentials are ignored and the transport value is used
        let junk = json!({"_meta": {"authorization": 42, "headers": {"authorization": ["x"]}}});
        assert_eq!(
            super::extract_request_bearer_token(Some(&junk), Some("transport-token")).as_deref(),
            Some("transport-token")
        );

        // Transport credentials may be raw or a full header value
        assert_eq!(
            super::extract_request_bearer_token(None, Some("Bearer wrapped")).as_deref(),
            Some("wrapped")
        );
        assert_eq!(super::extract_request_bearer_token(None, Some("   ")), None);
        assert_eq!(super::extract_request_bearer_token(None, None), None);
        assert_eq!(
            super::extract_request_bearer_token(Some(&json!({})), None),
            None
        );
    }

    #[test]
    fn test_authorize_request_disabled_is_unauthenticated() {
        let outcome =
            super::authorize_request(None, OperationType::Write, &OAuthConfig::default(), None);
        assert_eq!(outcome, RequestAuthorization::Unauthenticated);
    }

    #[test]
    fn test_authorize_request_accepts_valid_token_and_checks_scope() -> anyhow::Result<()> {
        let config = enforced_config();
        let read_token = valid_token("mcp:read")?;
        let write_token = valid_token("mcp:write")?;

        let outcome = super::authorize_request(
            Some(&bearer_params(&read_token)),
            OperationType::Read,
            &config,
            None,
        );
        assert!(
            matches!(&outcome, RequestAuthorization::Authenticated(principal)
                if principal.subject == "user-1" && principal.scopes == vec!["mcp:read".to_string()]),
            "read scope must authorize a read operation, got {outcome:?}"
        );

        // A read-scoped token cannot perform a write
        let outcome = super::authorize_request(
            Some(&bearer_params(&read_token)),
            OperationType::Write,
            &config,
            None,
        );
        assert!(
            matches!(&outcome, RequestAuthorization::Rejected { code, error, description }
                if *code == super::INSUFFICIENT_SCOPE_CODE
                    && error == "insufficient_scope"
                    && description.contains("mcp:write")),
            "expected insufficient scope, got {outcome:?}"
        );

        // The write-scoped token can
        let outcome = super::authorize_request(
            Some(&bearer_params(&write_token)),
            OperationType::Write,
            &config,
            None,
        );
        assert!(
            matches!(outcome, RequestAuthorization::Authenticated(_)),
            "write scope must authorize a write operation"
        );
        Ok(())
    }

    #[test]
    fn test_authorize_request_rejects_missing_and_invalid_tokens() -> anyhow::Result<()> {
        let config = enforced_config();

        let missing = super::authorize_request(None, OperationType::Read, &config, None);
        assert!(
            matches!(&missing, RequestAuthorization::Rejected { code, error, description }
                if *code == super::UNAUTHORIZED_CODE
                    && error == "unauthorized"
                    && description.contains("missing bearer token")),
            "expected unauthorized, got {missing:?}"
        );

        let invalid_token = "definitely.not.a.jwt";
        let invalid = super::authorize_request(
            Some(&bearer_params(invalid_token)),
            OperationType::Read,
            &config,
            None,
        );
        assert!(
            matches!(&invalid, RequestAuthorization::Rejected { code, error, description }
                if *code == super::UNAUTHORIZED_CODE
                    && error == "invalid_token"
                    && !description.contains(invalid_token)),
            "expected invalid_token without echoing the token, got {invalid:?}"
        );
        Ok(())
    }

    #[test]
    fn test_authorize_request_uses_transport_credential() -> anyhow::Result<()> {
        let token = valid_token("mcp:read")?;
        let outcome =
            super::authorize_request(None, OperationType::Read, &enforced_config(), Some(&token));
        assert!(matches!(outcome, RequestAuthorization::Authenticated(_)));
        Ok(())
    }

    #[test]
    fn test_authorize_request_missing_secret_fails_closed() {
        let outcome =
            super::authorize_request(None, OperationType::Write, &unenforceable_config(), None);
        assert!(
            matches!(
                &outcome,
                RequestAuthorization::Rejected { error, description, .. }
                    if error == "invalid_config" && description.contains("MCP_OAUTH_TOKEN_SECRET")
            ),
            "Expected fail-closed invalid_config rejection, got {outcome:?}"
        );
    }

    #[test]
    fn test_authorize_request_rejects_empty_transport_credential() {
        let outcome =
            super::authorize_request(None, OperationType::Write, &enforced_config(), Some("   "));
        assert!(
            matches!(&outcome, RequestAuthorization::Rejected { error, .. } if error == "unauthorized"),
            "an empty transport credential must not authorize, got {outcome:?}"
        );
    }

    #[test]
    fn test_required_scope_matches_operation() {
        assert_eq!(
            super::required_scope(OperationType::Read),
            super::SCOPE_READ
        );
        assert_eq!(
            super::required_scope(OperationType::Write),
            super::SCOPE_WRITE
        );
        assert_eq!(super::SCOPE_READ, "mcp:read");
        assert_eq!(super::SCOPE_WRITE, "mcp:write");
    }

    #[test]
    fn test_is_public_method_matches_discovery_only() {
        assert!(super::is_public_method("initialize"));
        assert!(super::is_public_method(
            ".well-known/oauth-protected-resource"
        ));
        assert!(!super::is_public_method("tools/list"));
        assert!(!super::is_public_method("tools/call"));
        assert!(!super::is_public_method(""));
    }

    #[test]
    fn test_validate_oauth_config_modes() {
        assert!(do_memory_mcp::protocol::validate_oauth_config(&OAuthConfig::default()).is_ok());
        assert!(!OAuthConfig::default().is_enforced());

        let misconfigured = unenforceable_config();
        let err = do_memory_mcp::protocol::validate_oauth_config(&misconfigured)
            .expect_err("enabled without a secret must not validate");
        assert!(err.contains("MCP_OAUTH_TOKEN_SECRET"), "unexpected: {err}");
        assert!(!misconfigured.is_enforced());

        let enforced = enforced_config();
        assert!(do_memory_mcp::protocol::validate_oauth_config(&enforced).is_ok());
        assert!(enforced.is_enforced());
        assert!(enforced.enforcement_error().is_none());
    }

    #[test]
    fn test_create_www_authenticate_header() {
        assert_eq!(
            super::create_www_authenticate_header("invalid_token", None, None),
            "Bearer error=\"invalid_token\""
        );
        assert_eq!(
            super::create_www_authenticate_header(
                "insufficient_scope",
                Some("needs mcp:write"),
                Some("memory")
            ),
            "Bearer error=\"insufficient_scope\", error_description=\"needs mcp:write\", realm=\"memory\""
        );
    }

    #[tokio::test]
    async fn test_load_oauth_config_and_transport_token_from_env() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().await;
        let previous_bearer = std::env::var("MCP_OAUTH_BEARER_TOKEN").ok();
        let previous_token = std::env::var("MCP_OAUTH_TOKEN").ok();

        // SAFETY: serialized by ENV_LOCK
        unsafe {
            std::env::set_var("MCP_OAUTH_ENABLED", "true");
            std::env::set_var("MCP_OAUTH_ISSUER", ISSUER);
            std::env::set_var("MCP_OAUTH_AUDIENCE", AUDIENCE);
            std::env::set_var("MCP_OAUTH_SCOPES", "mcp:read, mcp:write ,");
            std::env::set_var("MCP_OAUTH_JWKS_URI", "https://auth.example.com/jwks");
            std::env::set_var("MCP_OAUTH_TOKEN_SECRET", SECRET);
            std::env::set_var("MCP_OAUTH_TOKEN", "  env-token  ");
        }

        let config = super::load_oauth_config();
        let transport = super::load_transport_token();

        // SAFETY: serialized by ENV_LOCK
        unsafe {
            for var in [
                "MCP_OAUTH_ENABLED",
                "MCP_OAUTH_ISSUER",
                "MCP_OAUTH_AUDIENCE",
                "MCP_OAUTH_SCOPES",
                "MCP_OAUTH_JWKS_URI",
                "MCP_OAUTH_TOKEN_SECRET",
                "MCP_OAUTH_TOKEN",
            ] {
                std::env::remove_var(var);
            }
            if let Some(value) = &previous_token {
                std::env::set_var("MCP_OAUTH_TOKEN", value);
            }
        }

        assert!(config.enabled);
        assert!(config.is_enforced());
        assert_eq!(config.issuer.as_deref(), Some(ISSUER));
        assert_eq!(config.audience.as_deref(), Some(AUDIENCE));
        assert_eq!(
            config.scopes,
            vec!["mcp:read".to_string(), "mcp:write".to_string()]
        );
        assert_eq!(
            config.jwks_uri.as_deref(),
            Some("https://auth.example.com/jwks")
        );
        assert_eq!(config.token_secret.as_deref(), Some(SECRET));
        // The transport credential is trimmed
        assert_eq!(transport.as_deref(), Some("env-token"));

        // The bearer-token alias is the fallback when the primary var is unset
        // SAFETY: serialized by ENV_LOCK
        unsafe {
            std::env::remove_var("MCP_OAUTH_TOKEN");
            std::env::set_var("MCP_OAUTH_BEARER_TOKEN", "alias-token");
        }
        let transport = super::load_transport_token();
        // SAFETY: serialized by ENV_LOCK
        unsafe {
            std::env::remove_var("MCP_OAUTH_BEARER_TOKEN");
            if let Some(value) = &previous_bearer {
                std::env::set_var("MCP_OAUTH_BEARER_TOKEN", value);
            }
            if let Some(value) = &previous_token {
                std::env::set_var("MCP_OAUTH_TOKEN", value);
            }
        }
        assert_eq!(transport.as_deref(), Some("alias-token"));
        Ok(())
    }
}
