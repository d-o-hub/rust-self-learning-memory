# Test Isolation Rules

## isolate-mock-external

Mock databases, APIs, files.

```rust
use mockall::automock;

#[automock]
trait Database {
    fn get_user(&self, id: u64) -> Option<User>;
}

#[test]
fn test_service_returns_user() {
    let mut mock = MockDatabase::new();
    mock.expect_get_user()
        .with(eq(1))
        .returning(|_| Some(User::default()));
    
    let service = UserService::new(mock);
    assert!(service.find_user(1).is_some());
}
```

## isolate-no-shared-state

No mutable shared state.

**Bad:**
```rust
static mut GLOBAL_COUNT: i32 = 0;

#[test]
fn test_a() { unsafe { GLOBAL_COUNT += 1; } }

#[test]
fn test_b() { unsafe { GLOBAL_COUNT += 1; } }
// Tests interfere with each other!
```

**Good:**
```rust
#[test]
fn test_counter_increments() {
    let counter = Cell::new(0);
    counter.set(counter.get() + 1);
    assert_eq!(counter.get(), 1);
}
```

## isolate-fresh-fixtures

New instances per test.

```rust
#[test]
fn test_deposit() {
    let account = Account::new(100); // Fresh instance
    account.deposit(50);
    assert_eq!(account.balance(), 150);
}

#[test]
fn test_withdraw() {
    let account = Account::new(100); // Fresh instance
    account.withdraw(30);
    assert_eq!(account.balance(), 70);
}
```

## isolate-deterministic

Same input → same output.

**Bad:**
```rust
#[test]
fn test_random() {
    let value = rand::random::<u32>();
    // Non-deterministic! Can fail randomly
}
```

**Good:**
```rust
#[test]
fn test_with_seed() {
    let mut rng = StdRng::seed_from_u64(42);
    let value = rng.gen::<u32>();
    // Deterministic with seed
}
```

## isolate-cleanup

Clean up after tests.

```rust
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join("test.txt");
        std::fs::File::create(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[test]
fn test_file_operations() {
    let temp = TempFile::new().unwrap();
    // Use temp file...
} // Auto-cleaned up
```

## isolate-parallel-safe

Tests run in parallel.

```rust
// Don't use global resources
// Don't assume test order
// Don't depend on shared state

#[test]
fn test_parallel_safe() {
    // Each test should be independent
    // Can run in any order
    // Can run simultaneously
}
```
