#![cfg(feature = "oauth")]

use do_memory_mcp::protocol::OAuthConfig;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use server_impl::oauth::validate_bearer_token;
use server_impl::types::AuthorizationResult;

#[path = "../src/bin/server_impl/mod.rs"]
mod server_impl;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    sub: String,
    scope: Option<String>,
}

#[test]
fn test_validate_bearer_token_security() {
    let secret = "super-secret-key";
    let config = OAuthConfig {
        enabled: true,
        issuer: Some("https://auth.example.com".to_string()),
        audience: Some("mcp-server".to_string()),
        token_secret: Some(secret.to_string()),
        ..OAuthConfig::default()
    };

    // 1. Test Valid Token
    let my_claims = Claims {
        iss: Some("https://auth.example.com".to_string()),
        aud: Some("mcp-server".to_string()),
        exp: Some(jsonwebtoken::get_current_timestamp() + 10000),
        sub: "user123".to_string(),
        scope: Some("mcp:read".to_string()),
    };
    let valid_token = encode(
        &Header::new(Algorithm::HS256),
        &my_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let result = validate_bearer_token(&valid_token, &config);
    assert!(
        matches!(result, AuthorizationResult::Authorized),
        "Valid token should be authorized"
    );

    // 2. Test Forged Token (wrong secret)
    let forged_token = encode(
        &Header::new(Algorithm::HS256),
        &my_claims,
        &EncodingKey::from_secret("wrong-secret".as_bytes()),
    )
    .unwrap();
    let result = validate_bearer_token(&forged_token, &config);
    assert!(
        matches!(result, AuthorizationResult::InvalidToken(_)),
        "Forged token should be rejected"
    );

    // 3. Test Expired Token
    let expired_claims = Claims {
        exp: Some(jsonwebtoken::get_current_timestamp() - 10000),
        ..my_claims.clone()
    };
    let expired_token = encode(
        &Header::new(Algorithm::HS256),
        &expired_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let result = validate_bearer_token(&expired_token, &config);
    assert!(
        matches!(result, AuthorizationResult::InvalidToken(msg) if msg.contains("Expired")),
        "Expired token should be rejected"
    );

    // 4. Test Mismatched Issuer
    let wrong_iss_claims = Claims {
        iss: Some("https://wrong.example.com".to_string()),
        ..my_claims.clone()
    };
    let wrong_iss_token = encode(
        &Header::new(Algorithm::HS256),
        &wrong_iss_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let result = validate_bearer_token(&wrong_iss_token, &config);
    assert!(
        matches!(result, AuthorizationResult::InvalidToken(msg) if msg.contains("InvalidIssuer")),
        "Mismatched issuer should be rejected"
    );

    // 5. Test Mismatched Audience
    let wrong_aud_claims = Claims {
        aud: Some("wrong-audience".to_string()),
        ..my_claims.clone()
    };
    let wrong_aud_token = encode(
        &Header::new(Algorithm::HS256),
        &wrong_aud_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let result = validate_bearer_token(&wrong_aud_token, &config);
    assert!(
        matches!(result, AuthorizationResult::InvalidToken(msg) if msg.contains("InvalidAudience")),
        "Mismatched audience should be rejected"
    );
}

#[test]
fn test_validate_bearer_token_insecure_fallback() {
    let config = OAuthConfig {
        enabled: true,
        issuer: Some("https://auth.example.com".to_string()),
        audience: Some("mcp-server".to_string()),
        token_secret: None, // NO SECRET
        ..OAuthConfig::default()
    };

    let my_claims = Claims {
        iss: Some("https://auth.example.com".to_string()),
        aud: Some("mcp-server".to_string()),
        exp: Some(jsonwebtoken::get_current_timestamp() + 10000),
        sub: "user123".to_string(),
        scope: Some("mcp:read".to_string()),
    };

    // Create a token with ANY secret (or none)
    let token = encode(
        &Header::new(Algorithm::HS256),
        &my_claims,
        &EncodingKey::from_secret("anyone-can-sign".as_bytes()),
    )
    .unwrap();

    let result = validate_bearer_token(&token, &config);

    // Should be accepted because secret is missing (legacy behavior preserved but now warned)
    assert!(
        matches!(result, AuthorizationResult::Authorized),
        "Should accept any token if no secret is configured (legacy behavior)"
    );
}
