# API Design Patterns

## api-builder-pattern

Builder for complex construction.

```rust
let server = Server::builder()
    .bind("127.0.0.1:8080")
    .timeout(Duration::from_secs(30))
    .workers(4)
    .build()?;
```

## api-typestate

Typestate for compile-time states.

```rust
struct Unauthenticated;
struct Authenticated;

struct Client<S> {
    token: Option<String>,
    _state: PhantomData<S>,
}

impl Client<Unauthenticated> {
    fn new() -> Self { ... }
    
    fn authenticate(self, token: String) -> Client<Authenticated> {
        Client {
            token: Some(token),
            _state: PhantomData,
        }
    }
}

impl Client<Authenticated> {
    fn request(&self) { ... } // Only callable when authenticated
}
```

## api-extension-trait

Extend foreign types.

```rust
pub trait StringExt {
    fn is_palindrome(&self) -> bool;
    fn word_count(&self) -> usize;
}

impl StringExt for str {
    fn is_palindrome(&self) -> bool {
        self.chars().eq(self.chars().rev())
    }
    
    fn word_count(&self) -> usize {
        self.split_whitespace().count()
    }
}

// Usage
"hello".is_palindrome();
```

## api-impl-trait

Hide implementation with impl Trait.

```rust
// Caller doesn't know concrete type
pub fn users() -> impl Iterator<Item = User> {
    db.query_all().filter(|u| u.active)
}
```

## api-impl-into

Flexible argument types.

```rust
pub fn set_name(&mut self, name: impl Into<String>) {
    self.name = name.into();
}

// Accepts &str, String, Cow, etc.
obj.set_name("Alice");
obj.set_name(name.to_string());
```

## api-impl-asref

Accept borrowed inputs.

```rust
pub fn open(path: impl AsRef<Path>) -> Result<File> {
    std::fs::File::open(path)
}

// Accepts &str, PathBuf, &Path, etc.
open("file.txt");
open(PathBuf::from("file.txt"));
```
