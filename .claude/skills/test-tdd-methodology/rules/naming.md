# Test Naming Rules

## name-descriptive

Full sentence describing behavior.

**Bad:**
```rust
#[test]
fn test_add() { ... }
#[test]
fn test_user() { ... }
```

**Good:**
```rust
#[test]
fn test_add_positive_numbers_returns_sum() { ... }
#[test]
fn test_create_user_persists_to_database() { ... }
```

## name-scenario-focused

Describe scenario, not method.

**Bad:**
```rust
#[test]
fn test_calculator_add() { ... }
```

**Good:**
```rust
#[test]
fn test_when_user_enters_valid_input_calculation_succeeds() { ... }
```

## name-expected-outcome

Include expected result.

```rust
#[test]
fn test_divide_by_zero_returns_division_error() { ... }

#[test]
fn test_empty_cart_checkout_shows_error_message() { ... }
```

## name-consistent-style

Consistent naming convention.

**Choose one:**
- `test_*
- `should_*`
- `when_*_then_*`

**Examples:**
```rust
// test_ prefix
#[test]
fn test_withdraw_insufficient_funds_returns_error() { ... }

// should_ prefix
#[test]
fn should_return_error_when_withdrawing_insufficient_funds() { ... }

// when_then_ pattern
#[test]
fn when_balance_is_low_then_withdraw_fails() { ... }
```

## name-no-test-prefix

Descriptive without "test_".

**Alternative naming:**
```rust
#[test]
fn withdraw_insufficient_funds_returns_error() { ... }

#[test]
fn parse_date_invalid_format_fails() { ... }
```

**Note:** Many teams prefer `#[test]` attribute is enough, name should describe behavior.
