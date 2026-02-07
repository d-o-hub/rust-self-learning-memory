# Testing Rules

## test-cfg-test-module

Use `#[cfg(test)] mod tests { }`.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() { ... }
}
```

## test-use-super

Use `use super::*;` in test modules.

```rust
#[cfg(test)]
mod tests {
    use super::*; // Import everything from parent module
    
    #[test]
    fn test_private_fn() {
        // Can test private functions
        assert_eq!(private_add(2, 2), 4);
    }
}
```

## test-integration-dir

Put integration tests in `tests/` directory.

```
src/
  lib.rs
tests/
  integration_test.rs  // Integration tests
  common/
    mod.rs             // Shared test utilities
```

## test-descriptive-names

Use descriptive test names.

**Bad:**
```rust
#[test]
fn test1() { ... }
```

**Good:**
```rust
#[test]
fn test_withdraw_insufficient_funds_returns_error() { ... }
```

## test-arrange-act-assert

Structure tests as arrange/act/assert.

```rust
#[test]
fn test_deposit_increases_balance() {
    // Arrange
    let mut account = Account::new(100);
    
    // Act
    account.deposit(50);
    
    // Assert
    assert_eq!(account.balance(), 150);
}
```

## test-proptest-properties

Use `proptest` for property-based testing.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_is_commutative(a in 0..1000i32, b in 0..1000i32) {
        prop_assert_eq!(a + b, b + a);
    }
}
```

## test-mockall-mocking

Use `mockall` for trait mocking.

```rust
use mockall::automock;

#[automock]
trait Database {
    fn get_user(&self, id: u64) -> Option<User>;
}

#[test]
fn test_with_mock() {
    let mut mock = MockDatabase::new();
    mock.expect_get_user()
        .with(eq(1))
        .returning(|_| Some(User::default()));
    
    let service = UserService::new(mock);
    // ...
}
```

## test-mock-traits

Use traits for dependencies to enable mocking.

```rust
// Define trait
trait UserRepository {
    fn find(&self, id: u64) -> Option<User>;
}

// Real implementation
struct DbUserRepository { ... }

// Service depends on trait, not concrete type
struct UserService<R: UserRepository> {
    repo: R,
}
```

## test-fixture-raii

Use RAII pattern (Drop) for test cleanup.

```rust
struct TestDatabase {
    conn: Connection,
    name: String,
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Auto cleanup
        self.conn.execute(format!("DROP DATABASE {}", self.name));
    }
}
```

## test-tokio-async

Use `#[tokio::test]` for async tests.

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_fn().await;
    assert!(result.is_ok());
}
```

## test-should-panic

Use `#[should_panic]` for panic tests.

```rust
#[test]
#[should_panic(expected = "divide by zero")]
fn test_divide_by_zero() {
    divide(1, 0);
}
```

## test-criterion-bench

Use `criterion` for benchmarking.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 { ... }

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

## test-doctest-examples

Keep doc examples as executable tests.

```rust
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// let result = my_crate::add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
