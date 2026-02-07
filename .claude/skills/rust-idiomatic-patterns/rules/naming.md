# Naming Conventions

## name-types-camel

Use `UpperCamelCase` for types, traits, enums.

```rust
struct HttpRequest { ... }
trait AsyncRead { ... }
enum StatusCode { ... }
```

## name-variants-camel

Use `UpperCamelCase` for enum variants.

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

## name-funcs-snake

Use `snake_case` for functions, methods, modules.

```rust
fn process_data() { ... }
fn to_string(&self) -> String { ... }
```

## name-consts-screaming

Use `SCREAMING_SNAKE_CASE` for constants/statics.

```rust
const MAX_SIZE: usize = 100;
static GLOBAL_CONFIG: Config = Config::new();
```

## name-lifetime-short

Use short lowercase lifetimes: `'a`, `'de`, `'src`.

```rust
fn parse<'a>(input: &'a str) -> &'a str { ... }
```

## name-type-param-single

Use single uppercase for type params: `T`, `E`, `K`, `V`.

```rust
struct HashMap<K, V> { ... }
fn map<T, U>(f: impl Fn(T) -> U) { ... }
```

## name-as-free

`as_` prefix: free reference conversion.

```rust
fn as_slice(&self) -> &[Self::Item] { ... }
fn as_str(&self) -> &str { ... }
```

## name-to-expensive

`to_` prefix: expensive conversion.

```rust
fn to_string(&self) -> String { ... } // Allocates
fn to_vec(&self) -> Vec<T> { ... }   // Allocates
```

## name-into-ownership

`into_` prefix: ownership transfer.

```rust
fn into_bytes(self) -> Vec<u8> { ... } // Consumes self
```

## name-no-get-prefix

No `get_` prefix for simple getters.

**Bad:**
```rust
fn get_name(&self) -> &str { ... }
```

**Good:**
```rust
fn name(&self) -> &str { ... }
```

## name-is-has-bool

Use `is_`, `has_`, `can_` for boolean methods.

```rust
fn is_empty(&self) -> bool { ... }
fn has_key(&self, key: &K) -> bool { ... }
fn can_execute(&self) -> bool { ... }
```

## name-iter-convention

Use `iter`/`iter_mut`/`into_iter` for iterators.

```rust
fn iter(&self) -> Iter<T> { ... }
fn iter_mut(&mut self) -> IterMut<T> { ... }
fn into_iter(self) -> IntoIter<T> { ... }
```

## name-iter-method

Name iterator methods consistently.

```rust
fn users(&self) -> impl Iterator<Item = &User> { ... }
fn users_mut(&mut self) -> impl Iterator<Item = &mut User> { ... }
fn into_users(self) -> impl Iterator<Item = User> { ... }
```

## name-iter-type-match

Iterator type names match method.

```rust
fn iter(&self) -> Iter<T> { ... }
fn iter_mut(&mut self) -> IterMut<T> { ... }
```

## name-acronym-word

Treat acronyms as words: `Uuid` not `UUID`.

```rust
struct Uuid { ... }     // Not UUID
struct HttpClient { ... } // Not HTTPClient
```

## name-crate-no-rs

Crate names: no `-rs` suffix.

```toml
# Good
[package]
name = "serde"

# Bad
name = "serde-rs"
```
