# Test Organization

## org-unit-vs-integration

Clear separation.

```
src/
  lib.rs
  user.rs
  #[cfg(test)]
  mod tests { ... }  // Unit tests

tests/
  integration_test.rs  // Integration tests
  api_test.rs          // API tests
```

**Unit tests:**
- Test single function/module
- Fast (< 10ms)
- No I/O
- Mock dependencies

**Integration tests:**
- Test multiple components
- Slower (< 100ms)
- Real I/O (test DB)
- Minimal mocking

## org-describe-blocks

Group related tests.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod user_creation {
        use super::*;
        
        #[test]
        fn test_create_with_valid_data() { ... }
        
        #[test]
        fn test_create_with_invalid_email() { ... }
    }
    
    mod user_authentication {
        use super::*;
        
        #[test]
        fn test_login_with_correct_password() { ... }
        
        #[test]
        fn test_login_with_wrong_password() { ... }
    }
}
```

## org-setup-helper

Reusable fixtures.

```rust
fn test_db() -> TestDb {
    TestDb::new().unwrap()
}

fn test_user() -> User {
    User::new("test@example.com", "password123")
}

#[test]
fn test_user_can_login() {
    let db = test_db();
    let user = test_user();
    // Test...
}
```

## org-test-utils

Shared utilities.

```rust
// tests/common/mod.rs
pub fn setup_test_db() -> TestDb { ... }

pub fn create_test_user() -> User { ... }

pub fn assert_error_contains(err: &Error, msg: &str) { ... }
```

```rust
// tests/integration_test.rs
mod common;
use common::*;

#[test]
fn test_something() {
    let db = common::setup_test_db();
    // ...
}
```
