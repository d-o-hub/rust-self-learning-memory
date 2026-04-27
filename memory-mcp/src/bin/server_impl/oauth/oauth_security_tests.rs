#[cfg(all(test, feature = "oauth"))]
mod tests {
    use crate::server_impl::AuthorizationResult;
    use crate::server_impl::oauth::{Claims, validate_bearer_token};
    use do_memory_mcp::protocol::OAuthConfig;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Helper to create a valid JWT for testing
    fn create_test_token(claims: &Claims, secret: &str) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    #[test]
    fn test_validate_bearer_token_success() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            issuer: Some("test-issuer".to_string()),
            audience: Some("test-audience".to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: Some("test-issuer".to_string()),
            sub: "test-user".to_string(),
            aud: Some(vec!["test-audience".to_string()]),
            exp: now() + 3600,
            iat: Some(now()),
            scope: Some("mcp:read".to_string()),
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::Authorized => (),
            _ => panic!("Expected Authorized, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_array_audience() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            audience: Some("test-audience".to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: None,
            sub: "test-user".to_string(),
            aud: Some(vec![
                "other-audience".to_string(),
                "test-audience".to_string(),
            ]),
            exp: now() + 3600,
            iat: Some(now()),
            scope: None,
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::Authorized => (),
            _ => panic!("Expected Authorized, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_missing_secret() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: None,
            ..Default::default()
        };

        let result = validate_bearer_token("some-token", &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("MCP_OAUTH_TOKEN_SECRET is not set or empty"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_empty_secret() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some("  ".to_string()),
            ..Default::default()
        };

        let result = validate_bearer_token("some-token", &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("MCP_OAUTH_TOKEN_SECRET is not set or empty"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_invalid_signature() {
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some("correct-secret".to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: None,
            sub: "test-user".to_string(),
            aud: None,
            exp: now() + 3600,
            iat: Some(now()),
            scope: None,
        };

        let token = create_test_token(&claims, "wrong-secret");
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("JWT validation failed: InvalidSignature"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_expired() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: None,
            sub: "test-user".to_string(),
            aud: None,
            exp: now() - 3600,
            iat: Some(now() - 7200),
            scope: None,
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("JWT validation failed: ExpiredSignature"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_invalid_issuer() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            issuer: Some("expected-issuer".to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: Some("wrong-issuer".to_string()),
            sub: "test-user".to_string(),
            aud: None,
            exp: now() + 3600,
            iat: Some(now()),
            scope: None,
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("JWT validation failed: InvalidIssuer"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_invalid_audience() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            audience: Some("expected-audience".to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: None,
            sub: "test-user".to_string(),
            aud: Some(vec!["wrong-audience".to_string()]),
            exp: now() + 3600,
            iat: Some(now()),
            scope: None,
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("JWT validation failed: InvalidAudience"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }

    #[test]
    fn test_validate_bearer_token_empty_subject() {
        let secret = "test-secret";
        let config = OAuthConfig {
            enabled: true,
            token_secret: Some(secret.to_string()),
            ..Default::default()
        };

        let claims = Claims {
            iss: None,
            sub: "  ".to_string(),
            aud: None,
            exp: now() + 3600,
            iat: Some(now()),
            scope: None,
        };

        let token = create_test_token(&claims, secret);
        let result = validate_bearer_token(&token, &config);

        match result {
            AuthorizationResult::InvalidToken(msg) => {
                assert!(msg.contains("Token missing or empty subject claim"));
            }
            _ => panic!("Expected InvalidToken, got {:?}", result),
        }
    }
}
