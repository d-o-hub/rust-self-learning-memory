# Ownership & Borrowing Patterns

## own-borrow-over-clone

Prefer borrowing over cloning.

**Before:**
```rust
fn process(items: Vec<i32>) {
    for item in items {
        println!("{}", item);
    }
}

process(data.clone()); // Expensive!
```

**After:**
```rust
fn process(items: &[i32]) {
    for item in items {
        println!("{}", item);
    }
}

process(&data); // Zero cost
```

## own-slice-over-vec

Accept slices for flexibility.

```rust
// Accepts &Vec, &[T; N], and &[T]
fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

sum(&vec);
sum(&array);
sum(&slice);
```

## own-cow-smart

Cow for conditional ownership.

```rust
use std::borrow::Cow;

fn format_name(name: &str) -> Cow<'_, str> {
    if name.is_empty() {
        Cow::Borrowed("Anonymous")
    } else {
        Cow::Owned(name.to_uppercase())
    }
}
```

## own-box-recursion

Box for recursive types.

```rust
enum Tree<T> {
    Leaf(T),
    Node {
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}
```

## own-rc-shared

Rc for shared ownership (single-thread).

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let data2 = Rc::clone(&data); // Cheap ref count
// Both point to same data
```

## own-clone-explicit

Make cloning explicit.

```rust
// Clone is visible in call site
let copy = original.clone();

// Not implicit via Copy (unless cheap <= 16 bytes)
```
