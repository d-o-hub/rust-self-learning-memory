# Memory Optimization Rules

## mem-with-capacity

Use `with_capacity()` when size is known.

**Bad:**
```rust
let mut items = Vec::new();
for i in 0..1000 {
    items.push(i); // Multiple reallocations
}
```

**Good:**
```rust
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i); // Single allocation
}
```

## mem-smallvec

Use `SmallVec` for usually-small collections.

```rust
use smallvec::SmallVec;

// Stack-allocated for ≤4 elements, heap for more
let mut items: SmallVec<[i32; 4]> = SmallVec::new();
```

## mem-arrayvec

Use `ArrayVec` for bounded-size collections.

```rust
use arrayvec::ArrayVec;

// Fixed capacity, no heap allocation
let mut items: ArrayVec<i32, 10> = ArrayVec::new();
```

## mem-box-large-variant

Box large enum variants to reduce type size.

**Bad:**
```rust
enum Message {
    Small(u8),
    Large([u8; 1000]), // Makes entire enum large
}
```

**Good:**
```rust
enum Message {
    Small(u8),
    Large(Box<[u8; 1000]>), // Only 8 bytes in enum
}
```

## mem-boxed-slice

Use `Box<[T]>` instead of `Vec<T>` when fixed.

```rust
// When you don't need to push/pop
let data: Box<[u8]> = vec![1, 2, 3].into_boxed_slice();
```

## mem-thinvec

Use `ThinVec` for often-empty vectors.

```rust
use thin_vec::ThinVec;

// Zero-size when empty
let items: ThinVec<i32> = ThinVec::new();
```

## mem-clone-from

Use `clone_from()` to reuse allocations.

```rust
// Reuses existing allocation in dest
dest.clone_from(&src);
```

## mem-reuse-collections

Reuse collections with `clear()` in loops.

```rust
let mut buffer = Vec::with_capacity(1024);
for chunk in chunks {
    buffer.clear();
    buffer.extend_from_slice(chunk);
    process(&buffer);
}
```

## mem-avoid-format

Avoid `format!()` when string literals work.

**Bad:**
```rust
let msg = format!("Error"); // Unnecessary allocation
```

**Good:**
```rust
let msg = "Error"; // Static string
```

## mem-write-over-format

Use `write!()` instead of `format!()` for building strings.

```rust
use std::fmt::Write;

let mut output = String::new();
write!(output, "Error: {}", err)?;
```

## mem-arena-allocator

Use arena allocators for batch allocations.

```rust
use bumpalo::Bump;

let arena = Bump::new();
let items: &mut [Item] = arena.alloc_slice_copy(&[item1, item2]);
```

## mem-zero-copy

Use zero-copy patterns with slices and `Bytes`.

```rust
use bytes::Bytes;

// Cheap cloning via reference counting
let data = Bytes::from(vec![1, 2, 3]);
let data2 = data.clone(); // O(1)
```

## mem-compact-string

Use `CompactString` for small string optimization.

```rust
use compact_str::CompactString;

// Stack-allocated for ≤24 bytes
let s = CompactString::new("hello");
```

## mem-smaller-integers

Use smallest integer type that fits.

```rust
// Ages 0-255 fit in u8
struct Person {
    age: u8, // Not u32 or u64
}
```

## mem-assert-type-size

Assert hot type sizes to prevent regressions.

```rust
#[test]
fn test_type_sizes() {
    assert_eq!(std::mem::size_of::<MyStruct>(), 32);
}
```
