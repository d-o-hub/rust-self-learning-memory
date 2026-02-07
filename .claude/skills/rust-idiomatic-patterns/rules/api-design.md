# API Design Rules

## api-builder-pattern

Use Builder pattern for complex construction.

```rust
let server = Server::builder()
    .bind("127.0.0.1:8080")
    .timeout(Duration::from_secs(30))
    .workers(4)
    .build()?;
```

## api-builder-must-use

Add `#[must_use]` to builder types.

```rust
#[must_use]
pub struct ServerBuilder { ... }
```

## api-newtype-safety

Use newtypes for type-safe distinctions.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(u64);
```

## api-typestate

Use typestate for compile-time state machines.

```rust
struct Unauthenticated;
struct Authenticated;

struct Client<State> {
    token: Option<String>,
    _state: PhantomData<State>,
}

impl Client<Unauthenticated> {
    fn authenticate(self) -> Client<Authenticated> { ... }
}

impl Client<Authenticated> {
    fn request(&self) { ... } // Only available when authenticated
}
```

## api-sealed-trait

Seal traits to prevent external implementations.

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn method(&self);
}
```

## api-extension-trait

Use extension traits to add methods to foreign types.

```rust
pub trait StringExt {
    fn is_palindrome(&self) -> bool;
}

impl StringExt for str {
    fn is_palindrome(&self) -> bool {
        self.chars().eq(self.chars().rev())
    }
}
```

## api-parse-dont-validate

Parse into validated types at boundaries.

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
}
```

## api-impl-into

Accept `impl Into<T>` for flexible string inputs.

```rust
pub fn set_name(&mut self, name: impl Into<String>) {
    self.name = name.into();
}

// Accepts &str, String, Cow, etc.
obj.set_name("Alice");
obj.set_name(name.to_string());
```

## api-impl-asref

Accept `impl AsRef<T>` for borrowed inputs.

```rust
pub fn process_path(path: impl AsRef<Path>) {
    let path = path.as_ref();
    // ...
}
```

## api-must-use

Add `#[must_use]` to Result returning functions.

```rust
#[must_use = "futures do nothing unless polled"]
pub async fn fetch_data() -> Result<Data> { ... }
```

## api-non-exhaustive

Use `#[non_exhaustive]` for future-proof enums/structs.

```rust
#[non_exhaustive]
pub enum Status {
    Ok,
    Error,
}
```

## api-from-not-into

Implement `From`, not `Into` (auto-derived).

```rust
// Implement From, Into is auto-derived
impl From<String> for MyType {
    fn from(s: String) -> Self { ... }
}
```

## api-default-impl

Implement `Default` for sensible defaults.

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            workers: num_cpus::get(),
        }
    }
}
```

## api-common-traits

Implement `Debug`, `Clone`, `PartialEq` eagerly.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Point { ... }
```

## api-serde-optional

Gate `Serialize`/`Deserialize` behind feature flag.

```toml
[features]
serde = ["dep:serde"]
```

```rust
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MyType { ... }
```
