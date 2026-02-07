# Test Design Rules

## design-aaa-pattern

Arrange-Act-Assert structure.

```rust
#[test]
fn test_withdraw_decreases_balance() {
    // Arrange
    let mut account = Account::new(100);
    
    // Act
    account.withdraw(30);
    
    // Assert
    assert_eq!(account.balance(), 70);
}
```

**Benefits:**
- Clear test structure
- Easy to read
- Single responsibility
- Identifies setup issues

## design-behavior-over-impl

Test what, not how.

**Bad (50-80% more brittle):**
```rust
// Testing implementation details
test('sort calls quicksort with correct args', () => {
    const spy = jest.spyOn(utils, 'quicksort');
    sortUsers(users, 'name');
    expect(spy).toHaveBeenCalledWith(users, expect.any(Function));
});
```

**Good:**
```rust
// Testing observable behavior
#[test]
fn test_sort_returns_users_ordered_by_name() {
    let sorted = sort_users(users, "name");
    let names: Vec<_> = sorted.iter().map(|u| u.name.clone()).collect();
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
}
```

## design-one-assertion

One logical assertion per test.

**Bad:**
```rust
#[test]
fn test_user() {
    let user = create_user("Alice");
    assert_eq!(user.name, "Alice");
    assert!(user.id > 0);
    assert!(user.created_at <= now());
    assert!(!user.email.is_empty());
}
```

**Good:**
```rust
#[test]
fn test_user_has_correct_name() { ... }

#[test]
fn test_user_has_valid_id() { ... }

#[test]
fn test_user_has_creation_timestamp() { ... }
```

## design-independent-tests

No order dependency.

**Bad:**
```rust
static mut COUNTER: i32 = 0;

#[test]
fn test_first() { unsafe { COUNTER += 1; } }

#[test]
fn test_second() {
    // Depends on test_first running first!
    assert_eq!(unsafe { COUNTER }, 1);
}
```

**Good:**
```rust
#[test]
fn test_counter_increments() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

## design-edge-cases

Test boundary conditions.

**Examples:**
- Empty collections
- Single element
- Maximum values
- Null/None inputs
- Zero/negative numbers
- Very long strings

```rust
#[test]
fn test_sum_empty_list_returns_zero() {
    assert_eq!(sum(&[]), 0);
}

#[test]
fn test_sum_single_element() {
    assert_eq!(sum(&[42]), 42);
}
```

## design-error-paths

Test failure scenarios.

```rust
#[test]
fn test_divide_by_zero_returns_error() {
    let result = divide(10, 0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), DivisionError::DivideByZero);
}

#[test]
fn test_parse_invalid_date_fails() {
    let result = parse_date("not-a-date");
    assert!(result.is_err());
}
```
