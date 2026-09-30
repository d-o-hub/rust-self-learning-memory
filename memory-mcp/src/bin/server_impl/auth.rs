//! Request authorization gate and trusted rate-limit identity
//!
//! Two defects are handled here:
//!
//! - **#1082** — OAuth used to be logged but never enforced. Every request now
//!   passes [`AuthContext::resolve_identity`] before method dispatch; when
//!   authorization is enabled, protected methods require a verified token or the
//!   request is rejected (fail-closed, including for an unenforceable
//!   configuration).
//! - **#1084** — the rate-limit bucket used to be selected from caller-supplied
//!   request fields, so rotating an ID escaped a saturated bucket. The bucket is
//!   now derived from the validated token subject, or from a process-scoped
//!   identity when the stdio transport carries no credentials.

use super::oauth::{UNAUTHORIZED_CODE, authorize_request, is_public_method, load_transport_token};
use super::types::RequestAuthorization;
use do_memory_mcp::jsonrpc::{JsonRpcError, JsonRpcResponse};
use do_memory_mcp::protocol::OAuthConfig;
use do_memory_mcp::server::rate_limiter::{ClientId, OperationType};
use serde_json::{Value, json};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::warn;

/// Reason a request was rejected before dispatch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// JSON-RPC error code
    pub code: i32,
    /// Stable machine-readable error name
    pub error: String,
    /// Human-readable diagnostic (never contains credential material)
    pub description: String,
}

/// Authentication policy plus the credential the transport provided
///
/// Holds the single identity used when no validated principal exists, together
/// with the credential supplied at startup (for stdio, an environment variable).
/// Injecting it keeps tests deterministic and avoids process-wide environment
/// mutation.
#[derive(Debug, Clone)]
pub struct AuthContext {
    config: OAuthConfig,
    process_identity: ClientId,
    transport_token: Option<String>,
}

impl AuthContext {
    /// Create a context from an explicit configuration, identity and credential
    pub fn new(
        config: OAuthConfig,
        process_identity: ClientId,
        transport_token: Option<String>,
    ) -> Self {
        Self {
            config,
            process_identity,
            transport_token,
        }
    }

    /// Create a context from the server process environment
    ///
    /// The identity is process-scoped and the credential comes from
    /// `MCP_OAUTH_TOKEN`/`MCP_OAUTH_BEARER_TOKEN`, which is how a stdio client
    /// supplies a bearer token.
    pub fn from_env(config: OAuthConfig) -> Self {
        Self::new(config, ClientId::process(), load_transport_token())
    }

    /// Authorization configuration backing this context
    pub fn config(&self) -> &OAuthConfig {
        &self.config
    }

    /// Whether authorization is enabled *and* enforceable
    pub fn is_enforced(&self) -> bool {
        self.config.is_enforced()
    }

    /// Authorize a request and derive the trusted rate-limit identity
    ///
    /// Returns the identity that owns the rate-limit bucket, or the rejection to
    /// return to the caller. Discovery methods (`initialize`,
    /// `.well-known/oauth-protected-resource`) stay reachable without a token
    /// when authorization is enforced, so a client can learn how to authenticate.
    pub fn resolve_identity(
        &self,
        method: &str,
        operation: OperationType,
        params: Option<&Value>,
    ) -> Result<ClientId, Rejection> {
        if self.is_enforced() && is_public_method(method) {
            return Ok(self.process_identity.clone());
        }

        match authorize_request(
            params,
            operation,
            &self.config,
            self.transport_token.as_deref(),
        ) {
            RequestAuthorization::Unauthenticated => Ok(self.process_identity.clone()),
            RequestAuthorization::Authenticated(principal) => {
                Ok(subject_identity(&principal.subject))
            }
            RequestAuthorization::Rejected {
                code,
                error,
                description,
            } => Err(Rejection {
                code,
                error,
                description,
            }),
        }
    }
}

/// Opaque, fixed-width identity for a validated token subject
///
/// Hashing keeps bucket keys bounded in size and avoids persisting raw subject
/// identifiers in memory or logs.
pub fn subject_identity(subject: &str) -> ClientId {
    let mut hasher = DefaultHasher::new();
    subject.hash(&mut hasher);
    ClientId::Id(format!("subject:{:016x}", hasher.finish()))
}

