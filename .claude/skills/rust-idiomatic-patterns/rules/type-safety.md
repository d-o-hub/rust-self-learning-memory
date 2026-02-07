# Type Safety Rules

## type-newtype-ids

Wrap IDs in newtypes: `UserId(u64)`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(u64);

// Can't mix up UserId and OrderId
fn process(user_id: UserId, order_id: OrderId) { ... }
```

## type-newtype-validated

Newtypes for validated data: `Email`, `Url`.

```rust
pub struct Email(String);

impl Email {
    pub fn parse(s: &str) -> Result<Self, EmailError> {
        if s.contains('@') {
            Ok(Self(s.to_string()))
        } else {
            Err(EmailError::Invalid)
        }
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## type-enum-states

Use enums for mutually exclusive states.

```rust
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { session_id: String },
    Error { reason: String },
}
```

## type-option-nullable

Use `Option<T>` for nullable values.

```rust
struct User {
    name: String,
    email: Option<String>, // May be null
}
```

## type-result-fallible

Use `Result<T, E>` for fallible operations.

```rust
fn divide(a: f64, b: f64) -> Result<f64, DivisionError> {
    if b == 0.0 {
        Err(DivisionError::DivideByZero)
    } else {
        Ok(a / b)
    }
}
```

## type-phantom-marker

Use `PhantomData<T>` for type-level markers.

```rust
use std::marker::PhantomData;

struct Parser<'a> {
    input: &'a str,
    _marker: PhantomData<&'a ()>,
}
```

## type-never-diverge

Use `!` type for functions that never return.

```rust
fn exit(code: i32) -> ! {
    std::process::exit(code)
}
```

## type-generic-bounds

Add trait bounds only where needed.

**Bad:**
```rust
fn process<T: Display + Debug + Clone + Send + Sync>(item: T)
```

**Good:**
```rust
fn process<T: Display>(item: T) // Only require what's used
```

## type-no-stringly

Avoid stringly-typed APIs, use enums/newtypes.

**Bad:**
```rust
fn set_status(status: &str) { ... }
obj.set_status("active");
```

**Good:**
```rust
enum Status { Active, Inactive }
fn set_status(status: Status) { ... }
obj.set_status(Status::Active);
```

## type-repr-transparent

Use `#[repr(transparent)]` for FFI newtypes.

```rust
#[repr(transparent)]
pub struct Handle(*mut c_void);
```
