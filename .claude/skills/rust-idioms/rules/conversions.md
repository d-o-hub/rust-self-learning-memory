# Conversion Traits

## conv-from-over-into

Implement From, not Into.

```rust
// Implement From
impl From<String> for MyType {
    fn from(s: String) -> Self {
        Self { value: s }
    }
}

// Into is auto-derived
let my_type: MyType = string.into();
```

## conv-tryfrom-fallible

TryFrom for fallible conversions.

```rust
#[derive(Debug)]
struct Port(u16);

#[derive(Debug)]
struct PortError;

impl TryFrom<u16> for Port {
    type Error = PortError;
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 0 && value <= 65535 {
            Ok(Port(value))
        } else {
            Err(PortError)
        }
    }
}

let port = Port::try_from(8080)?;
```

## conv-asref-borrow

AsRef for cheap reference conversion.

```rust
impl AsRef<str> for MyString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

// Generic function accepts anything convertible to &str
fn print(s: impl AsRef<str>) {
    println!("{}", s.as_ref());
}
```

## conv-asmut-mutation

AsMut for mutable conversion.

```rust
impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
```

## conv-borrow-collections

Borrow for HashMap/HashSet keys.

```rust
use std::borrow::Borrow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Identifier(String);

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        &self.0
    }
}

// Can look up with &str even though key is Identifier
let mut map = HashMap::new();
map.insert(Identifier("key".to_string()), "value");
assert_eq!(map.get("key"), Some(&"value"));
```
