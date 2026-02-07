# Idiomatic Patterns

## idiom-raii-cleanup

RAII with Drop for cleanup.

```rust
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join("temp.txt");
        std::fs::File::create(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// Auto cleanup when out of scope
{
    let temp = TempFile::new()?;
    // Use temp...
} // Deleted automatically
```

## idiom-deref-smart-pointer

Deref for smart pointer behavior.

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        &self.0
    }
}

// Can use * or auto-deref
let my_box = MyBox(42);
assert_eq!(*my_box, 42);
```

## idiom-default-config

Default for configuration.

```rust
#[derive(Debug)]
struct Config {
    timeout: Duration,
    workers: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            workers: num_cpus::get(),
        }
    }
}

let config = Config::default();
```

## idiom-display-user-output

Display for user-facing output.

```rust
use std::fmt;

struct Money {
    cents: u64,
    currency: String,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{:02} {}", 
            self.cents / 100,
            self.cents % 100,
            self.currency
        )
    }
}

println!("Price: {}", Money { cents: 1999, currency: "USD".to_string() });
// Price: 19.99 USD
```

## idiom-debug-dev-output

Debug for developer output.

```rust
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let user = User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() };
println!("{:?}", user);
// User { id: 1, name: "Alice", email: "alice@example.com" }

println!("{:#?}", user); // Pretty print
```
