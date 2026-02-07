# Type Safety & Newtype Patterns

## type-newtype-units

Use newtype pattern for unit safety.

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Seconds(f64);

impl Meters {
    fn to_kilometers(&self) -> f64 {
        self.0 / 1000.0
    }
}

// Prevents mixing units
let distance = Meters(100.0);
let time = Seconds(10.0);
// speed = distance / time (compile-time unit checking)
```

## type-newtype-ids

Use newtypes for distinct IDs.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct OrderId(u64);

// Can't accidentally use OrderId where UserId expected
fn get_user(id: UserId) -> Option<User> { ... }
fn get_order(id: OrderId) -> Option<Order> { ... }
```

## type-newtype-validated

Newtypes with parse-time validation.

```rust
pub struct Email(String);

#[derive(Debug)]
pub enum EmailError {
    MissingAt,
    Empty,
}

impl Email {
    pub fn parse(s: &str) -> Result<Self, EmailError> {
        if s.is_empty() {
            return Err(EmailError::Empty);
        }
        if !s.contains('@') {
            return Err(EmailError::MissingAt);
        }
        Ok(Self(s.to_string()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## type-enum-states

Use enums for mutually exclusive states.

```rust
enum Connection {
    Disconnected,
    Connecting { since: Instant },
    Connected { session_id: String },
    Error { reason: String },
}

impl Connection {
    fn is_active(&self) -> bool {
        matches!(self, Connection::Connected { .. })
    }
}
```

## type-option-nullable

Use Option for nullable values.

```rust
struct User {
    name: String,
    email: Option<String>,  // May be null
    phone: Option<String>, // May be null
}

// Forces handling of null case
if let Some(email) = &user.email {
    send_email(email);
}
```

## type-phantom-marker

PhantomData for type-level markers.

```rust
use std::marker::PhantomData;

// Type parameter only used at compile time
struct Handle<T> {
    id: u64,
    _marker: PhantomData<T>,
}

// Different types for different resources
struct User;
struct Order;

let user_handle: Handle<User> = ...;
let order_handle: Handle<Order> = ...;
// Can't mix handles accidentally
```
