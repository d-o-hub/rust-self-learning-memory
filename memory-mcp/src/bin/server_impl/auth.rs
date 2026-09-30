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
