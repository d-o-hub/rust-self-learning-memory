//! Tests for the `oauth` module (split out of it to keep the source file under the LOC ceiling).

use super::super::types::{AuthorizationResult, RequestAuthorization};
use do_memory_mcp::protocol::OAuthConfig;
use do_memory_mcp::server::rate_limiter::OperationType;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

const SECRET: &str = "unit-test-secret";
const ISSUER: &str = "https://auth.example.com";
const AUDIENCE: &str = "mcp-server";

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

#[test]
fn test_load_oauth_config_and_transport_token_from_lookup() {
    let env = HashMap::from([
        ("MCP_OAUTH_ENABLED", "true"),
        ("MCP_OAUTH_ISSUER", ISSUER),
        ("MCP_OAUTH_AUDIENCE", AUDIENCE),
        ("MCP_OAUTH_SCOPES", "mcp:read, mcp:write ,"),
        ("MCP_OAUTH_JWKS_URI", "https://auth.example.com/jwks"),
        ("MCP_OAUTH_TOKEN_SECRET", SECRET),
        ("MCP_OAUTH_TOKEN", "  env-token  "),
    ]);
    let lookup = |var: &str| env.get(var).map(|value| (*value).to_string());

    let config = super::load_oauth_config_from(&lookup);
    let transport = super::load_transport_token_from(&lookup);

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
    let alias = HashMap::from([("MCP_OAUTH_BEARER_TOKEN", "alias-token")]);
    let alias_lookup = |var: &str| alias.get(var).map(|value| (*value).to_string());
    assert_eq!(
        super::load_transport_token_from(&alias_lookup).as_deref(),
        Some("alias-token")
    );
}
