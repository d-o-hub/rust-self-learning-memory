# Ownership & Borrowing Rules

## own-borrow-over-clone

Prefer `&T` borrowing over `.clone()` for better performance.

**Bad:**
```rust
fn process_user(user: User) { ... }
process_user(user.clone()); // Allocates entire User
```

**Good:**
```rust
fn process_user(user: &User) { ... }
process_user(&user); // Zero allocations
```

## own-slice-over-vec

Accept `&[T]` not `&Vec<T>`, `&str` not `&String` for flexibility.

**Bad:**
```rust
fn sum(numbers: &Vec<i32>) -> i32 { ... }
```

**Good:**
```rust
fn sum(numbers: &[i32]) -> i32 { ... }
```

## own-cow-conditional

Use `Cow<'a, T>` when ownership depends on conditions.

```rust
fn format_name(name: &str) -> Cow<'_, str> {
    if name.is_empty() {
        Cow::Borrowed("Anonymous")
    } else {
        Cow::Owned(name.to_uppercase())
    }
}
```

## own-arc-shared

Use `Arc<T>` for thread-safe shared ownership.

```rust
let data = Arc::new(vec![1, 2, 3]);
let data2 = data.clone(); // Cheap reference count bump
```

## own-rc-single-thread

Use `Rc<T>` for single-threaded sharing (cheaper than Arc).

```rust
let data = Rc::new(vec![1, 2, 3]);
let data2 = data.clone(); // Non-atomic ref count
```

## own-refcell-interior

Use `RefCell<T>` for interior mutability (single-thread only).

```rust
let counter = RefCell::new(0);
*counter.borrow_mut() += 1;
```

## own-mutex-interior

Use `Mutex<T>` for thread-safe interior mutability.

```rust
let counter = Arc::new(Mutex::new(0));
*counter.lock().unwrap() += 1;
```

## own-rwlock-readers

Use `RwLock<T>` when reads dominate writes.

```rust
let cache = RwLock::new(HashMap::new());
// Many readers can hold read lock concurrently
let data = cache.read().unwrap().get("key").cloned();
```

## own-copy-small

Derive `Copy` for small, trivial types (≤ 16 bytes).

```rust
#[derive(Clone, Copy)]
struct Point { x: f64, y: f64 } // 16 bytes, trivial
```

## own-clone-explicit

Make Clone explicit, avoid implicit copies.

**Bad:**
```rust
let s1 = String::from("hello");
let s2 = s1; // Implicit move (confusing)
```

**Good:**
```rust
let s1 = String::from("hello");
let s2 = s1.clone(); // Explicit clone
```

## own-move-large

Move large data instead of cloning.

**Bad:**
```rust
fn process(data: Vec<u8>) -> Vec<u8> {
    data.clone() // Expensive clone
}
```

**Good:**
```rust
fn process(mut data: Vec<u8>) -> Vec<u8> {
    data.sort(); // Mutate in place
    data
}
```

## own-lifetime-elision

Rely on lifetime elision when possible.

**Bad:**
```rust
fn first_word<'a>(s: &'a str) -> &'a str { ... }
```

**Good:**
```rust
fn first_word(s: &str) -> &str { ... } // Elided
```