/// Build a stable error response for a rejected request
///
/// The response carries a diagnostic but never the token or secret that failed
/// verification.
pub fn rejection_response(id: Option<Value>, rejection: &Rejection) -> JsonRpcResponse {
    warn!(
        "Request rejected before dispatch: {} ({})",
        rejection.error, rejection.description
    );
    let message = if rejection.code == UNAUTHORIZED_CODE {
        "Unauthorized"
    } else {
        "Insufficient scope"
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code: rejection.code,
            message: message.to_string(),
            data: Some(json!({
                "error": rejection.error,
                "description": rejection.description,
            })),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "oauth")]
    const SECRET: &str = "auth-context-test-secret";
    #[cfg(feature = "oauth")]
    const ISSUER: &str = "https://auth.example.com";
    #[cfg(feature = "oauth")]
    const AUDIENCE: &str = "mcp-server";

    fn disabled_config() -> OAuthConfig {
        OAuthConfig::default()
    }

    fn unenforceable_config() -> OAuthConfig {
        OAuthConfig {
            enabled: true,
            token_secret: None,
            ..OAuthConfig::default()
        }
    }

    #[cfg(feature = "oauth")]
    fn enforced_config() -> OAuthConfig {
        OAuthConfig {
            enabled: true,
            issuer: Some(ISSUER.to_string()),
            audience: Some(AUDIENCE.to_string()),
            token_secret: Some(SECRET.to_string()),
            ..OAuthConfig::default()
        }
    }

    #[cfg(feature = "oauth")]
    fn signed_token(scope: &str, subject: &str) -> anyhow::Result<String> {
        use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
        use serde::Serialize;

        #[derive(Debug, Serialize)]
        struct Claims {
            iss: String,
            aud: String,
            exp: u64,
            sub: String,
            scope: String,
        }

        Ok(encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                iss: ISSUER.to_string(),
                aud: AUDIENCE.to_string(),
                exp: jsonwebtoken::get_current_timestamp() + 600,
                sub: subject.to_string(),
                scope: scope.to_string(),
            },
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )?)
    }

    #[test]
    fn test_subject_identity_is_stable_and_opaque() {
        let alice = subject_identity("alice@example.com");
        assert_eq!(alice, subject_identity("alice@example.com"));
        assert_ne!(alice, subject_identity("bob@example.com"));
        assert!(
            !format!("{alice}").contains("alice@example.com"),
            "bucket keys must not persist the raw subject"
        );
    }

    #[test]
    fn test_auth_context_reports_enforcement() {
        let disabled = AuthContext::new(disabled_config(), ClientId::process(), None);
        assert!(!disabled.is_enforced());
        assert!(!disabled.config().enabled);

        let misconfigured = AuthContext::new(
            unenforceable_config(),
            ClientId::process(),
            Some("unused".to_string()),
        );
        assert!(!misconfigured.is_enforced());
        assert!(misconfigured.config().enabled);
    }

    #[test]
    fn test_auth_context_from_env_uses_process_identity() {
        let context = AuthContext::from_env(disabled_config());
        assert!(!context.is_enforced());
        // The process identity is the fallback principal for unauthenticated stdio
        let identity = context
            .resolve_identity("tools/list", OperationType::Write, None)
            .expect("disabled OAuth must not reject requests");
        assert_eq!(identity, ClientId::process());
    }

    #[test]
    fn test_resolve_identity_disabled_uses_process_identity() {
        let context = AuthContext::new(disabled_config(), ClientId::process(), None);
        for method in ["tools/list", "tools/call", "initialize"] {
            let identity = context
                .resolve_identity(method, OperationType::Write, None)
                .expect("disabled OAuth must not reject requests");
            assert_eq!(identity, ClientId::process());
        }
    }

    #[test]
    fn test_resolve_identity_unenforceable_config_fails_closed() {
        // Even discovery methods are rejected: the configuration claims to
        // enforce authorization while it cannot.
        let context = AuthContext::new(unenforceable_config(), ClientId::process(), None);
        for method in ["initialize", "tools/list", "tools/call"] {
            let rejection = context
                .resolve_identity(method, OperationType::Write, None)
                .expect_err("unenforceable configuration must fail closed");
            assert_eq!(rejection.code, UNAUTHORIZED_CODE);
            assert_eq!(rejection.error, "invalid_config");
            // The diagnostic names the missing secret (with the `oauth` feature)
            // or the missing feature itself, and always states that enforcement
            // is impossible.
            assert!(
                rejection.description.contains("cannot be enforced"),
                "unexpected diagnostic: {rejection:?}"
            );
        }
    }

    #[test]
    fn test_rejection_response_codes_and_messages() {
        let unauthorized = Rejection {
            code: UNAUTHORIZED_CODE,
            error: "unauthorized".to_string(),
            description: "missing bearer token".to_string(),
        };
        let response = rejection_response(Some(json!(7)), &unauthorized);
        assert_eq!(response.id, Some(json!(7)));
        assert!(response.result.is_none());
        let error = response.error.expect("rejection must carry an error");
        assert_eq!(error.code, UNAUTHORIZED_CODE);
        assert_eq!(error.message, "Unauthorized");
        assert_eq!(
            error.data,
            Some(json!({"error": "unauthorized", "description": "missing bearer token"}))
        );

        let scope = Rejection {
            code: -32003,
            error: "insufficient_scope".to_string(),
            description: "token is missing the required scope 'mcp:write'".to_string(),
        };
        let response = rejection_response(None, &scope);
        assert!(response.id.is_none());
        let error = response.error.expect("rejection must carry an error");
        assert_eq!(error.code, -32003);
        assert_eq!(error.message, "Insufficient scope");
    }

    #[test]
    fn test_resolve_identity_ignores_request_identity_fields() {
        // Regression for #1084: identity-like request fields are not a principal
        let params = json!({"_meta": {"client_id": "spoofed"}, "client_id": "spoofed"});
        let context = AuthContext::new(disabled_config(), ClientId::process(), None);
        let identity = context
            .resolve_identity("tools/call", OperationType::Write, Some(&params))
            .expect("disabled OAuth must not reject requests");
        assert_eq!(identity, ClientId::process());
    }

    #[cfg(feature = "oauth")]
    #[test]
    fn test_resolve_identity_authenticated_uses_subject() -> anyhow::Result<()> {
        let token = signed_token("mcp:read", "user-42")?;
        let params = json!({"_meta": {"authorization": format!("Bearer {token}")}});
        let context = AuthContext::new(enforced_config(), ClientId::process(), None);

        let identity = context
            .resolve_identity("tools/list", OperationType::Read, Some(&params))
            .map_err(|rejection| anyhow::anyhow!("{rejection:?}"))?;
        assert_eq!(identity, subject_identity("user-42"));
        assert_ne!(identity, ClientId::process());
        Ok(())
    }

    #[cfg(feature = "oauth")]
    #[test]
    fn test_resolve_identity_uses_transport_credential() -> anyhow::Result<()> {
        let token = signed_token("mcp:write", "user-43")?;
        let context = AuthContext::new(enforced_config(), ClientId::process(), Some(token.clone()));

        let identity = context
            .resolve_identity("tools/call", OperationType::Write, None)
            .map_err(|rejection| anyhow::anyhow!("{rejection:?}"))?;
        assert_eq!(identity, subject_identity("user-43"));
        Ok(())
    }

    #[cfg(feature = "oauth")]
    #[test]
    fn test_resolve_identity_enforced_rules() -> anyhow::Result<()> {
        let context = AuthContext::new(enforced_config(), ClientId::process(), None);

        // Discovery methods stay reachable without credentials
        assert_eq!(
            context
                .resolve_identity("initialize", OperationType::Read, None)
                .map_err(|rejection| anyhow::anyhow!("{rejection:?}"))?,
            ClientId::process()
        );

        // Protected methods require a token
        let rejection = context
            .resolve_identity("tools/list", OperationType::Read, None)
            .expect_err("protected method must require a token");
        assert_eq!(rejection.code, UNAUTHORIZED_CODE);
        assert_eq!(rejection.error, "unauthorized");

        // Invalid tokens are rejected without echoing the token
        let params = json!({"_meta": {"authorization": "Bearer not-a-jwt"}});
        let rejection = context
            .resolve_identity("tools/list", OperationType::Read, Some(&params))
            .expect_err("invalid token must be rejected");
        assert_eq!(rejection.error, "invalid_token");
        assert!(!rejection.description.contains("not-a-jwt"));

        // Valid token with the wrong scope is rejected with -32003
        let token = signed_token("mcp:read", "user-44")?;
        let params = json!({"_meta": {"authorization": format!("Bearer {token}")}});
        let rejection = context
            .resolve_identity("tools/call", OperationType::Write, Some(&params))
            .expect_err("read scope must not authorize a write");
        assert_eq!(rejection.code, -32003);
        assert_eq!(rejection.error, "insufficient_scope");
        Ok(())
    }
}
